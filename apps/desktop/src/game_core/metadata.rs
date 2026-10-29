use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const MANIFEST: &str = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";
const METADATA_LIMIT: usize = 8 * 1024 * 1024;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VersionEntry {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub url: String,
    pub sha1: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Artifact {
    pub url: String,
    pub sha1: String,
    pub size: u64,
    #[serde(default)]
    pub path: Option<String>,
}

pub fn version_id(value: &str) -> Result<()> {
    if value.is_empty()
        || value.len() > 100
        || !value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._- ".contains(&b))
        || value.trim() != value
        || value == "."
        || value == ".."
    {
        bail!("Invalid Minecraft version");
    }
    Ok(())
}

pub fn official_url(value: &str) -> Result<reqwest::Url> {
    let url = reqwest::Url::parse(value)?;
    if url.scheme() != "https"
        || !url.username().is_empty()
        || url.password().is_some()
        || url.port().is_some()
        || url.fragment().is_some()
        || !matches!(
            url.host_str(),
            Some(
                "piston-meta.mojang.com"
                    | "piston-data.mojang.com"
                    | "launcher.mojang.com"
                    | "launchermeta.mojang.com"
                    | "libraries.minecraft.net"
                    | "resources.download.minecraft.net"
                    | "meta.fabricmc.net"
                    | "maven.fabricmc.net"
                    | "meta.quiltmc.org"
                    | "maven.quiltmc.org"
                    | "cdn.modrinth.com"
            )
        )
    {
        bail!("Unsupported Minecraft download origin");
    }
    Ok(url)
}

pub fn relative_path(value: &str) -> Result<()> {
    if value.len() > 1024
        || value.is_empty()
        || value.contains('\\')
        || value.contains(':')
        || value.contains('\0')
        || value.split('/').any(|part| {
            let stem = part.split('.').next().unwrap_or("").to_ascii_uppercase();
            part.is_empty()
                || part == "."
                || part == ".."
                || part.ends_with([' ', '.'])
                || part
                    .chars()
                    .any(|c| c.is_control() || "<>\"|?*".contains(c))
                || matches!(
                    stem.as_str(),
                    "CON"
                        | "PRN"
                        | "AUX"
                        | "NUL"
                        | "COM1"
                        | "COM2"
                        | "COM3"
                        | "COM4"
                        | "COM5"
                        | "COM6"
                        | "COM7"
                        | "COM8"
                        | "COM9"
                        | "LPT1"
                        | "LPT2"
                        | "LPT3"
                        | "LPT4"
                        | "LPT5"
                        | "LPT6"
                        | "LPT7"
                        | "LPT8"
                        | "LPT9"
                )
        })
    {
        bail!("Unsafe Minecraft file path")
    }
    Ok(())
}

impl Artifact {
    pub fn validate(&self) -> Result<()> {
        official_url(&self.url)?;
        if self.sha1.len() != 40
            || !self.sha1.bytes().all(|b| b.is_ascii_hexdigit())
            || self.size == 0
            || self.size > 1024 * 1024 * 1024
        {
            bail!("Invalid Minecraft file integrity metadata");
        }
        if let Some(path) = &self.path {
            relative_path(path)?
        }
        Ok(())
    }
}

pub(super) async fn fetch_metadata(url: &str) -> Result<Vec<u8>> {
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(30))
        .user_agent("SolaraLauncher/independent-core")
        .build()?;
    let mut response = client.get(official_url(url)?).send().await?;
    if !response.status().is_success() {
        bail!("Minecraft metadata is unavailable")
    }
    if response
        .content_length()
        .is_some_and(|n| n > METADATA_LIMIT as u64)
    {
        bail!("Minecraft metadata exceeds limit")
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await? {
        if bytes.len() + chunk.len() > METADATA_LIMIT {
            bail!("Minecraft metadata exceeds limit")
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}

static CACHED_VERSIONS: std::sync::Mutex<Option<(std::time::Instant, Vec<VersionEntry>)>> =
    std::sync::Mutex::new(None);

pub async fn versions() -> Result<Vec<VersionEntry>> {
    if let Ok(guard) = CACHED_VERSIONS.lock() {
        if let Some((at, ref list)) = *guard {
            if at.elapsed() < std::time::Duration::from_secs(300) {
                return Ok(list.clone());
            }
        }
    }
    match fetch_metadata(MANIFEST).await {
        Ok(bytes) => {
            let data: Value = serde_json::from_slice(&bytes)?;
            let raw = data["versions"]
                .as_array()
                .context("Invalid Minecraft version list")?;
            if raw.len() > 10000 {
                bail!("Minecraft version list exceeds limit")
            }
            let mut result = Vec::with_capacity(raw.len());
            for value in raw {
                let entry: VersionEntry = serde_json::from_value(value.clone())?;
                version_id(&entry.id)?;
                official_url(&entry.url)?;
                if entry.sha1.len() != 40 || !entry.sha1.bytes().all(|b| b.is_ascii_hexdigit()) {
                    bail!("Invalid Minecraft metadata checksum")
                }
                result.push(entry);
            }
            if let Ok(mut guard) = CACHED_VERSIONS.lock() {
                *guard = Some((std::time::Instant::now(), result.clone()));
            }
            Ok(result)
        }
        Err(e) => {
            if let Ok(guard) = CACHED_VERSIONS.lock() {
                if let Some((_, ref list)) = *guard {
                    return Ok(list.clone());
                }
            }
            Err(e)
        }
    }
}

pub fn verify_version(entry: &VersionEntry, bytes: &[u8]) -> Result<Value> {
    version_id(&entry.id)?;
    official_url(&entry.url)?;
    if !sha1_smol::Sha1::from(bytes)
        .digest()
        .to_string()
        .eq_ignore_ascii_case(&entry.sha1)
    {
        bail!("Minecraft version metadata checksum failed")
    }
    let value: Value = serde_json::from_slice(bytes)?;
    if value["id"].as_str() != Some(entry.id.as_str()) {
        bail!("Minecraft version identity mismatch")
    }
    let artifact: Artifact = serde_json::from_value(value["downloads"]["client"].clone())?;
    artifact.validate()?;
    if value["mainClass"].as_str().is_none_or(|v| {
        v.is_empty()
            || v.len() > 200
            || !v
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"._$".contains(&b))
    }) {
        bail!("Invalid Minecraft main class")
    }
    Ok(value)
}

pub async fn version(entry: &VersionEntry) -> Result<Value> {
    verify_version(entry, &fetch_metadata(&entry.url).await?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn untrusted_origins_and_windows_path_escapes_are_rejected() {
        for value in [
            "http://libraries.minecraft.net/a",
            "https://libraries.minecraft.net.evil.test/a",
            "https://user:password@libraries.minecraft.net/a",
            "https://libraries.minecraft.net:8443/a",
        ] {
            assert!(official_url(value).is_err());
        }
        for value in [
            "../file.jar",
            "/absolute.jar",
            "C:/escape.jar",
            "a\\escape.jar",
            "CON.txt",
            "x/../file.jar",
            "x/file. ",
            "x//file.jar",
        ] {
            assert!(relative_path(value).is_err(), "{value}");
        }
        assert!(relative_path("org/example/lib/1.0/lib-1.0.jar").is_ok());
    }
    #[test]
    fn version_identity_and_integrity_are_bound_before_planning_install() {
        let data = json!({"id":"1.21.1","mainClass":"net.minecraft.client.main.Main","downloads":{"client":{"url":"https://piston-data.mojang.com/client.jar","sha1":"a".repeat(40),"size":1000}}});
        let bytes = serde_json::to_vec(&data).unwrap();
        let mut entry = VersionEntry {
            id: "1.21.1".into(),
            kind: "release".into(),
            url: "https://piston-meta.mojang.com/version.json".into(),
            sha1: sha1_smol::Sha1::from(&bytes).digest().to_string(),
        };
        assert!(verify_version(&entry, &bytes).is_ok());
        assert!(verify_version(&entry, b"modified").is_err());
        entry.id = "1.20.1".into();
        assert!(verify_version(&entry, &bytes).is_err());
    }
}
