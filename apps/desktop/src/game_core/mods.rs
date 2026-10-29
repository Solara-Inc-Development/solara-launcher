//! Read local mod metadata without executing JARs. Enable/disable is a file rename.
use super::{
    files::{self, Files},
    instances::Instances,
};
use anyhow::{bail, Context, Result};
use serde::Serialize;
use serde_json::Value;
use std::{
    io::{Read, Seek, SeekFrom},
    path::Path,
};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Mod {
    pub path: String,
    pub file_name: String,
    pub name: String,
    pub mod_id: Option<String>,
    pub version: Option<String>,
    pub description: Option<String>,
    pub loaders: Vec<String>,
    pub enabled: bool,
    pub size: u64,
    pub issue: Option<String>,
}
fn bounded(text: Option<&str>, limit: usize) -> Option<String> {
    text.map(|t| t.chars().take(limit).collect::<String>())
        .filter(|s| !s.is_empty())
}
fn inspect(
    path: &Path,
) -> Result<(
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Vec<String>,
)> {
    let mut file = std::fs::File::open(path)?;
    let length = file.metadata()?.len();
    if length > 512 * 1024 * 1024 {
        bail!("Mod file is too large")
    }
    // Bound central directory allocation before the ZIP reader builds its index.
    let tail = length.min(65557);
    file.seek(SeekFrom::End(-(tail as i64)))?;
    let mut bytes = vec![0; tail as usize];
    file.read_exact(&mut bytes)?;
    let at = bytes
        .windows(4)
        .rposition(|w| w == b"PK\x05\x06")
        .context("Mod file is not a supported JAR")?;
    if at + 22 > bytes.len() {
        bail!("Invalid JAR index")
    }
    let count = u16::from_le_bytes([bytes[at + 10], bytes[at + 11]]);
    let central = u32::from_le_bytes(bytes[at + 12..at + 16].try_into()?);
    if count > 20000 || central > 8 * 1024 * 1024 {
        bail!("JAR metadata exceeds limit")
    }
    file.seek(SeekFrom::Start(0))?;
    let mut zip = zip::ZipArchive::new(file)?;
    fn read(zip: &mut zip::ZipArchive<std::fs::File>, name: &str) -> Result<Option<String>> {
        let mut entry = match zip.by_name(name) {
            Ok(e) => e,
            Err(zip::result::ZipError::FileNotFound) => return Ok(None),
            Err(e) => return Err(e.into()),
        };
        if entry.size() > 256 * 1024 {
            bail!("Mod manifest exceeds limit")
        }
        let mut text = String::new();
        entry
            .by_ref()
            .take(256 * 1024 + 1)
            .read_to_string(&mut text)?;
        if text.len() > 256 * 1024 {
            bail!("Mod manifest exceeds limit")
        }
        Ok(Some(text))
    }
    if let Some(text) = read(&mut zip, "fabric.mod.json")? {
        let v: Value = serde_json::from_str(&text)?;
        return Ok((
            bounded(v["name"].as_str(), 160),
            bounded(v["id"].as_str(), 160),
            bounded(v["version"].as_str(), 100),
            bounded(v["description"].as_str(), 2000),
            vec!["fabric".into()],
        ));
    }
    if let Some(text) = read(&mut zip, "quilt.mod.json")? {
        let v: Value = serde_json::from_str(&text)?;
        let q = &v["quilt_loader"];
        return Ok((
            bounded(q["metadata"]["name"].as_str(), 160),
            bounded(q["id"].as_str(), 160),
            bounded(q["version"].as_str(), 100),
            bounded(q["metadata"]["description"].as_str(), 2000),
            vec!["quilt".into()],
        ));
    }
    for (manifest, loader) in [
        ("META-INF/neoforge.mods.toml", "neoforge"),
        ("META-INF/mods.toml", "forge"),
    ] {
        if let Some(text) = read(&mut zip, manifest)? {
            let v: toml::Value = toml::from_str(&text)?;
            let m = v
                .get("mods")
                .and_then(toml::Value::as_array)
                .and_then(|a| a.first())
                .context("Mod manifest has no mod entry")?;
            let value = |key: &str| m.get(key).and_then(toml::Value::as_str);
            let version = bounded(value("version"), 100).filter(|v| !v.starts_with("${"));
            return Ok((
                bounded(value("displayName"), 160),
                bounded(value("modId"), 160),
                version,
                bounded(value("description"), 2000),
                vec![loader.into()],
            ));
        }
    }
    if let Some(text) = read(&mut zip, "mcmod.info")? {
        let v: Value = serde_json::from_str(&text)?;
        let m = if v.is_array() {
            &v[0]
        } else {
            &v["modList"][0]
        };
        return Ok((
            bounded(m["name"].as_str(), 160),
            bounded(m["modid"].as_str(), 160),
            bounded(m["version"].as_str(), 100),
            bounded(m["description"].as_str(), 2000),
            vec!["forge".into()],
        ));
    }
    Ok((None, None, None, None, vec![]))
}
pub async fn list(store: &Instances, files: &Files, id: &str) -> Result<Vec<Mod>> {
    if !tokio::fs::try_exists(store.game_dir(id).await?.join("mods")).await? {
        return Ok(vec![]);
    }
    let entries = files.list(store, id, "mods").await?;
    let mut out = Vec::new();
    for entry in entries {
        let lower = entry.name.to_ascii_lowercase();
        if entry.directory || !(lower.ends_with(".jar") || lower.ends_with(".jar.disabled")) {
            continue;
        }
        if out.len() >= 2000 {
            bail!("This instance has too many mods to list at once")
        }
        let path = files::resolve(store, id, &entry.path, false).await?;
        let parsed = tokio::task::spawn_blocking(move || inspect(&path)).await?;
        let (name, mod_id, version, description, loaders, issue) = match parsed {
            Ok((a, b, c, d, e)) => (a, b, c, d, e, None),
            Err(_) => (
                None,
                None,
                None,
                None,
                vec![],
                Some("Could not read this mod's details".into()),
            ),
        };
        out.push(Mod {
            path: entry.path,
            file_name: entry.name.clone(),
            name: name.unwrap_or_else(|| {
                entry
                    .name
                    .trim_end_matches(".disabled")
                    .trim_end_matches(".jar")
                    .into()
            }),
            mod_id,
            version,
            description,
            loaders,
            enabled: lower.ends_with(".jar"),
            size: entry.size,
            issue,
        });
    }
    out.sort_by_key(|m| m.name.to_lowercase());
    Ok(out)
}
pub async fn toggle(
    store: &Instances,
    files: &Files,
    id: &str,
    path: &str,
    enabled: bool,
) -> Result<()> {
    if !path.starts_with("mods/") || path[5..].contains('/') {
        bail!("Choose a mod in this instance's mods folder")
    }
    let lower = path.to_ascii_lowercase();
    let target = if enabled && lower.ends_with(".jar.disabled") {
        path[..path.len() - 9].to_string()
    } else if !enabled && lower.ends_with(".jar") {
        format!("{path}.disabled")
    } else {
        bail!("This mod is already in the requested state")
    };
    files.rename(store, id, path, &target).await
}

