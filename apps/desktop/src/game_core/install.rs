//! Independent vanilla installation. User worlds live in game/, never runtime/.
use super::{
    download::{Cancellation, FileCache},
    instances::{write_json, Instances},
    metadata::{self, Artifact},
    plan,
};
use anyhow::{bail, Context, Result};
use serde::Serialize;
use serde_json::Value;
use std::path::Path;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Progress {
    pub phase: String,
    pub verified_bytes: u64,
    pub total_bytes: u64,
}

pub async fn instance(
    store: &Instances,
    cache: &FileCache,
    id: &str,
    cancel: &Cancellation,
    os_version: Option<&str>,
    mut report: impl FnMut(Progress) + Send,
) -> Result<()> {
    cancel.check()?;
    let instance = store.get(id).await?;
    if !matches!(instance.loader.as_str(), "vanilla" | "fabric") {
        bail!("This loader is not supported by the installer yet")
    }
    store
        .transition(id, &["created", "failed", "cancelled"], "installing")
        .await?;
    let stage = store
        .path(id)?
        .join(format!("runtime.install-{}", uuid::Uuid::new_v4()));
    let result = async {
        report(Progress {
            phase: "Reading official Minecraft files".into(),
            verified_bytes: 0,
            total_bytes: 0,
        });
        cancel.check()?;
        let entries = metadata::versions().await?;
        let entry = entries
            .iter()
            .find(|v| v.id == instance.game_version)
            .context("Minecraft version is not in the official list")?;
        let document = metadata::version(entry).await?;
        let mut plan = plan::windows(&document, os_version)?;
        if instance.loader == "fabric" {
            cancel.check()?;
            let loader = instance
                .loader_version
                .as_deref()
                .context("Fabric loader version missing")?;
            let profile = super::loaders::fabric_profile(&instance.game_version, loader).await?;
            super::loaders::apply_fabric(&mut plan, &profile, os_version)?;
        }
        if plan.asset_index.size > 16 * 1024 * 1024 {
            bail!("Minecraft asset index exceeds limit")
        }
        // Build a complete bounded file plan before downloading the actual game.
        let index = cache
            .obtain(&plan.asset_index, cancel, |done, total| {
                report(Progress {
                    phase: "Reading the asset list".into(),
                    verified_bytes: done,
                    total_bytes: total,
                })
            })
            .await?;
        let assets: Value = serde_json::from_slice(&tokio::fs::read(index).await?)?;
        if assets["virtual"].as_bool() == Some(true)
            || assets["map_to_resources"].as_bool() == Some(true)
        {
            bail!("This legacy asset layout is not supported yet")
        }
        let objects = assets["objects"]
            .as_object()
            .context("Invalid Minecraft asset index")?;
        if objects.len() > 100000 {
            bail!("Minecraft asset count exceeds limit")
        }
        let mut assets_plan = Vec::new();
        for object in objects.values() {
            let hash = object["hash"].as_str().context("Missing asset checksum")?;
            if hash.len() != 40 || !hash.bytes().all(|b| b.is_ascii_hexdigit()) {
                bail!("Invalid asset checksum")
            }
            let hash = hash.to_ascii_lowercase();
            let artifact = Artifact {
                url: format!(
                    "https://resources.download.minecraft.net/{}/{hash}",
                    &hash[..2]
                ),
                sha1: hash.clone(),
                size: object["size"].as_u64().context("Invalid asset size")?,
                path: None,
            };
            artifact.validate()?;
            assets_plan.push((format!("assets/objects/{}/{hash}", &hash[..2]), artifact));
        }
        // Several logical asset names can use the same content hash.
        assets_plan.sort_by(|a, b| a.0.cmp(&b.0));
        assets_plan.dedup_by(|a, b| a.0 == b.0);
        let mut files: Vec<(String, Artifact)> = plan
            .files
            .iter()
            .map(|f| (f.destination.clone(), f.artifact.clone()))
            .collect();
        files.extend(assets_plan);
        for native in &plan.natives {
            files.push((format!("native-jars/{}.jar", native.sha1), native.clone()));
        }
        let total = files
            .iter()
            .try_fold(plan.asset_index.size, |sum, (_, file)| {
                sum.checked_add(file.size)
                    .context("Minecraft install size overflow")
            })?;
        if total > 32 * 1024 * 1024 * 1024 {
            bail!("Minecraft installation exceeds size limit")
        }
        tokio::fs::create_dir(&stage).await?;
        tokio::fs::create_dir_all(stage.join("assets/indexes")).await?;
        tokio::fs::create_dir(stage.join("natives")).await?;
        metadata::relative_path(&format!("{}.json", plan.asset_index_id))?;
        write_json(
            &stage
                .join("assets/indexes")
                .join(format!("{}.json", plan.asset_index_id)),
            &assets,
        )
        .await?;
        let mut complete = plan.asset_index.size;
        for (position, (destination, artifact)) in files.iter().enumerate() {
            cancel.check()?;
            metadata::relative_path(destination)?;
            let phase = format!("Preparing file {} of {}", position + 1, files.len());
            let source = cache
                .obtain(artifact, cancel, |done, _| {
                    report(Progress {
                        phase: phase.clone(),
                        verified_bytes: complete.saturating_add(done),
                        total_bytes: total,
                    })
                })
                .await?;
            let target = stage.join(destination);
            tokio::fs::create_dir_all(target.parent().context("Invalid install destination")?)
                .await?;
            // Copy, do not hard-link: one instance cannot mutate another's cache.
            tokio::fs::copy(source, &target).await?;
            complete = complete.saturating_add(artifact.size);
        }
        for native in &plan.natives {
            cancel.check()?;
            let archive = stage.join(format!("native-jars/{}.jar", native.sha1));
            let output = stage.join("natives");
            tokio::task::spawn_blocking(move || extract_natives(&archive, &output)).await??;
        }
        write_json(&stage.join("version.json"), &document).await?;
        write_json(&stage.join("plan.json"), &plan).await?;
        cancel.check()?;
        let runtime = store.path(id)?.join("runtime");
        let backup = store
            .path(id)?
            .join(format!("runtime.previous-{}", uuid::Uuid::new_v4()));
        let prior = tokio::fs::symlink_metadata(&runtime).await.ok();
        if let Some(meta) = &prior {
            if !meta.is_dir() || meta.file_type().is_symlink() {
                bail!("Unsafe previous runtime directory")
            }
            tokio::fs::rename(&runtime, &backup).await?;
        }
        if let Err(error) = tokio::fs::rename(&stage, &runtime).await {
            if prior.is_some() {
                let _ = tokio::fs::rename(&backup, &runtime).await;
            }
            return Err(error.into());
        }
        // Retain the previous runtime for recovery; never remove worlds/saves.
        store.transition(id, &["installing"], "installed").await?;
        report(Progress {
            phase: "Minecraft files are ready".into(),
            verified_bytes: total,
            total_bytes: total,
        });
        Ok::<_, anyhow::Error>(())
    }
    .await;
    if result.is_err() {
        let _ = tokio::fs::remove_dir_all(&stage).await;
        let state = if cancel.check().is_err() {
            "cancelled"
        } else {
            "failed"
        };
        let _ = store.transition(id, &["installing"], state).await;
    }
    result
}

