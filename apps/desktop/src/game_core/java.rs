//! Managed Temurin runtimes. Downloads stay on the user's PC, outside game saves.
use super::{download::Cancellation, instances::write_json, metadata::relative_path};
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::Mutex,
};

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Runtime {
    pub major: u32,
    pub version: String,
    pub archive_sha256: String,
    pub java_sha256: String,
    pub jvm_sha256: String,
}
pub struct Runtimes {
    root: PathBuf,
    install: Mutex<()>,
}
fn supported(major: u32) -> Result<()> {
    if !(8..=64).contains(&major) {
        bail!("This Java version is not supported")
    }
    Ok(())
}
fn package_url(url: &reqwest::Url, major: u32) -> bool {
    url.scheme() == "https"
        && url.username().is_empty()
        && url.password().is_none()
        && url.port().is_none()
        && url.fragment().is_none()
        && url.host_str() == Some("github.com")
        && url.path().starts_with(&format!(
            "/adoptium/temurin{major}-binaries/releases/download/"
        ))
        && url.path().ends_with(".zip")
}
fn redirect_url(url: &reqwest::Url, major: u32) -> bool {
    package_url(url, major)
        || (url.scheme() == "https"
            && url.username().is_empty()
            && url.password().is_none()
            && url.port().is_none()
            && url.fragment().is_none()
            && matches!(
                url.host_str(),
                Some("release-assets.githubusercontent.com" | "objects.githubusercontent.com")
            )
            && url.path().starts_with("/github-production-release-asset/"))
}
#[derive(Clone)]
struct Package {
    version: String,
    url: reqwest::Url,
    size: u64,
    sha256: String,
}
fn select_package(raw: &Value, major: u32) -> Result<Package> {
    supported(major)?;
    let rows = raw.as_array().context("Invalid Java release list")?;
    let row = rows
        .iter()
        .find(|r| {
            r["vendor"] == "eclipse"
                && r["binary"]["os"] == "windows"
                && r["binary"]["architecture"] == "x64"
                && r["binary"]["jvm_impl"] == "hotspot"
        })
        .context("No matching Windows Java runtime is available")?;
    let p = &row["binary"]["package"];
    let url = reqwest::Url::parse(
        p["link"]
            .as_str()
            .context("Java download address missing")?,
    )?;
    let size = p["size"]
        .as_u64()
        .filter(|n| *n > 0 && *n <= 512 * 1024 * 1024)
        .context("Invalid Java package size")?;
    let hash = p["checksum"].as_str().context("Java checksum missing")?;
    if !package_url(&url, major) || hash.len() != 64 || !hash.bytes().all(|b| b.is_ascii_hexdigit())
    {
        bail!("Java release failed origin or integrity checks")
    }
    let version = row["version"]["semver"]
        .as_str()
        .filter(|v| !v.is_empty() && v.len() < 100)
        .context("Invalid Java release version")?;
    Ok(Package {
        version: version.into(),
        url,
        size,
        sha256: hash.to_ascii_lowercase(),
    })
}
async fn digest(path: &Path) -> Result<String> {
    let meta = tokio::fs::symlink_metadata(path).await?;
    if !meta.is_file() || meta.file_type().is_symlink() || meta.len() > 512 * 1024 * 1024 {
        bail!("Invalid Java runtime file")
    }
    let mut file = tokio::fs::File::open(path).await?;
    let mut hash = Sha256::new();
    let mut buf = [0; 65536];
    loop {
        let n = file.read(&mut buf).await?;
        if n == 0 {
            break;
        }
        hash.update(&buf[..n]);
    }
    Ok(format!("{:x}", hash.finalize()))
}
impl Runtimes {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            install: Mutex::new(()),
        }
    }
    async fn cached(&self, major: u32) -> Result<Option<PathBuf>> {
        let dir = self.root.join(format!("java-{major}"));
        let meta = match tokio::fs::symlink_metadata(&dir).await {
            Ok(m) => m,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(e.into()),
        };
        if !meta.is_dir() || meta.file_type().is_symlink() {
            bail!("Unsafe Java runtime directory")
        }
        let path = dir.join("solara-runtime.json");
        let meta = match tokio::fs::symlink_metadata(&path).await {
            Ok(m) => m,
            Err(_) => return Ok(None),
        };
        if !meta.is_file() || meta.file_type().is_symlink() || meta.len() > 8192 {
            return Ok(None);
        }
        let record: Runtime = match serde_json::from_slice(&tokio::fs::read(path).await?) {
            Ok(r) => r,
            Err(_) => return Ok(None),
        };
        let java = dir.join("bin/java.exe");
        let jvm = dir.join("bin/server/jvm.dll");
        if record.major != major
            || digest(&java).await.ok().as_deref() != Some(&record.java_sha256)
            || digest(&jvm).await.ok().as_deref() != Some(&record.jvm_sha256)
        {
            return Ok(None);
        }
        Ok(Some(java))
    }
    pub async fn ensure(
        &self,
        major: u32,
        cancel: &Cancellation,
        mut report: impl FnMut(&str, u64, u64) + Send,
    ) -> Result<PathBuf> {
        supported(major)?;
        cancel.check()?;
        let _guard = tokio::select! {_ = cancel.cancelled()=>bail!("Java setup cancelled"),guard=self.install.lock()=>guard};
        if let Some(path) = self.cached(major).await? {
            report("Java is ready", 0, 0);
            return Ok(path);
        }
        report("Finding the right Java version", 0, 0);
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(std::time::Duration::from_secs(30))
            .user_agent("SolaraLauncher/independent-core")
            .build()?;
        let mut package = None;
        for image in ["jre", "jdk"] {
            let url=format!("https://api.adoptium.net/v3/assets/latest/{major}/hotspot?architecture=x64&image_type={image}&os=windows");
            let response = tokio::select! {_ = cancel.cancelled()=>bail!("Java setup cancelled"),r=client.get(url).send()=>r.map_err(|_|anyhow::anyhow!("Java release service is unavailable"))?};
            if response.status() == reqwest::StatusCode::NOT_FOUND {
                continue;
            }
            if !response.status().is_success() {
                bail!("Java release service is unavailable")
            }
            let mut response = response;
            let mut bytes = Vec::new();
            while let Some(chunk) = response
                .chunk()
                .await
                .map_err(|_| anyhow::anyhow!("Java release metadata could not be read"))?
            {
                cancel.check()?;
                if bytes.len() + chunk.len() > 2 * 1024 * 1024 {
                    bail!("Java metadata exceeds limit")
                }
                bytes.extend_from_slice(&chunk);
            }
            package = Some(select_package(&serde_json::from_slice(&bytes)?, major)?);
            break;
        }
        let package = package.context(
            "This Java runtime is unavailable. Choose an installed Java executable in Settings.",
        )?;
        tokio::fs::create_dir_all(&self.root).await?;
        if tokio::fs::symlink_metadata(&self.root)
            .await?
            .file_type()
            .is_symlink()
        {
            bail!("Unsafe Java runtime root")
        }
        let archive = self.root.join(format!("{}.zip", package.sha256));
        if digest(&archive).await.ok().as_deref() != Some(&package.sha256) {
            let partial = self.root.join(format!("{}.part", uuid::Uuid::new_v4()));
            let download=async {
                let client=reqwest::Client::builder().user_agent("SolaraLauncher/independent-core")
                    .timeout(std::time::Duration::from_secs(900)).connect_timeout(std::time::Duration::from_secs(15))
                    .redirect(reqwest::redirect::Policy::custom(move|attempt|{
                        if attempt.previous().len()<5 && redirect_url(attempt.url(),major){attempt.follow()}else{attempt.stop()}
                    })).build()?;
                let mut response=tokio::select!{_ = cancel.cancelled()=>bail!("Java setup cancelled"),r=client.get(package.url.clone()).send()=>r.map_err(|_|anyhow::anyhow!("Java download could not start"))?};
                if !response.status().is_success()||response.content_length().is_some_and(|n|n!=package.size){bail!("Java download failed size or status checks")}
                let mut file=tokio::fs::OpenOptions::new().write(true).create_new(true).open(&partial).await?;
                let mut size=0;let mut hash=Sha256::new();
                loop {
                    let chunk=tokio::select!{_ = cancel.cancelled()=>bail!("Java setup cancelled"),r=response.chunk()=>r.map_err(|_|anyhow::anyhow!("Java download was interrupted"))?};
                    let Some(chunk)=chunk else{break};size+=chunk.len()as u64;if size>package.size{bail!("Java download exceeds expected size")}
                    hash.update(&chunk);file.write_all(&chunk).await?;report("Downloading Java",size,package.size);
                }
                if size!=package.size||format!("{:x}",hash.finalize())!=package.sha256{bail!("Java download checksum did not match")}
                cancel.check()?;file.sync_all().await?;drop(file);tokio::fs::rename(&partial,&archive).await?;Ok::<_,anyhow::Error>(())
            }.await;
            if download.is_err() {
                let _ = tokio::fs::remove_file(&partial).await;
            }
            download?;
        }
        let stage = self
            .root
            .join(format!("java-{major}.install-{}", uuid::Uuid::new_v4()));
        let result = async {
            tokio::fs::create_dir(&stage).await?;
            report("Unpacking Java", 0, 0);
            let zip = archive.clone();
            let dest = stage.clone();
            let flag = cancel.clone();
            tokio::task::spawn_blocking(move || extract(&zip, &dest, &flag)).await??;
            let java = stage.join("bin/java.exe");
            let jvm = stage.join("bin/server/jvm.dll");
            let record = Runtime {
                major,
                version: package.version,
                archive_sha256: package.sha256,
                java_sha256: digest(&java).await?,
                jvm_sha256: digest(&jvm).await?,
            };
            if !tokio::fs::try_exists(stage.join("release")).await? {
                bail!("Java package release details are missing")
            }
            write_json(&stage.join("solara-runtime.json"), &record).await?;
            cancel.check()?;
            let target = self.root.join(format!("java-{major}"));
            if tokio::fs::symlink_metadata(&target).await.is_ok() {
                tokio::fs::rename(
                    &target,
                    self.root
                        .join(format!("java-{major}.previous-{}", uuid::Uuid::new_v4())),
                )
                .await?
            }
            tokio::fs::rename(&stage, &target).await?;
            report("Java is ready", package.size, package.size);
            Ok(target.join("bin/java.exe"))
        }
        .await;
        if result.is_err() {
            let _ = tokio::fs::remove_dir_all(&stage).await;
        }
        result
    }
}
fn extract(archive: &Path, output: &Path, cancel: &Cancellation) -> Result<()> {
    use std::io::{Read, Write};
    let mut zip = zip::ZipArchive::new(std::fs::File::open(archive)?)?;
    if zip.len() > 20000 {
        bail!("Java package file count exceeds limit")
    }
    let mut total = 0u64;
    let mut top = None::<String>;
    for index in 0..zip.len() {
        cancel.check()?;
        let mut entry = zip.by_index(index)?;
        let name = entry.name().trim_end_matches('/');
        relative_path(name)?;
        if entry.unix_mode().is_some_and(|m| m & 0o170000 == 0o120000) {
            bail!("Java package links are not allowed")
        }
        let (prefix, relative) = name.split_once('/').unwrap_or((name, ""));
        if top.as_deref().is_some_and(|v| v != prefix) {
            bail!("Java package has conflicting root directories")
        }
        top = Some(prefix.into());
        if relative.is_empty() {
            continue;
        }
        relative_path(relative)?;
        let dest = output.join(relative);
        if entry.is_dir() {
            std::fs::create_dir_all(dest)?;
            continue;
        }
        total = total
            .checked_add(entry.size())
            .context("Java package size overflow")?;
        if entry.size() > 256 * 1024 * 1024 || total > 1024 * 1024 * 1024 {
            bail!("Java package extraction exceeds limit")
        }
        std::fs::create_dir_all(dest.parent().context("Invalid Java package path")?)?;
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(dest)?;
        let mut size = 0;
        let mut bytes = [0; 65536];
        loop {
            cancel.check()?;
            let n = entry.read(&mut bytes)?;
            if n == 0 {
                break;
            }
            size += n as u64;
            if size > 256 * 1024 * 1024 {
                bail!("Java package entry exceeds limit")
            }
            file.write_all(&bytes[..n])?;
        }
        if size != entry.size() {
            bail!("Java package entry size mismatch")
        }
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn downloads_reject_untrusted_runtime_origins() {
        assert!(package_url(
            &reqwest::Url::parse(
                "https://github.com/adoptium/temurin21-binaries/releases/download/test/runtime.zip"
            )
            .unwrap(),
            21
        ));
        assert!(!package_url(
            &reqwest::Url::parse(
                "https://github.com/other/project/releases/download/test/runtime.zip"
            )
            .unwrap(),
            21
        ));
        assert!(!redirect_url(
            &reqwest::Url::parse("https://evil.invalid/runtime.zip").unwrap(),
            21
        ));
        assert!(!redirect_url(
            &reqwest::Url::parse("https://release-assets.githubusercontent.com/unrelated/file")
                .unwrap(),
            21
        ));
    }
    #[test]
    fn extraction_rejects_paths_outside_runtime() {
        use std::io::Write;
        let dir = std::env::temp_dir().join(format!("solara-java-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&dir).unwrap();
        let archive = dir.join("bad.zip");
        let mut zip = zip::ZipWriter::new(std::fs::File::create(&archive).unwrap());
        zip.start_file(
            "runtime/../../escape.exe",
            zip::write::SimpleFileOptions::default(),
        )
        .unwrap();
        zip.write_all(b"fixture").unwrap();
        zip.finish().unwrap();
        assert!(extract(&archive, &dir, &Cancellation::default()).is_err());
        assert!(!dir.join("escape.exe").exists());
        std::fs::remove_dir_all(dir).unwrap();
    }
}