fn validate_jar_file_name(file_name: &str) -> Result<String> {
    let trimmed = file_name.trim();
    if trimmed.len() <= 4
        || trimmed.starts_with('.')
        || trimmed.contains('/')
        || trimmed.contains('\\')
        || !trimmed.to_ascii_lowercase().ends_with(".jar")
    {
        bail!("Choose a valid .jar mod file")
    }
    let stem = &trimmed[..trimmed.len() - 4];
    if stem.trim().is_empty() || stem.ends_with([' ', '.']) {
        bail!("Choose a valid .jar mod file")
    }
    let normalized = format!("{stem}.jar");
    super::metadata::relative_path(&format!("mods/{normalized}"))?;
    Ok(normalized)
}

async fn ensure_no_mod_conflict(mods_dir: &Path, clean_name: &str) -> Result<()> {
    if !tokio::fs::try_exists(mods_dir).await? {
        return Ok(());
    }
    let lower_jar = clean_name.to_ascii_lowercase();
    let lower_disabled = format!("{lower_jar}.disabled");
    let mut rd = tokio::fs::read_dir(mods_dir).await?;
    while let Some(entry) = rd.next_entry().await? {
        let existing = entry.file_name().to_string_lossy().to_ascii_lowercase();
        if existing == lower_jar || existing == lower_disabled {
            bail!("A mod file with that name already exists in this instance")
        }
    }
    Ok(())
}

