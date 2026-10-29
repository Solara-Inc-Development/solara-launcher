//! Isolated per-instance world and configuration backups under `backups/`.
use super::{
    files::{self, is_link},
    instances::{instance_id, write_json, Instances},
    metadata::relative_path,
};
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    io::{Read, Write},
    path::{Path, PathBuf},
};

const MAX_BACKUPS: usize = 50;
const MAX_BACKUP_FILES: usize = 10_000;
const MAX_BACKUP_BYTES: u64 = 512 * 1024 * 1024;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupEntry {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub size: u64,
    pub file_count: u32,
    pub world_count: u32,
    pub sha256: String,
}

fn excluded_top_dir(segment: &str) -> bool {
    matches!(
        segment,
        "logs" | "crash-reports" | ".cache" | "mods" | "resourcepacks" | "shaderpacks"
    )
}

fn forbidden_backup_extension(path: &str) -> bool {
    Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|ext| {
            matches!(
                ext.to_ascii_lowercase().as_str(),
                "exe"
                    | "dll"
                    | "so"
                    | "dylib"
                    | "bat"
                    | "cmd"
                    | "ps1"
                    | "vbs"
                    | "vbe"
                    | "wsf"
                    | "wsh"
                    | "msi"
                    | "msp"
                    | "com"
                    | "scr"
                    | "pif"
                    | "hta"
                    | "cpl"
                    | "sys"
                    | "sh"
            )
        })
}

fn valid_label(label: Option<&str>, fallback_instance_name: &str) -> Result<String> {
    let default_name = format!("{fallback_instance_name} backup");
    let chosen = label
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(&default_name);
    if chosen.chars().count() > 80 || chosen.chars().any(char::is_control) {
        bail!("Choose a backup name up to 80 characters")
    }
    Ok(chosen.to_string())
}

async fn ensure_backups_dir(store: &Instances, id: &str, create: bool) -> Result<PathBuf> {
    store.get(id).await?;
    let dir = store.path(id)?.join("backups");
    if create {
        tokio::fs::create_dir_all(&dir).await?;
    } else if !tokio::fs::try_exists(&dir).await? {
        return Ok(dir);
    }
    let meta = tokio::fs::symlink_metadata(&dir).await?;
    if is_link(&meta) || !meta.is_dir() {
        bail!("Unsafe instance backups folder")
    }
    Ok(dir)
}

pub async fn list(store: &Instances, id: &str) -> Result<Vec<BackupEntry>> {
    let dir = ensure_backups_dir(store, id, false).await?;
    if !tokio::fs::try_exists(&dir).await? {
        return Ok(vec![]);
    }
    let mut iter = tokio::fs::read_dir(&dir).await?;
    let mut out = Vec::new();
    while let Some(entry) = iter.next_entry().await? {
        if out.len() >= 500 {
            bail!("Instance backup list limit reached")
        }
        let key = entry.file_name().to_string_lossy().to_string();
        if key.starts_with('.') || instance_id(&key).is_err() {
            continue;
        }
        let meta = tokio::fs::symlink_metadata(entry.path()).await?;
        if is_link(&meta) || !meta.is_dir() {
            bail!("Unsafe backup entry")
        }
        let record_path = entry.path().join("record.json");
        let record_meta = tokio::fs::symlink_metadata(&record_path).await?;
        if is_link(&record_meta) || !record_meta.is_file() || record_meta.len() > 16 * 1024 {
            bail!("Invalid backup record")
        }
        let archive_path = entry.path().join("archive.zip");
        let archive_meta = tokio::fs::symlink_metadata(&archive_path).await?;
        if is_link(&archive_meta) || !archive_meta.is_file() {
            bail!("Invalid backup archive")
        }
        let record: BackupEntry = serde_json::from_slice(&tokio::fs::read(&record_path).await?)?;
        if record.id != key
            || record.name.trim().is_empty()
            || record.name.chars().count() > 80
            || record.name.chars().any(char::is_control)
            || record.size != archive_meta.len()
            || record.sha256.len() != 64
            || !record.sha256.chars().all(|c| c.is_ascii_hexdigit())
        {
            bail!("Backup metadata mismatch")
        }
        out.push(record);
    }
    out.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(out)
}