pub fn extract_natives(archive: &Path, output: &Path) -> Result<()> {
    use std::io::{Read, Write};
    let mut zip = zip::ZipArchive::new(std::fs::File::open(archive)?)?;
    if zip.len() > 4096 {
        bail!("Native archive file count exceeds limit")
    }
    let mut expanded = 0u64;
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i)?;
        if entry.is_dir() {
            continue;
        }
        metadata::relative_path(entry.name())?;
        if entry
            .unix_mode()
            .is_some_and(|mode| mode & 0o170000 == 0o120000)
        {
            bail!("Native archive symlinks are not allowed")
        }
        if entry.name().starts_with("META-INF/")
            || !entry.name().to_ascii_lowercase().ends_with(".dll")
        {
            continue;
        }
        expanded = expanded
            .checked_add(entry.size())
            .context("Native archive size overflow")?;
        if entry.size() > 64 * 1024 * 1024 || expanded > 256 * 1024 * 1024 {
            bail!("Native archive exceeds extraction limit")
        }
        let name = Path::new(entry.name())
            .file_name()
            .context("Invalid native library name")?
            .to_owned();
        let mut bytes = Vec::new();
        entry
            .by_ref()
            .take(64 * 1024 * 1024 + 1)
            .read_to_end(&mut bytes)?;
        if bytes.len() as u64 != entry.size() {
            bail!("Native archive entry size mismatch")
        }
        let path = output.join(name);
        if path.exists() {
            if std::fs::read(&path)? != bytes {
                bail!("Conflicting native library")
            }
        } else {
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(path)?;
            file.write_all(&bytes)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_extraction_rejects_archive_path_escape() {
        use std::io::Write;
        let root =
            std::env::temp_dir().join(format!("solara-native-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&root).unwrap();
        let archive = root.join("fixture.zip");
        let mut zip = zip::ZipWriter::new(std::fs::File::create(&archive).unwrap());
        zip.start_file("../escape.dll", zip::write::SimpleFileOptions::default())
            .unwrap();
        zip.write_all(b"non-executable archive fixture").unwrap();
        zip.finish().unwrap();
        assert!(extract_natives(&archive, &root).is_err());
        assert!(!root.parent().unwrap().join("escape.dll").exists());
        std::fs::remove_dir_all(root).unwrap();
    }
}
