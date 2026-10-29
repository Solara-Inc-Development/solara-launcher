//! Verified official-file cache. No Minecraft/Solara credentials are attached.
use super::metadata::Artifact;
use anyhow::{bail, Context, Result};
use std::{
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::Notify,
};

#[derive(Clone, Default)]
pub struct Cancellation {
    flag: Arc<AtomicBool>,
    changed: Arc<Notify>,
}
impl Cancellation {
    pub fn cancel(&self) {
        self.flag.store(true, Ordering::SeqCst);
        self.changed.notify_waiters();
    }
    pub fn check(&self) -> Result<()> {
        if self.flag.load(Ordering::SeqCst) {
            bail!("Download cancelled")
        }
        Ok(())
    }
    async fn cancelled(&self) {
        loop {
            let changed = self.changed.notified();
            if self.flag.load(Ordering::SeqCst) {
                return;
            }
            changed.await;
        }
    }
}

pub async fn verified_file(path: &Path, artifact: &Artifact) -> Result<bool> {
    artifact.validate()?;
    let meta = match tokio::fs::symlink_metadata(path).await {
        Ok(v) => v,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(e) => return Err(e.into()),
    };
    if !meta.is_file() || meta.file_type().is_symlink() || meta.len() != artifact.size {
        return Ok(false);
    }
    let mut file = tokio::fs::File::open(path).await?;
    let mut hash = sha1_smol::Sha1::new();
    let mut buffer = [0u8; 65536];
    loop {
        let read = file.read(&mut buffer).await?;
        if read == 0 {
            break;
        }
        hash.update(&buffer[..read]);
    }
    Ok(hash
        .digest()
        .to_string()
        .eq_ignore_ascii_case(&artifact.sha1))
}

pub struct FileCache {
    root: PathBuf,
    client: reqwest::Client,
}
impl FileCache {
    pub fn new(root: PathBuf) -> Result<Self> {
        Ok(Self {
            root,
            client: reqwest::Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .timeout(std::time::Duration::from_secs(300))
                .user_agent("SolaraLauncher/independent-core")
                .build()?,
        })
    }
    pub async fn obtain(
        &self,
        artifact: &Artifact,
        cancel: &Cancellation,
        mut progress: impl FnMut(u64, u64),
    ) -> Result<PathBuf> {
        artifact.validate()?;
        cancel.check()?;
        let key = artifact.sha1.to_ascii_lowercase();
        let dir = self.root.join(&key[..2]);
        tokio::fs::create_dir_all(&dir).await?;
        let target = dir.join(&key);
        if verified_file(&target, artifact).await? {
            progress(artifact.size, artifact.size);
            return Ok(target);
        }
        let temp = dir.join(format!(".{}.part", uuid::Uuid::new_v4()));
        let result=async {
            let mut response=tokio::select! {
                _=cancel.cancelled()=>bail!("Download cancelled"),
                r=self.client.get(super::metadata::official_url(&artifact.url)?).send()=>r.context("Minecraft download unavailable")?,
            };
            if !response.status().is_success() {bail!("Minecraft download unavailable")}
            if response.content_length().is_some_and(|size|size!=artifact.size) {bail!("Minecraft download size mismatch")}
            let mut file=tokio::fs::OpenOptions::new().write(true).create_new(true).open(&temp).await?;
            let mut size=0u64;let mut hash=sha1_smol::Sha1::new();
            progress(0,artifact.size);
            loop {
                cancel.check()?;
                let chunk=tokio::select! {_=cancel.cancelled()=>bail!("Download cancelled"),r=response.chunk()=>r?};
                let Some(chunk)=chunk else {break};
                size=size.checked_add(chunk.len() as u64).context("Minecraft file too large")?;
                if size>artifact.size {bail!("Minecraft download exceeds expected size")}
                hash.update(&chunk);file.write_all(&chunk).await?;progress(size,artifact.size);
            }
            if size!=artifact.size || !hash.digest().to_string().eq_ignore_ascii_case(&artifact.sha1) {bail!("Minecraft download integrity check failed")}
            cancel.check()?;file.sync_all().await?;drop(file);
            // A concurrent valid download may already own this content address.
            if verified_file(&target,artifact).await? {tokio::fs::remove_file(&temp).await?;} else {tokio::fs::rename(&temp,&target).await?;}
            Ok(target)
        }.await;
        if result.is_err() {
            let _ = tokio::fs::remove_file(&temp).await;
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn cache_bytes_require_exact_size_and_checksum() {
        let root = std::env::temp_dir().join(format!("solara-cache-test-{}", uuid::Uuid::new_v4()));
        tokio::fs::create_dir(&root).await.unwrap();
        let path = root.join("fixture.bin");
        let bytes = b"official-file-fixture";
        let artifact = Artifact {
            url: "https://libraries.minecraft.net/fixture.bin".into(),
            sha1: sha1_smol::Sha1::from(bytes).digest().to_string(),
            size: bytes.len() as u64,
            path: None,
        };
        assert!(!verified_file(&path, &artifact).await.unwrap());
        tokio::fs::write(&path, bytes).await.unwrap();
        assert!(verified_file(&path, &artifact).await.unwrap());
        tokio::fs::write(&path, b"tampered-file-fixture")
            .await
            .unwrap();
        assert!(!verified_file(&path, &artifact).await.unwrap());
        tokio::fs::remove_dir_all(root).await.unwrap();
    }
    #[tokio::test]
    async fn cancellation_is_sticky_and_wakes_waiters() {
        let cancel = Cancellation::default();
        assert!(cancel.check().is_ok());
        let other = cancel.clone();
        let waiting = tokio::spawn(async move { other.cancelled().await });
        cancel.cancel();
        tokio::time::timeout(std::time::Duration::from_secs(1), waiting)
            .await
            .unwrap()
            .unwrap();
        assert!(cancel.check().is_err());
    }
}