pub async fn create(store: &Instances, id: &str, label: Option<&str>) -> Result<BackupEntry> {
    let instance = store.get(id).await?;
    if instance.state == "installing" {
        bail!("Wait for this instance to finish installing before creating a backup")
    }
    let name = valid_label(label, &instance.name)?;
    let existing = list(store, id).await?;
    if existing.len() >= MAX_BACKUPS {
        bail!("Backup limit reached for this instance. Delete an older backup first.")
    }
    let game_dir = store.game_dir(id).await?;
    if is_link(&tokio::fs::symlink_metadata(&game_dir).await?) {
        bail!("Unsafe instance game directory")
    }
    let backups_dir = ensure_backups_dir(store, id, true).await?;

    // Walk game_dir recursively, rejecting links and skipping transient/binary mod folders.
    let mut stack = vec![(String::new(), game_dir.clone())];
    let mut files_to_archive: Vec<(String, PathBuf, u64)> = Vec::new();
    let mut worlds = BTreeSet::new();
    let mut total_bytes = 0u64;

    while let Some((rel_dir, abs_dir)) = stack.pop() {
        let mut rd = tokio::fs::read_dir(&abs_dir).await?;
        while let Some(entry) = rd.next_entry().await? {
            let file_name = entry
                .file_name()
                .to_str()
                .context("A filename is not valid Unicode")?
                .to_string();
            if file_name.starts_with(".solara-")
                || file_name.ends_with(".tmp")
                || file_name == "session.lock"
            {
                continue;
            }
            let rel = if rel_dir.is_empty() {
                file_name.clone()
            } else {
                format!("{rel_dir}/{file_name}")
            };
            if rel_dir.is_empty() && excluded_top_dir(&file_name) {
                continue;
            }
            relative_path(&rel)?;
            let meta = tokio::fs::symlink_metadata(entry.path()).await?;
            if is_link(&meta) {
                bail!("Linked files and folders cannot be backed up")
            }
            if meta.is_dir() {
                stack.push((rel, entry.path()));
            } else if meta.is_file() {
                if forbidden_backup_extension(&rel) {
                    bail!("Executable files cannot be backed up in instance snapshots")
                }
                if files_to_archive.len() >= MAX_BACKUP_FILES {
                    bail!("This instance has too many files for a single backup")
                }
                total_bytes = total_bytes.saturating_add(meta.len());
                if total_bytes > MAX_BACKUP_BYTES {
                    bail!("Backup size exceeds the 512 MiB limit")
                }
                let mut parts = rel.split('/');
                if parts.next() == Some("saves") {
                    if let Some(world_name) = parts.next() {
                        if parts.next().is_some() {
                            worlds.insert(world_name.to_string());
                        }
                    }
                }
                files_to_archive.push((rel, entry.path(), meta.len()));
            }
        }
    }
    files_to_archive.sort_by(|a, b| a.0.cmp(&b.0));

    let backup_id = uuid::Uuid::new_v4().to_string();
    let stage_dir = backups_dir.join(format!(".create-stage-{}", uuid::Uuid::new_v4()));
    let entry_dir = backups_dir.join(&backup_id);
    tokio::fs::create_dir(&stage_dir).await?;
    let archive_path = stage_dir.join("archive.zip");
    let temp_zip = stage_dir.join(format!(".solara-backup-{}.tmp", uuid::Uuid::new_v4()));
    let file_count = files_to_archive.len() as u32;
    let world_count = worlds.len() as u32;

    let build_result = tokio::task::spawn_blocking({
        let temp_zip = temp_zip.clone();
        move || -> Result<(u64, String)> {
            let out_file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temp_zip)?;
            let mut writer = zip::ZipWriter::new(out_file);
            let options = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated);
            for (rel, src_path, expected_len) in files_to_archive {
                let meta = std::fs::symlink_metadata(&src_path)?;
                if is_link(&meta) || !meta.is_file() {
                    bail!("Source file changed or became unsafe during backup")
                }
                writer.start_file(&rel, options)?;
                let src = std::fs::File::open(&src_path)?;
                let mut bounded = src.take(expected_len.saturating_add(1));
                let copied = std::io::copy(&mut bounded, &mut writer)?;
                if copied > expected_len {
                    bail!("Source file grew while creating backup")
                }
            }
            let out_file = writer.finish()?;
            out_file.sync_all()?;
            drop(out_file);

            let bytes = std::fs::read(&temp_zip)?;
            let sha256 = format!("{:x}", Sha256::digest(&bytes));
            Ok((bytes.len() as u64, sha256))
        }
    })
    .await?;

    let (size, sha256) = match build_result {
        Ok(v) => v,
        Err(e) => {
            let _ = tokio::fs::remove_dir_all(&stage_dir).await;
            return Err(e);
        }
    };

    let finalize = async {
        tokio::fs::rename(&temp_zip, &archive_path).await?;
        let record = BackupEntry {
            id: backup_id.clone(),
            name,
            created_at: chrono::Utc::now().to_rfc3339(),
            size,
            file_count,
            world_count,
            sha256,
        };
        write_json(&stage_dir.join("record.json"), &record).await?;
        tokio::fs::rename(&stage_dir, &entry_dir).await?;
        Ok::<BackupEntry, anyhow::Error>(record)
    }
    .await;

    match finalize {
        Ok(_) => {
            let record: BackupEntry =
                serde_json::from_slice(&tokio::fs::read(entry_dir.join("record.json")).await?)?;
            Ok(record)
        }
        Err(e) => {
            let _ = tokio::fs::remove_dir_all(&stage_dir).await;
            let _ = tokio::fs::remove_dir_all(&entry_dir).await;
            Err(e)
        }
    }
}

