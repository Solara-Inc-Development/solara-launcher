//! Solara-owned instance records and directories, separate from the old engine.
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::sync::Mutex;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Instance {
    pub schema_version: u8,
    pub id: String,
    pub name: String,
    pub game_version: String,
    pub loader: String,
    pub loader_version: Option<String>,
    pub memory: u32,
    pub java_path: String,
    pub state: String,
}
impl Instance {
    pub fn validate(&self) -> Result<()> {
        instance_id(&self.id)?;
        super::metadata::version_id(&self.game_version)?;
        if self.schema_version != 1
            || self.name.trim().is_empty()
            || self.name.chars().count() > 80
            || self.name.chars().any(char::is_control)
            || !(512..=65536).contains(&self.memory)
            || self.java_path.trim().is_empty()
            || self.java_path.contains('\0')
            || !matches!(
                self.loader.as_str(),
                "vanilla" | "fabric" | "quilt" | "forge" | "neoforge"
            )
            || !matches!(
                self.state.as_str(),
                "created" | "installing" | "installed" | "failed" | "cancelled"
            )
        {
            bail!("Invalid local instance record");
        }
        if let Some(version) = &self.loader_version {
            super::loaders::loader_id(version)?;
        }
        Ok(())
    }
}
pub fn instance_id(value: &str) -> Result<()> {
    if uuid::Uuid::parse_str(value)?.to_string() != value {
        bail!("Invalid local instance identifier")
    }
    Ok(())
}

