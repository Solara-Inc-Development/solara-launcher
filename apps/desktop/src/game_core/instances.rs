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
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub group: Option<String>,
    #[serde(default)]
    pub pinned: bool,
    #[serde(default)]
    pub favourite: bool,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub last_played: Option<String>,
    #[serde(default)]
    pub playtime_seconds: u64,
    #[serde(default)]
    pub jvm_args: Option<String>,
    #[serde(default)]
    pub resolution: Option<[u32; 2]>,
    #[serde(default)]
    pub fullscreen: Option<bool>,
    #[serde(default)]
    pub pre_launch: Option<String>,
    #[serde(default)]
    pub post_exit: Option<String>,
    #[serde(default)]
    pub update_channel: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
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
        if let Some(icon) = &self.icon {
            if icon.len() > 128 || icon.chars().any(char::is_control) {
                bail!("Invalid instance icon identifier");
            }
        }
        if let Some(group) = &self.group {
            if group.chars().count() > 60 || group.chars().any(char::is_control) {
                bail!("Invalid instance group name");
            }
        }
        if let Some(jvm) = &self.jvm_args {
            if jvm.chars().count() > 2000 || jvm.contains('\0') || jvm.chars().any(|c| c.is_control() && c != ' ' && c != '\t') {
                bail!("Invalid JVM arguments");
            }
        }
        if let Some([w, h]) = self.resolution {
            if !(320..=7680).contains(&w) || !(240..=4320).contains(&h) {
                bail!("Invalid game window resolution");
            }
        }
        if let Some(hook) = &self.pre_launch {
            if hook.chars().count() > 500 || hook.contains('\0') || hook.chars().any(char::is_control) {
                bail!("Invalid pre-launch hook");
            }
        }
        if let Some(hook) = &self.post_exit {
            if hook.chars().count() > 500 || hook.contains('\0') || hook.chars().any(char::is_control) {
                bail!("Invalid post-exit hook");
            }
        }
        if let Some(ch) = &self.update_channel {
            if !matches!(ch.as_str(), "release" | "beta" | "alpha") {
                bail!("Invalid update channel");
            }
        }
        if let Some(notes) = &self.notes {
            if notes.chars().count() > 2000 || notes.contains('\0') {
                bail!("Instance notes exceed length limit");
            }
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
            icon: None,
            group: None,
            pinned: false,
            favourite: false,
            created_at: Some(chrono::Utc::now().to_rfc3339()),
            last_played: None,
            playtime_seconds: 0,
            jvm_args: None,
            resolution: None,
            fullscreen: None,
            pre_launch: None,
            post_exit: None,
            update_channel: Some("release".into()),
            notes: None,
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
    pub async fn save_icon(&self, id: &str, data_url_or_base64: Option<&str>) -> Result<Option<String>> {
        let _guard = self.writes.lock().await;
        let dir = self.safe_dir(id).await?;
        let icon_path = dir.join("icon.png");
        let Some(raw_input) = data_url_or_base64.map(str::trim).filter(|s| !s.is_empty()) else {
            let _ = tokio::fs::remove_file(&icon_path).await;
            return Ok(None);
        };
        if raw_input.starts_with("preset:") || raw_input.starts_with("builtin:") {
            let _ = tokio::fs::remove_file(&icon_path).await;
            return Ok(Some(raw_input.chars().take(120).collect()));
        }
        let b64 = raw_input
            .strip_prefix("data:image/png;base64,")
            .or_else(|| raw_input.strip_prefix("data:image/jpeg;base64,"))
            .or_else(|| raw_input.strip_prefix("data:image/webp;base64,"))
            .unwrap_or(raw_input);
        use base64::Engine as _;
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(b64)
            .context("Could not decode custom instance icon")?;
        if bytes.is_empty() || bytes.len() > 2 * 1024 * 1024 {
            bail!("Custom instance icon must be smaller than 2 MiB");
        }
        let temp = dir.join(format!(".icon-{}.tmp", uuid::Uuid::new_v4()));
        tokio::fs::write(&temp, &bytes).await?;
        if let Err(e) = tokio::fs::rename(&temp, &icon_path).await {
            let _ = tokio::fs::remove_file(&temp).await;
            return Err(e.into());
        }
        Ok(Some("custom".into()))
    }
    pub async fn read_icon(&self, id: &str) -> Result<Option<String>> {
        let dir = self.safe_dir(id).await?;
        let icon_path = dir.join("icon.png");
        let meta = match tokio::fs::symlink_metadata(&icon_path).await {
            Ok(m) => m,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(e.into()),
        };
        if !meta.is_file() || meta.file_type().is_symlink() || meta.len() > 2 * 1024 * 1024 {
            return Ok(None);
        }
        let bytes = tokio::fs::read(&icon_path).await?;
        let mime = if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
            "image/png"
        } else if bytes.starts_with(b"\xff\xd8\xff") {
            "image/jpeg"
        } else if bytes.starts_with(b"RIFF") && bytes.get(8..12) == Some(b"WEBP") {
            "image/webp"
        } else {
            return Ok(None);
        };
        use base64::Engine as _;
        let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
        Ok(Some(format!("data:{mime};base64,{b64}")))
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
        let safe_name: String = if target_name.chars().count() > 80 {
            target_name.chars().take(80).collect()
        } else {
            target_name.to_string()
        };
        let new_id = uuid::Uuid::new_v4().to_string();
        let record = Instance {
            schema_version: source.schema_version,
            id: new_id.clone(),
            name: safe_name,
            game_version: source.game_version.clone(),
            loader: source.loader.clone(),
            loader_version: source.loader_version.clone(),
            memory: source.memory,
            java_path: source.java_path.clone(),
            state: if source.state == "installed" { "installed".into() } else { "created".into() },
            icon: source.icon.clone(),
            group: source.group.clone(),
            pinned: false,
            favourite: false,
            created_at: Some(chrono::Utc::now().to_rfc3339()),
            last_played: None,
            playtime_seconds: 0,
            jvm_args: source.jvm_args.clone(),
            resolution: source.resolution,
            fullscreen: source.fullscreen,
            pre_launch: source.pre_launch.clone(),
            post_exit: source.post_exit.clone(),
            update_channel: source.update_channel.clone(),
            notes: source.notes.clone(),
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
            let src_runtime = src_dir.join("runtime");
            let dest_runtime = dest_dir.join("runtime");
            let src_icon = src_dir.join("icon.png");
            let dest_icon = dest_dir.join("icon.png");

            // Copy files in blocking pool so heavy I/O remains fully responsive and does not stall async runtime
            tokio::task::spawn_blocking(move || -> Result<()> {
                if src_game.is_dir() {
                    copy_dir_recursive(&src_game, &dest_game)?;
                }
                if src_runtime.is_dir() {
                    copy_dir_recursive(&src_runtime, &dest_runtime)?;
                }
                if src_icon.is_file() {
                    let _ = std::fs::copy(&src_icon, &dest_icon);
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
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let file_type = match entry.file_type() {
            Ok(ft) => ft,
            Err(_) => continue,
        };
        // Ignore symlinks to prevent path escape
        if file_type.is_symlink() {
            continue;
        }
        let file_name = entry.file_name().to_string_lossy().to_string();
        if file_name.ends_with(".lock")
            || file_name.ends_with(".lck")
            || file_name == "session.lock"
        {
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

        // Long duplicate names exceeding 80 characters are safely clamped and pass validation
        let long_name = "A".repeat(120);
        let long_copy = store.duplicate(&original.id, &long_name).await.unwrap();
        assert_eq!(long_copy.name.chars().count(), 80);
        assert_eq!(long_copy.name, "A".repeat(80));
        store.delete(&long_copy.id).await.unwrap();

        store.delete(&original.id).await.unwrap();
        // Copy still exists after original deleted
        assert_eq!(store.get(&copy.id).await.unwrap().name, "Original World (Copy)");
        store.delete(&copy.id).await.unwrap();
        let _ = tokio::fs::remove_dir_all(root).await;
    }
}