async fn verify_no_links_recursive(dir: &Path) -> Result<()> {
    let mut stack = vec![dir.to_path_buf()];
    while let Some(current) = stack.pop() {
        let meta = tokio::fs::symlink_metadata(&current).await?;
        if is_link(&meta) {
            bail!("Linked files and folders cannot be managed here")
        }
        if meta.is_dir() {
            let mut rd = tokio::fs::read_dir(&current).await?;
            while let Some(entry) = rd.next_entry().await? {
                let child_meta = tokio::fs::symlink_metadata(entry.path()).await?;
                if is_link(&child_meta) {
                    bail!("Linked files and folders cannot be managed here")
                }
                if child_meta.is_dir() {
                    stack.push(entry.path());
                }
            }
        }
    }
    Ok(())
}

pub async fn restore(store: &Instances, id: &str, backup_id: &str) -> Result<BackupEntry> {
    instance_id(backup_id)?;
    let instance = store.get(id).await?;
    if instance.state == "installing" {
        bail!("Wait for this instance to finish installing before restoring a backup")
    }
    let record = list(store, id)
        .await?
        .into_iter()
        .find(|b| b.id == backup_id)
        .context("Backup was not found")?;
    let backups_dir = ensure_backups_dir(store, id, false).await?;
    let archive_path = backups_dir.join(backup_id).join("archive.zip");
    let stage_dir = backups_dir.join(format!(".restore-stage-{}", uuid::Uuid::new_v4()));
    let files_stage_dir = stage_dir.join("files");
    tokio::fs::create_dir_all(&files_stage_dir).await?;

    let stage_result = tokio::task::spawn_blocking({
        let archive_path = archive_path.clone();
        let files_stage_dir = files_stage_dir.clone();
        let expected_sha256 = record.sha256.clone();
        let expected_size = record.size;
        move || -> Result<(Vec<String>, BTreeSet<String>)> {
            let meta = std::fs::symlink_metadata(&archive_path)?;
            if is_link(&meta) || !meta.is_file() || meta.len() != expected_size {
                bail!("Backup archive failed integrity verification")
            }
            let raw = std::fs::read(&archive_path)?;
            let actual_sha256 = format!("{:x}", Sha256::digest(&raw));
            if actual_sha256 != expected_sha256 {
                bail!("Backup archive failed integrity verification")
            }
            let reader = std::io::Cursor::new(raw);
            let mut zip = zip::ZipArchive::new(reader)?;
            if zip.len() > MAX_BACKUP_FILES {
                bail!("Backup archive contains too many files")
            }
            let mut extracted_files = Vec::new();
            let mut restored_worlds = BTreeSet::new();
            let mut total_uncompressed = 0u64;

            for i in 0..zip.len() {
                let mut entry = zip.by_index(i)?;
                if entry.is_symlink()
                    || entry
                        .unix_mode()
                        .is_some_and(|mode| mode & 0o170000 == 0o120000)
                {
                    bail!("Backup archive contains an unsupported link entry")
                }
                let raw_name = entry.name().replace('\\', "/");
                let trimmed = raw_name.trim_end_matches('/');
                if trimmed.is_empty() {
                    continue;
                }
                relative_path(trimmed)?;
                let first_seg = trimmed.split('/').next().unwrap_or("");
                if excluded_top_dir(first_seg) {
                    bail!("Backup archive contains a restricted folder path")
                }
                if entry.is_dir() {
                    continue;
                }
                if forbidden_backup_extension(trimmed) {
                    bail!("Backup archive contains a forbidden executable file")
                }
                let size = entry.size();
                total_uncompressed = total_uncompressed.saturating_add(size);
                if size > MAX_BACKUP_BYTES || total_uncompressed > MAX_BACKUP_BYTES {
                    bail!("Backup archive exceeds the 512 MiB restore limit")
                }
                let mut parts = trimmed.split('/');
                if parts.next() == Some("saves") {
                    if let Some(world_name) = parts.next() {
                        if parts.next().is_some() {
                            restored_worlds.insert(world_name.to_string());
                        }
                    }
                }
                let staged_path = files_stage_dir.join(trimmed);
                if let Some(parent) = staged_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                let mut out = std::fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&staged_path)?;
                let mut limited = (&mut entry).take(size.saturating_add(1));
                let written = std::io::copy(&mut limited, &mut out)?;
                if written != size {
                    bail!("Backup archive entry size mismatch")
                }
                out.write_all(&[])?;
                out.sync_all()?;
                extracted_files.push(trimmed.to_string());
            }
            Ok((extracted_files, restored_worlds))
        }
    })
    .await?;

    let (extracted_files, restored_worlds) = match stage_result {
        Ok(v) => v,
        Err(e) => {
            let _ = tokio::fs::remove_dir_all(&stage_dir).await;
            return Err(e);
        }
    };

    let apply_result = async {
        let game_dir = store.game_dir(id).await?;
        if is_link(&tokio::fs::symlink_metadata(&game_dir).await?) {
            bail!("Unsafe instance game directory")
        }
        let saves_dir = game_dir.join("saves");
        if tokio::fs::try_exists(&saves_dir).await? {
            let saves_meta = tokio::fs::symlink_metadata(&saves_dir).await?;
            if is_link(&saves_meta) || !saves_meta.is_dir() {
                bail!("Unsafe instance saves folder")
            }
            // Pre-validate every restored world folder for links before moving or deleting anything.
            for world in &restored_worlds {
                let world_dir = saves_dir.join(world);
                if tokio::fs::try_exists(&world_dir).await? {
                    verify_no_links_recursive(&world_dir).await?;
                }
            }
        }
        // Pre-validate all non-world target paths before touching saves/.
        for rel in &extracted_files {
            let is_restored_world_file = {
                let mut parts = rel.split('/');
                parts.next() == Some("saves")
                    && parts
                        .next()
                        .is_some_and(|w| parts.next().is_some() && restored_worlds.contains(w))
            };
            if !is_restored_world_file {
                files::resolve_create_parents(store, id, rel).await?;
            }
        }

        // Move existing folders for restored worlds into rollback_saves so post-backup
        // file/folder changes inside those worlds do not block restore and can be rolled back on error.
        let rollback_saves = stage_dir.join("rollback-saves");
        tokio::fs::create_dir_all(&rollback_saves).await?;
        let mut moved_worlds: Vec<String> = Vec::new();

        let place_result = async {
            if tokio::fs::try_exists(&saves_dir).await? {
                for world in &restored_worlds {
                    let world_dir = saves_dir.join(world);
                    if tokio::fs::try_exists(&world_dir).await? {
                        tokio::fs::rename(&world_dir, rollback_saves.join(world)).await?;
                        moved_worlds.push(world.clone());
                    }
                }
            }
            for rel in &extracted_files {
                let target = files::resolve_create_parents(store, id, rel).await?;
                let staged = files_stage_dir.join(rel);
                let temp =
                    target.with_file_name(format!(".solara-restore-{}.tmp", uuid::Uuid::new_v4()));
                if let Err(e) = async {
                    tokio::fs::copy(&staged, &temp).await?;
                    tokio::fs::rename(&temp, &target).await?;
                    Ok::<_, anyhow::Error>(())
                }
                .await
                {
                    let _ = tokio::fs::remove_file(&temp).await;
                    return Err(e);
                }
            }
            Ok::<_, anyhow::Error>(())
        }
        .await;

        if let Err(e) = place_result {
            for world in &moved_worlds {
                let world_dir = saves_dir.join(world);
                let _ = tokio::fs::remove_dir_all(&world_dir).await;
                let _ = tokio::fs::rename(rollback_saves.join(world), &world_dir).await;
            }
            return Err(e);
        }
        Ok::<_, anyhow::Error>(())
    }
    .await;

    let _ = tokio::fs::remove_dir_all(&stage_dir).await;
    apply_result?;
    Ok(record)
}