pub struct Instances {
    root: PathBuf,
    writes: Mutex<()>,
}
impl Instances {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            writes: Mutex::new(()),
        }
    }
    pub fn path(&self, id: &str) -> Result<PathBuf> {
        instance_id(id)?;
        Ok(self.root.join(id))
    }
    async fn safe_dir(&self, id: &str) -> Result<PathBuf> {
        let path = self.path(id)?;
        let meta = tokio::fs::symlink_metadata(&path)
            .await
            .context("Local instance directory is missing")?;
        if !meta.is_dir() || meta.file_type().is_symlink() {
            bail!("Unsafe local instance directory")
        }
        Ok(path)
    }
    pub async fn get(&self, id: &str) -> Result<Instance> {
        let dir = self.safe_dir(id).await?;
        let path = dir.join("instance.json");
        let meta = tokio::fs::symlink_metadata(&path).await?;
        if !meta.is_file() || meta.file_type().is_symlink() || meta.len() > 64 * 1024 {
            bail!("Invalid local instance metadata")
        }
        let record: Instance = serde_json::from_slice(&tokio::fs::read(path).await?)?;
        record.validate()?;
        if record.id != id {
            bail!("Local instance identity mismatch")
        }
        Ok(record)
    }
    pub async fn list(&self) -> Result<Vec<Instance>> {
        tokio::fs::create_dir_all(&self.root).await?;
        let mut entries = tokio::fs::read_dir(&self.root).await?;
        let mut out = Vec::new();
        while let Some(entry) = entries.next_entry().await? {
            if out.len() >= 1000 {
                bail!("Local instance limit reached")
            }
            let id = entry.file_name().to_string_lossy().to_string();
            if instance_id(&id).is_ok() {
                out.push(self.get(&id).await?);
            }
        }
        out.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        Ok(out)
    }
    pub async fn create(
        &self,
        name: String,
        game_version: String,
        loader: String,
        loader_version: Option<String>,
        memory: u32,
        java_path: String,
    ) -> Result<Instance> {
        let _guard = self.writes.lock().await;
        let record = Instance {
            schema_version: 1,
            id: uuid::Uuid::new_v4().to_string(),
            name: name.trim().into(),
            game_version,
            loader,
            loader_version,
            memory,
            java_path,
            state: "created".into(),
        };
        record.validate()?;
        tokio::fs::create_dir_all(&self.root).await?;
        let dir = self.path(&record.id)?;
        tokio::fs::create_dir(&dir).await?;
        let result = async {
            for name in ["game", "logs", "natives"] {
                tokio::fs::create_dir(dir.join(name)).await?;
            }
            write_json(&dir.join("instance.json"), &record).await
        }
        .await;
        if let Err(error) = result {
            let _ = tokio::fs::remove_dir_all(&dir).await;
            return Err(error);
        }
        Ok(record)
    }
    pub async fn change(&self, id: &str, edit: impl FnOnce(&mut Instance)) -> Result<Instance> {
        let _guard = self.writes.lock().await;
        let mut record = self.get(id).await?;
        edit(&mut record);
        record.validate()?;
        if record.id != id {
            bail!("Instance identifier cannot change")
        }
        write_json(&self.safe_dir(id).await?.join("instance.json"), &record).await?;
        Ok(record)
    }
    pub async fn transition(&self, id: &str, from: &[&str], to: &str) -> Result<Instance> {
        let _guard = self.writes.lock().await;
        let mut record = self.get(id).await?;
        if !from.contains(&record.state.as_str()) {
            bail!("Instance is already busy or cannot enter this state")
        }
        record.state = to.into();
        record.validate()?;
        write_json(&self.safe_dir(id).await?.join("instance.json"), &record).await?;
        Ok(record)
    }
    pub async fn delete(&self, id: &str) -> Result<()> {
        let _guard = self.writes.lock().await;
        let dir = self.safe_dir(id).await?;
        tokio::fs::remove_dir_all(&dir).await?;
        Ok(())
    }
    pub async fn duplicate(&self, source_id: &str, target_name: &str) -> Result<Instance> {
        let _guard = self.writes.lock().await;
        let source = self.get(source_id).await?;
        let src_dir = self.safe_dir(source_id).await?;
        let target_name = target_name.trim();
        if target_name.is_empty() {
            bail!("Duplicate instance name cannot be empty");
        }
        let new_id = uuid::Uuid::new_v4().to_string();
        let record = Instance {
            schema_version: source.schema_version,
            id: new_id.clone(),
            name: target_name.to_string(),
            game_version: source.game_version.clone(),
            loader: source.loader.clone(),
            loader_version: source.loader_version.clone(),
            memory: source.memory,
            java_path: source.java_path.clone(),
            state: if source.state == "installed" { "installed".into() } else { "created".into() },
        };
        record.validate()?;
        tokio::fs::create_dir_all(&self.root).await?;
        let dest_dir = self.path(&new_id)?;
        tokio::fs::create_dir(&dest_dir).await?;

        let copy_result = async {
            for name in ["logs", "natives"] {
                tokio::fs::create_dir(dest_dir.join(name)).await?;
            }
            tokio::fs::create_dir(dest_dir.join("game")).await?;

            let src_game = src_dir.join("game");
            let dest_game = dest_dir.join("game");

            // Copy files in blocking pool so heavy I/O remains fully responsive and does not stall async runtime
            tokio::task::spawn_blocking(move || -> Result<()> {
                if src_game.is_dir() {
                    copy_dir_recursive(&src_game, &dest_game)?;
                }
                Ok(())
            })
            .await??;

            write_json(&dest_dir.join("instance.json"), &record).await
        }
        .await;

        if let Err(err) = copy_result {
            let _ = tokio::fs::remove_dir_all(&dest_dir).await;
            return Err(err);
        }

        Ok(record)
    }
    pub async fn game_dir(&self, id: &str) -> Result<PathBuf> {
        let dir = self.safe_dir(id).await?.join("game");
        let meta = tokio::fs::symlink_metadata(&dir).await?;
        if !meta.is_dir() || meta.file_type().is_symlink() {
            bail!("Unsafe instance game directory")
        }
        Ok(dir)
    }
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    if !dst.exists() {
        std::fs::create_dir_all(dst)?;
    }
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        // Ignore symlinks to prevent path escape
        if file_type.is_symlink() {
            continue;
        }
        let file_name = entry.file_name().to_string_lossy().to_string();
        if file_name.ends_with(".lock") || file_name == "session.lock" {
            continue;
        }
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else if file_type.is_file() {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

pub async fn write_json(path: &Path, value: &impl Serialize) -> Result<()> {
    let temp = path.with_extension(format!("{}.tmp", uuid::Uuid::new_v4()));
    let result = async {
        use tokio::io::AsyncWriteExt;
        let mut file = tokio::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)
            .await?;
        file.write_all(&serde_json::to_vec_pretty(value)?).await?;
        file.sync_all().await?;
        drop(file);
        tokio::fs::rename(&temp, path).await?;
        Ok::<_, anyhow::Error>(())
    }
    .await;
    if result.is_err() {
        let _ = tokio::fs::remove_file(temp).await;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn instance_changes_preserve_world_files_and_reject_identity_escape() {
        let root =
            std::env::temp_dir().join(format!("solara-instance-test-{}", uuid::Uuid::new_v4()));
        let store = Instances::new(root.clone());
        let first = store
            .create(
                "My world".into(),
                "1.21.1".into(),
                "vanilla".into(),
                None,
                2048,
                "java".into(),
            )
            .await
            .unwrap();
        let game = store.game_dir(&first.id).await.unwrap();
        tokio::fs::write(game.join("world-fixture.bin"), b"preserve-user-data")
            .await
            .unwrap();
        store
            .change(&first.id, |i| {
                i.name = "New name".into();
                i.memory = 4096
            })
            .await
            .unwrap();
        assert_eq!(store.list().await.unwrap()[0].name, "New name");
        assert_eq!(
            tokio::fs::read(game.join("world-fixture.bin"))
                .await
                .unwrap(),
            b"preserve-user-data"
        );
        assert!(store.path("../escape").is_err());
        assert!(store
            .change(&first.id, |i| i.id = uuid::Uuid::new_v4().to_string())
            .await
            .is_err());
        assert_eq!(store.get(&first.id).await.unwrap().id, first.id);
        store.delete(&first.id).await.unwrap();
        assert!(store.get(&first.id).await.is_err());
        tokio::fs::remove_dir_all(root).await.unwrap();
    }

    #[tokio::test]
    async fn instance_duplication_copies_files_and_preserves_independence() {
        let root =
            std::env::temp_dir().join(format!("solara-duplicate-test-{}", uuid::Uuid::new_v4()));
        let store = Instances::new(root.clone());
        let original = store
            .create(
                "Original World".into(),
                "1.21.1".into(),
                "fabric".into(),
                Some("0.16.0".into()),
                4096,
                "auto".into(),
            )
            .await
            .unwrap();
        let game = store.game_dir(&original.id).await.unwrap();
        let saves_dir = game.join("saves").join("TestWorld");
        tokio::fs::create_dir_all(&saves_dir).await.unwrap();
        tokio::fs::write(saves_dir.join("level.dat"), b"level-data-v1")
            .await
            .unwrap();
        tokio::fs::write(game.join("options.txt"), b"gamma:1.0")
            .await
            .unwrap();

        // Duplicate
        let copy = store
            .duplicate(&original.id, "Original World (Copy)")
            .await
            .unwrap();
        assert_ne!(copy.id, original.id);
        assert_eq!(copy.name, "Original World (Copy)");
        assert_eq!(copy.game_version, "1.21.1");
        assert_eq!(copy.loader, "fabric");
        assert_eq!(copy.loader_version.as_deref(), Some("0.16.0"));
        assert_eq!(copy.memory, 4096);

        let copy_game = store.game_dir(&copy.id).await.unwrap();
        assert_eq!(
            tokio::fs::read(copy_game.join("saves").join("TestWorld").join("level.dat"))
                .await
                .unwrap(),
            b"level-data-v1"
        );
        assert_eq!(
            tokio::fs::read(copy_game.join("options.txt"))
                .await
                .unwrap(),
            b"gamma:1.0"
        );

        // Verify independent mutation
        tokio::fs::write(copy_game.join("options.txt"), b"gamma:2.0")
            .await
            .unwrap();
        assert_eq!(
            tokio::fs::read(game.join("options.txt")).await.unwrap(),
            b"gamma:1.0"
        );

        // Empty duplicate name is rejected
        assert!(store.duplicate(&original.id, "   ").await.is_err());

        store.delete(&original.id).await.unwrap();
        // Copy still exists after original deleted
        assert_eq!(store.get(&copy.id).await.unwrap().name, "Original World (Copy)");
        store.delete(&copy.id).await.unwrap();
        let _ = tokio::fs::remove_dir_all(root).await;
    }
}