fn verify_imported_jar(
    path: &Path,
) -> Result<(
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Vec<String>,
)> {
    let parsed = inspect(path)?;
    let file = std::fs::File::open(path)?;
    let mut zip = zip::ZipArchive::new(file).context("Mod file is not a valid .jar archive")?;
    if zip.is_empty() {
        bail!("Mod .jar archive is empty")
    }
    if zip.len() > 8_192 {
        bail!("Mod .jar archive contains too many entries")
    }
    let mut has_manifest_or_class = !parsed.4.is_empty();
    let mut total_uncompressed = 0u64;
    for i in 0..zip.len() {
        let entry = zip.by_index_raw(i)?;
        if entry.is_symlink()
            || entry
                .unix_mode()
                .is_some_and(|mode| mode & 0o170000 == 0o120000)
        {
            bail!("Mod .jar archive contains an unsupported link entry")
        }
        let name = entry.name().replace('\\', "/");
        if name.starts_with('/')
            || name.contains(':')
            || name.contains('\0')
            || name
                .split('/')
                .any(|seg| seg == "." || seg == ".." || seg.chars().any(char::is_control))
        {
            bail!("Mod .jar archive contains an unsafe entry path")
        }
        let size = entry.size();
        total_uncompressed = total_uncompressed.saturating_add(size);
        if size > 256 * 1024 * 1024 || total_uncompressed > 256 * 1024 * 1024 {
            bail!("Mod .jar uncompressed contents exceed the 256 MiB limit")
        }
        if name == "META-INF/MANIFEST.MF" || name.ends_with(".class") {
            has_manifest_or_class = true;
        }
    }
    if !has_manifest_or_class {
        bail!("This .jar file does not contain valid Minecraft mod metadata")
    }
    Ok(parsed)
}

pub async fn import_bytes(
    store: &Instances,
    id: &str,
    file_name: &str,
    bytes: &[u8],
) -> Result<Mod> {
    let instance = store.get(id).await?;
    if instance.loader == "vanilla" {
        bail!("Vanilla instances do not load mods. Use a Fabric or Quilt instance.")
    }
    let clean_name = validate_jar_file_name(file_name)?;
    if bytes.is_empty() || bytes.len() > 128 * 1024 * 1024 {
        bail!("Choose a .jar mod file up to 128 MiB")
    }
    let rel = format!("mods/{clean_name}");
    let disabled_rel = format!("{rel}.disabled");
    let target = files::resolve_create_parents(store, id, &rel).await?;
    let disabled = files::resolve(store, id, &disabled_rel, true).await?;
    let mods_dir = target.parent().context("Mods folder unavailable")?;
    if tokio::fs::symlink_metadata(&target).await.is_ok()
        || tokio::fs::symlink_metadata(&disabled).await.is_ok()
    {
        bail!("A mod file with that name already exists in this instance")
    }
    ensure_no_mod_conflict(mods_dir, &clean_name).await?;
    let temp = target.with_file_name(format!(".solara-mod-{}.tmp", uuid::Uuid::new_v4()));
    let write_res = async {
        use tokio::io::AsyncWriteExt;
        let mut f = tokio::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)
            .await?;
        f.write_all(bytes).await?;
        f.sync_all().await?;
        Ok::<_, anyhow::Error>(())
    }
    .await;
    if let Err(e) = write_res {
        let _ = tokio::fs::remove_file(&temp).await;
        return Err(e);
    }
    let temp_check = temp.clone();
    let verified = tokio::task::spawn_blocking(move || verify_imported_jar(&temp_check)).await?;
    let (name, mod_id, version, description, loaders) = match verified {
        Ok(v) => v,
        Err(e) => {
            let _ = tokio::fs::remove_file(&temp).await;
            return Err(e);
        }
    };
    if tokio::fs::symlink_metadata(&target).await.is_ok()
        || tokio::fs::symlink_metadata(&disabled).await.is_ok()
    {
        let _ = tokio::fs::remove_file(&temp).await;
        bail!("A mod file with that name already exists in this instance")
    }
    if let Err(e) = ensure_no_mod_conflict(mods_dir, &clean_name).await {
        let _ = tokio::fs::remove_file(&temp).await;
        return Err(e);
    }
    if let Err(e) = tokio::fs::rename(&temp, &target).await {
        let _ = tokio::fs::remove_file(&temp).await;
        return Err(e.into());
    }
    Ok(Mod {
        path: rel,
        file_name: clean_name.clone(),
        name: name.unwrap_or_else(|| clean_name.trim_end_matches(".jar").into()),
        mod_id,
        version,
        description,
        loaders,
        enabled: true,
        size: bytes.len() as u64,
        issue: None,
    })
}