pub async fn delete(store: &Instances, id: &str, backup_id: &str) -> Result<()> {
    instance_id(backup_id)?;
    let instance = store.get(id).await?;
    if instance.state == "installing" {
        bail!("Wait for this instance to finish installing before deleting a backup")
    }
    let backups_dir = ensure_backups_dir(store, id, false).await?;
    let entry_dir = backups_dir.join(backup_id);
    let meta = tokio::fs::symlink_metadata(&entry_dir)
        .await
        .context("Backup was not found")?;
    if is_link(&meta) || !meta.is_dir() {
        bail!("Unsafe backup entry")
    }
    tokio::fs::remove_dir_all(&entry_dir).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn backup_create_list_restore_and_delete_preserve_worlds_and_reject_tampering() {
        let root =
            std::env::temp_dir().join(format!("solara-backups-test-{}", uuid::Uuid::new_v4()));
        let store = Instances::new(root.clone());
        let inst = store
            .create(
                "Backup World".into(),
                "1.21.1".into(),
                "fabric".into(),
                Some("0.16.9".into()),
                2048,
                "auto".into(),
            )
            .await
            .unwrap();
        store
            .transition(&inst.id, &["created"], "installed")
            .await
            .unwrap();
        let game = store.game_dir(&inst.id).await.unwrap();
        tokio::fs::write(game.join("options.txt"), b"fov:70")
            .await
            .unwrap();
        tokio::fs::create_dir_all(game.join("config"))
            .await
            .unwrap();
        tokio::fs::write(game.join("config/sodium.json"), b"{\"quality\":\"high\"}")
            .await
            .unwrap();
        tokio::fs::create_dir_all(game.join("saves/SurvivalWorld"))
            .await
            .unwrap();
        tokio::fs::write(game.join("saves/SurvivalWorld/level.dat"), b"world-v1")
            .await
            .unwrap();
        // Empty folder under saves/ should not inflate world_count when it has no files.
        tokio::fs::create_dir_all(game.join("saves/EmptyFolder"))
            .await
            .unwrap();

        let created = create(&store, &inst.id, Some("Before Nether"))
            .await
            .unwrap();
        assert_eq!(created.name, "Before Nether");
        assert_eq!(created.file_count, 3);
        assert_eq!(created.world_count, 1);

        let listed = list(&store, &inst.id).await.unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].id, created.id);

        // Mutate existing world/config, replace level.dat file with a directory inside SurvivalWorld,
        // and add a second independent world + extra file in SurvivalWorld.
        tokio::fs::write(game.join("options.txt"), b"fov:110")
            .await
            .unwrap();
        tokio::fs::remove_file(game.join("saves/SurvivalWorld/level.dat"))
            .await
            .unwrap();
        tokio::fs::create_dir_all(game.join("saves/SurvivalWorld/level.dat"))
            .await
            .unwrap();
        tokio::fs::write(
            game.join("saves/SurvivalWorld/level.dat/conflict.txt"),
            b"folder-where-file-was",
        )
        .await
        .unwrap();
        tokio::fs::write(game.join("saves/SurvivalWorld/post-backup.dat"), b"stale")
            .await
            .unwrap();
        tokio::fs::create_dir_all(game.join("saves/CreativeWorld"))
            .await
            .unwrap();
        tokio::fs::write(game.join("saves/CreativeWorld/level.dat"), b"keep-me")
            .await
            .unwrap();

        // Reject restore and delete while instance is installing.
        store
            .transition(&inst.id, &["installed"], "installing")
            .await
            .unwrap();
        assert!(restore(&store, &inst.id, &created.id).await.is_err());
        assert!(delete(&store, &inst.id, &created.id).await.is_err());
        store
            .transition(&inst.id, &["installing"], "installed")
            .await
            .unwrap();

        // Restore snapshot and verify SurvivalWorld + options.txt revert while CreativeWorld is preserved.
        restore(&store, &inst.id, &created.id).await.unwrap();
        assert_eq!(
            tokio::fs::read(game.join("options.txt")).await.unwrap(),
            b"fov:70"
        );
        assert_eq!(
            tokio::fs::read(game.join("saves/SurvivalWorld/level.dat"))
                .await
                .unwrap(),
            b"world-v1"
        );
        assert!(
            !tokio::fs::try_exists(game.join("saves/SurvivalWorld/post-backup.dat"))
                .await
                .unwrap()
        );
        assert_eq!(
            tokio::fs::read(game.join("saves/CreativeWorld/level.dat"))
                .await
                .unwrap(),
            b"keep-me"
        );

        // Tamper with archive.zip and verify restore fails closed without clobbering files.
        let archive_path = store
            .path(&inst.id)
            .unwrap()
            .join("backups")
            .join(&created.id)
            .join("archive.zip");
        let mut bytes = tokio::fs::read(&archive_path).await.unwrap();
        bytes[0] ^= 0xff;
        tokio::fs::write(&archive_path, &bytes).await.unwrap();
        assert!(restore(&store, &inst.id, &created.id).await.is_err());

        // Delete backup and verify list is empty.
        delete(&store, &inst.id, &created.id).await.unwrap();
        assert!(list(&store, &inst.id).await.unwrap().is_empty());

        tokio::fs::remove_dir_all(root).await.unwrap();
    }
}
