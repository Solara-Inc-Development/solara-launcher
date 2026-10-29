//! Instance-local files, optimistic text edits, and recoverable trash. No shell.
use super::{
    instances::{instance_id, write_json, Instances},
    metadata::relative_path,
};
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use tokio::sync::Mutex;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub name: String,
    pub path: String,
    pub directory: bool,
    pub size: u64,
    pub modified_at: Option<u64>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextFile {
    pub path: String,
    pub content: String,
    pub sha256: String,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrashEntry {
    pub id: String,
    pub original_path: String,
    pub removed_at: String,
    pub directory: bool,
}
#[derive(Default)]
pub struct Files {
    writes: Mutex<()>,
}
fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
pub fn is_link(meta: &std::fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        meta.file_attributes() & 0x400 != 0
    }
    #[cfg(not(windows))]
    {
        meta.file_type().is_symlink()
    }
}
fn link(meta: &std::fs::Metadata) -> bool {
    is_link(meta)
}
fn editable(path: &str) -> bool {
    Path::new(path)
        .extension()
        .and_then(|v| v.to_str())
        .is_some_and(|v| {
            matches!(
                v.to_ascii_lowercase().as_str(),
                "txt"
                    | "json"
                    | "toml"
                    | "yaml"
                    | "yml"
                    | "properties"
                    | "log"
                    | "cfg"
                    | "ini"
                    | "conf"
                    | "md"
                    | "csv"
                    | "xml"
                    | "mcmeta"
            )
        })
}
/// Every existing component is checked; a final missing leaf is allowed only for creation.
pub async fn resolve(
    store: &Instances,
    id: &str,
    relative: &str,
    new_leaf: bool,
) -> Result<PathBuf> {
    let root = store.game_dir(id).await?;
    if link(&tokio::fs::symlink_metadata(&root).await?) {
        bail!("Unsafe instance game directory")
    }
    if relative.is_empty() {
        return Ok(root);
    }
    relative_path(relative)?;
    let mut path = root;
    let parts: Vec<_> = relative.split('/').collect();
    for (index, part) in parts.iter().enumerate() {
        path.push(part);
        match tokio::fs::symlink_metadata(&path).await {
            Ok(meta) => {
                if link(&meta) {
                    bail!("Linked files and folders cannot be managed here")
                }
                if index + 1 < parts.len() && !meta.is_dir() {
                    bail!("A parent path is not a folder")
                }
            }
            Err(e)
                if new_leaf
                    && index + 1 == parts.len()
                    && e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e).context("Instance file or folder is unavailable"),
        }
    }
    Ok(path)
}
/// Resolves a target file path under game/, safely creating missing parent directories while rejecting links.
pub async fn resolve_create_parents(
    store: &Instances,
    id: &str,
    relative: &str,
) -> Result<PathBuf> {
    let root = store.game_dir(id).await?;
    if link(&tokio::fs::symlink_metadata(&root).await?) {
        bail!("Unsafe instance game directory")
    }
    leaf_required(relative)?;
    let mut path = root;
    let parts: Vec<_> = relative.split('/').collect();
    for (index, part) in parts.iter().enumerate() {
        path.push(part);
        if index + 1 < parts.len() {
            match tokio::fs::symlink_metadata(&path).await {
                Ok(meta) => {
                    if link(&meta) {
                        bail!("Linked files and folders cannot be managed here")
                    }
                    if !meta.is_dir() {
                        bail!("A parent path is not a folder")
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                    tokio::fs::create_dir(&path).await?;
                }
                Err(e) => return Err(e).context("Instance file or folder is unavailable"),
            }
        } else {
            match tokio::fs::symlink_metadata(&path).await {
                Ok(meta) => {
                    if link(&meta) {
                        bail!("Linked files and folders cannot be managed here")
                    }
                    if !meta.is_file() {
                        bail!("A folder already exists at the target file path")
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => return Err(e).context("Instance file or folder is unavailable"),
            }
        }
    }
    Ok(path)
}
fn leaf_required(path: &str) -> Result<()> {
    if path.is_empty() {
        bail!("Choose a file or folder inside the instance")
    }
    relative_path(path)
}
impl Files {
    pub async fn list(&self, store: &Instances, id: &str, relative: &str) -> Result<Vec<Entry>> {
        let path = resolve(store, id, relative, false).await?;
        let mut entries = tokio::fs::read_dir(path).await?;
        let mut out = Vec::new();
        while let Some(entry) = entries.next_entry().await? {
            if out.len() >= 5000 {
                bail!("This folder has too many files to show at once")
            }
            let meta = tokio::fs::symlink_metadata(entry.path()).await?;
            if link(&meta) {
                continue;
            }
            let name = entry
                .file_name()
                .to_str()
                .context("A filename is not valid Unicode")?
                .to_string();
            let path = if relative.is_empty() {
                name.clone()
            } else {
                format!("{relative}/{name}")
            };
            relative_path(&path)?;
            out.push(Entry {
                name,
                path,
                directory: meta.is_dir(),
                size: if meta.is_dir() { 0 } else { meta.len() },
                modified_at: meta
                    .modified()
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|t| t.as_secs()),
            });
        }
        out.sort_by(|a, b| {
            b.directory
                .cmp(&a.directory)
                .then(a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        });
        Ok(out)
    }
    pub async fn read(&self, store: &Instances, id: &str, relative: &str) -> Result<TextFile> {
        leaf_required(relative)?;
        if !editable(relative) {
            bail!("This file type cannot be edited as text")
        }
        let path = resolve(store, id, relative, false).await?;
        let meta = tokio::fs::metadata(&path).await?;
        if !meta.is_file() || meta.len() > 2 * 1024 * 1024 {
            bail!("Choose a text file smaller than 2 MiB")
        }
        let bytes = tokio::fs::read(path).await?;
        if bytes.len() > 2 * 1024 * 1024 {
            bail!("Text file exceeds limit")
        }
        let sha256 = hash(&bytes);
        let content = String::from_utf8(bytes).context("This file is not UTF-8 text")?;
        Ok(TextFile {
            path: relative.into(),
            content,
            sha256,
        })
    }
    pub async fn write(
        &self,
        store: &Instances,
        id: &str,
        relative: &str,
        content: &str,
        expected: Option<&str>,
    ) -> Result<String> {
        leaf_required(relative)?;
        if !editable(relative) || content.len() > 2 * 1024 * 1024 || content.contains('\0') {
            bail!("Choose a supported text file smaller than 2 MiB")
        }
        let _lock = self.writes.lock().await;
        let path = resolve(store, id, relative, true).await?;
        match tokio::fs::symlink_metadata(&path).await {
            Ok(meta) => {
                if !meta.is_file() || meta.len() > 2 * 1024 * 1024 {
                    bail!("Invalid existing text file")
                }
                let actual = hash(&tokio::fs::read(&path).await?);
                if expected != Some(actual.as_str()) {
                    bail!("This file changed since you opened it. Reload it before saving.")
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                if expected.is_some() {
                    bail!("This file was moved or removed. Reload the folder.")
                }
            }
            Err(e) => return Err(e.into()),
        }
        let temp = path.with_file_name(format!(".solara-edit-{}.tmp", uuid::Uuid::new_v4()));
        let result = async {
            use tokio::io::AsyncWriteExt;
            let mut file = tokio::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temp)
                .await?;
            file.write_all(content.as_bytes()).await?;
            file.sync_all().await?;
            drop(file);
            tokio::fs::rename(&temp, &path).await?;
            Ok::<_, anyhow::Error>(())
        }
        .await;
        if result.is_err() {
            let _ = tokio::fs::remove_file(temp).await;
        }
        result?;
        Ok(hash(content.as_bytes()))
    }
    pub async fn mkdir(&self, store: &Instances, id: &str, relative: &str) -> Result<()> {
        leaf_required(relative)?;
        let _lock = self.writes.lock().await;
        let path = resolve(store, id, relative, true).await?;
        tokio::fs::create_dir(path).await?;
        Ok(())
    }
    pub async fn rename(&self, store: &Instances, id: &str, from: &str, to: &str) -> Result<()> {
        leaf_required(from)?;
        leaf_required(to)?;
        let _lock = self.writes.lock().await;
        let source = resolve(store, id, from, false).await?;
        let target = resolve(store, id, to, true).await?;
        if tokio::fs::symlink_metadata(&target).await.is_ok() {
            bail!("A file or folder with that name already exists")
        }
        tokio::fs::rename(source, target).await?;
        Ok(())
    }
    pub async fn trash(&self, store: &Instances, id: &str, relative: &str) -> Result<String> {
        leaf_required(relative)?;
        let _lock = self.writes.lock().await;
        let source = resolve(store, id, relative, false).await?;
        let directory = tokio::fs::metadata(&source).await?.is_dir();
        let key = uuid::Uuid::new_v4().to_string();
        let dir = store.path(id)?.join("trash");
        tokio::fs::create_dir_all(&dir).await?;
        if link(&tokio::fs::symlink_metadata(&dir).await?) {
            bail!("Unsafe instance trash folder")
        }
        let dir = dir.join(&key);
        tokio::fs::create_dir(&dir).await?;
        write_json(
            &dir.join("record.json"),
            &TrashEntry {
                id: key.clone(),
                original_path: relative.into(),
                removed_at: chrono::Utc::now().to_rfc3339(),
                directory,
            },
        )
        .await?;
        tokio::fs::rename(source, dir.join("payload")).await?;
        Ok(key)
    }
    pub async fn trash_list(&self, store: &Instances, id: &str) -> Result<Vec<TrashEntry>> {
        store.get(id).await?;
        let dir = store.path(id)?.join("trash");
        if !tokio::fs::try_exists(&dir).await? {
            return Ok(vec![]);
        }
        if link(&tokio::fs::symlink_metadata(&dir).await?) {
            bail!("Unsafe instance trash folder")
        }
        let mut iter = tokio::fs::read_dir(dir).await?;
        let mut entries = Vec::new();
        while let Some(entry) = iter.next_entry().await? {
            if entries.len() >= 1000 {
                bail!("Instance trash limit reached")
            }
            let key = entry.file_name().to_string_lossy().to_string();
            if instance_id(&key).is_err() {
                continue;
            }
            let meta = tokio::fs::symlink_metadata(entry.path()).await?;
            if link(&meta) || !meta.is_dir() {
                bail!("Unsafe trash entry")
            }
            let path = entry.path().join("record.json");
            let meta = tokio::fs::symlink_metadata(&path).await?;
            if link(&meta) || !meta.is_file() || meta.len() > 8192 {
                bail!("Invalid trash record")
            }
            let record: TrashEntry = serde_json::from_slice(&tokio::fs::read(path).await?)?;
            if record.id != key {
                bail!("Trash identity mismatch")
            }
            leaf_required(&record.original_path)?;
            if tokio::fs::try_exists(entry.path().join("payload")).await? {
                entries.push(record)
            }
        }
        entries.sort_by(|a, b| b.removed_at.cmp(&a.removed_at));
        Ok(entries)
    }
    pub async fn restore(&self, store: &Instances, id: &str, key: &str) -> Result<()> {
        instance_id(key)?;
        let _lock = self.writes.lock().await;
        let record = self
            .trash_list(store, id)
            .await?
            .into_iter()
            .find(|r| r.id == key)
            .context("Trash item was not found")?;
        let target = resolve(store, id, &record.original_path, true).await?;
        if tokio::fs::symlink_metadata(&target).await.is_ok() {
            bail!("Restore would replace an existing file. Rename it first.")
        }
        let source = store.path(id)?.join("trash").join(key).join("payload");
        if link(&tokio::fs::symlink_metadata(&source).await?) {
            bail!("Linked trash files cannot be restored")
        }
        tokio::fs::rename(&source, target).await?;
        tokio::fs::remove_file(source.parent().unwrap().join("record.json")).await?;
        tokio::fs::remove_dir(source.parent().unwrap()).await?;
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn editing_conflicts_and_trash_preserve_actual_user_bytes() {
        let root = std::env::temp_dir().join(format!("solara-files-test-{}", uuid::Uuid::new_v4()));
        let store = Instances::new(root.clone());
        let fs = Files::default();
        let i = store
            .create(
                "Fixture".into(),
                "1.21.1".into(),
                "vanilla".into(),
                None,
                2048,
                "auto".into(),
            )
            .await
            .unwrap();
        let first = fs
            .write(&store, &i.id, "options.txt", "original", None)
            .await
            .unwrap();
        assert!(fs
            .write(&store, &i.id, "options.txt", "stale", Some("wrong"))
            .await
            .is_err());
        fs.write(&store, &i.id, "options.txt", "saved", Some(&first))
            .await
            .unwrap();
        let key = fs.trash(&store, &i.id, "options.txt").await.unwrap();
        assert!(fs.read(&store, &i.id, "options.txt").await.is_err());
        assert_eq!(fs.trash_list(&store, &i.id).await.unwrap().len(), 1);
        fs.restore(&store, &i.id, &key).await.unwrap();
        assert_eq!(
            fs.read(&store, &i.id, "options.txt").await.unwrap().content,
            "saved"
        );
        assert!(fs
            .write(&store, &i.id, "../escape.txt", "escape", None)
            .await
            .is_err());
        assert!(fs
            .write(&store, &i.id, "evil.exe", "fixture", None)
            .await
            .is_err());
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(
                std::env::temp_dir(),
                store.game_dir(&i.id).await.unwrap().join("linked"),
            )
            .unwrap();
            assert!(resolve(&store, &i.id, "linked/escape.txt", true)
                .await
                .is_err());
        }
        tokio::fs::remove_dir_all(root).await.unwrap();
    }
}