pub async fn import_path(store: &Instances, id: &str, source_path: &Path) -> Result<Mod> {
    let meta = tokio::fs::symlink_metadata(source_path)
        .await
        .context("Selected mod file is unavailable")?;
    if files::is_link(&meta) {
        bail!("Linked mod files cannot be imported")
    }
    if !meta.is_file() || meta.len() == 0 || meta.len() > 128 * 1024 * 1024 {
        bail!("Choose a .jar mod file up to 128 MiB")
    }
    let file_name = source_path
        .file_name()
        .and_then(|n| n.to_str())
        .context("Selected mod file name is not valid Unicode")?;
    let bytes = tokio::fs::read(source_path).await?;
    import_bytes(store, id, file_name, &bytes).await
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reads_real_jar_manifest_without_execution() {
        use std::io::Write;
        let dir = std::env::temp_dir().join(format!("solara-mod-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&dir).unwrap();
        let path = dir.join("fixture.jar");
        let mut zip = zip::ZipWriter::new(std::fs::File::create(&path).unwrap());
        zip.start_file("fabric.mod.json", zip::write::SimpleFileOptions::default())
            .unwrap();
        zip.write_all(br#"{"id":"fixture","name":"Fixture mod","version":"1.2.3"}"#)
            .unwrap();
        zip.finish().unwrap();
        let result = inspect(&path).unwrap();
        assert_eq!(result.1.as_deref(), Some("fixture"));
        assert_eq!(result.2.as_deref(), Some("1.2.3"));
        assert_eq!(result.4, vec!["fabric"]);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[tokio::test]
    async fn import_validates_jar_handles_conflicts_and_rejects_links_or_vanilla() {
        use std::io::Write;
        let root =
            std::env::temp_dir().join(format!("solara-mod-import-test-{}", uuid::Uuid::new_v4()));
        let store = Instances::new(root.clone());
        let fs = Files::default();
        let fabric_inst = store
            .create(
                "Fabric Test".into(),
                "1.21.1".into(),
                "fabric".into(),
                Some("0.16.9".into()),
                2048,
                "auto".into(),
            )
            .await
            .unwrap();
        let vanilla_inst = store
            .create(
                "Vanilla Test".into(),
                "1.21.1".into(),
                "vanilla".into(),
                None,
                2048,
                "auto".into(),
            )
            .await
            .unwrap();

        let mut jar_buf = std::io::Cursor::new(Vec::new());
        {
            let mut zip = zip::ZipWriter::new(&mut jar_buf);
            zip.start_file("quilt.mod.json", zip::write::SimpleFileOptions::default())
                .unwrap();
            zip.write_all(
                br#"{"quilt_loader":{"id":"zoom_mod","version":"2.0.0","metadata":{"name":"Zoom Mod","description":"Clean zoom"}}}"#,
            )
            .unwrap();
            zip.finish().unwrap();
        }
        let jar_bytes = jar_buf.into_inner();

        // Reject importing into vanilla instance
        assert!(
            import_bytes(&store, &vanilla_inst.id, "zoom.jar", &jar_bytes)
                .await
                .is_err()
        );

        // Reject corrupt/non-jar bytes, dotfile names, and non-.jar filenames
        assert!(
            import_bytes(&store, &fabric_inst.id, "zoom.jar", b"not-a-zip")
                .await
                .is_err()
        );
        assert!(
            import_bytes(&store, &fabric_inst.id, ".hidden.jar", &jar_bytes)
                .await
                .is_err()
        );
        assert!(
            import_bytes(&store, &fabric_inst.id, "zoom.exe", &jar_bytes)
                .await
                .is_err()
        );

        // Import valid JAR into fresh Fabric instance (creates game/mods/ automatically)
        let imported = import_bytes(&store, &fabric_inst.id, "zoom.jar", &jar_bytes)
            .await
            .unwrap();
        assert_eq!(imported.name, "Zoom Mod");
        assert_eq!(imported.mod_id.as_deref(), Some("zoom_mod"));
        assert_eq!(imported.version.as_deref(), Some("2.0.0"));
        assert_eq!(imported.loaders, vec!["quilt"]);
        assert!(imported.enabled);

        // Reject duplicate import (even if disabled or with different casing)
        assert!(
            import_bytes(&store, &fabric_inst.id, "zoom.jar", &jar_bytes)
                .await
                .is_err()
        );
        assert!(
            import_bytes(&store, &fabric_inst.id, "ZOOM.JAR", &jar_bytes)
                .await
                .is_err()
        );
        toggle(&store, &fs, &fabric_inst.id, "mods/zoom.jar", false)
            .await
            .unwrap();
        assert!(
            import_bytes(&store, &fabric_inst.id, "zoom.jar", &jar_bytes)
                .await
                .is_err()
        );
        assert!(
            import_bytes(&store, &fabric_inst.id, "Zoom.JAR", &jar_bytes)
                .await
                .is_err()
        );

        #[cfg(unix)]
        {
            let src_jar = root.join("real-source.jar");
            tokio::fs::write(&src_jar, &jar_bytes).await.unwrap();
            let link_jar = root.join("linked-source.jar");
            std::os::unix::fs::symlink(&src_jar, &link_jar).unwrap();
            assert!(import_path(&store, &fabric_inst.id, &link_jar)
                .await
                .is_err());
            let from_path = import_path(&store, &fabric_inst.id, &src_jar)
                .await
                .unwrap();
            assert_eq!(from_path.file_name, "real-source.jar");
        }

        tokio::fs::remove_dir_all(root).await.unwrap();
    }
}
