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
}
