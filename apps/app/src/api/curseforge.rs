use crate::api::Result;
use anyhow::{Context, bail};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
use theseus::data::{InstanceLink, ModLoader};
use theseus::install::{InstallJobStatus, create_instance, get_job};
use theseus::{LoadingBarType, emit_loading, init_loading};
use tokio::sync::Semaphore;
use uuid::Uuid;

const SOLARA_CURSEFORGE_API_BASE: &str = "https://api.solara-inc.in/v1/launcher/curseforge";

pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("curseforge")
        .invoke_handler(tauri::generate_handler![
            curseforge_install_modpack,
            curseforge_install_content
        ])
        .build()
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CurseForgeContentInstallRequest {
    pub instance_id: String,
    pub content_type: String,
    pub mod_id: u64,
    pub file_id: u64,
    pub download_url: String,
    pub file_name: Option<String>,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CurseForgeContentInstallResult {
    pub instance_id: String,
    pub file_name: String,
    pub relative_path: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CurseForgeInstallRequest {
    pub mod_id: u64,
    pub file_id: u64,
    pub name: String,
    pub download_url: String,
    pub icon_url: Option<String>,
    pub file_name: Option<String>,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CurseForgeRestrictedMod {
    pub mod_id: u64,
    pub file_id: u64,
    pub file_name: Option<String>,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CurseForgeInstallResult {
    pub instance_id: String,
    pub name: String,
    pub game_version: String,
    pub loader: String,
    pub installed_mods: usize,
    pub restricted_mods: Vec<CurseForgeRestrictedMod>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct CurseForgeManifest {
    minecraft: CurseForgeManifestMinecraft,
    name: Option<String>,
    #[serde(default = "default_overrides")]
    overrides: String,
    #[serde(default)]
    files: Vec<CurseForgeManifestFile>,
}

fn default_overrides() -> String {
    "overrides".to_string()
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct CurseForgeManifestMinecraft {
    version: String,
    #[serde(default)]
    mod_loaders: Vec<CurseForgeManifestLoader>,
}

#[derive(Deserialize, Debug)]
struct CurseForgeManifestLoader {
    id: String,
    #[serde(default)]
    primary: bool,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct CurseForgeManifestFile {
    #[serde(rename = "projectID")]
    project_id: u64,
    #[serde(rename = "fileID")]
    file_id: u64,
    #[serde(default = "default_true")]
    required: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ResolveFilesResponse {
    files: Vec<ResolvedCurseForgeFile>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct ResolvedCurseForgeFile {
    id: u64,
    mod_id: u64,
    file_name: String,
    download_url: Option<String>,
    #[serde(default)]
    restricted: bool,
}

fn is_allowed_download_url(raw: &str) -> bool {
    let Ok(parsed) = url::Url::parse(raw) else {
        return false;
    };
    if parsed.scheme() != "https" {
        return false;
    }
    let Some(host) = parsed.host_str() else {
        return false;
    };
    let host_lower = host.to_ascii_lowercase();
    host_lower == "edge.forgecdn.net"
        || host_lower == "mediafilez.forgecdn.net"
        || host_lower == "media.forgecdn.net"
        || host_lower.ends_with(".forgecdn.net")
        || host_lower.ends_with(".curseforge.com")
}

fn parse_curseforge_loader(loaders: &[CurseForgeManifestLoader]) -> (ModLoader, Option<String>) {
    let chosen = loaders
        .iter()
        .find(|l| l.primary)
        .or_else(|| loaders.first());

    let Some(entry) = chosen else {
        return (ModLoader::Vanilla, None);
    };

    let raw = entry.id.trim();
    if let Some(ver) = raw.strip_prefix("neoforge-") {
        return (ModLoader::NeoForge, Some(ver.to_string()));
    }
    if let Some(ver) = raw.strip_prefix("forge-") {
        return (ModLoader::Forge, Some(ver.to_string()));
    }
    if let Some(ver) = raw.strip_prefix("fabric-") {
        return (ModLoader::Fabric, Some(ver.to_string()));
    }
    if let Some(ver) = raw.strip_prefix("quilt-") {
        return (ModLoader::Quilt, Some(ver.to_string()));
    }

    (ModLoader::Vanilla, None)
}

fn safe_relative_override_path(entry_name: &str, overrides_prefix: &str) -> Option<PathBuf> {
    let normalized_prefix = overrides_prefix.trim_matches('/').to_string();
    let prefix = if normalized_prefix.is_empty() {
        "overrides/".to_string()
    } else {
        format!("{normalized_prefix}/")
    };

    let normalized_name = entry_name.replace('\\', "/");
    let stripped = normalized_name.strip_prefix(&prefix)?;
    if stripped.is_empty() || stripped.ends_with('/') {
        return None;
    }

    let candidate = Path::new(stripped);
    let mut clean = PathBuf::new();
    for component in candidate.components() {
        match component {
            Component::Normal(part) => clean.push(part),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => return None,
        }
    }

    if clean.as_os_str().is_empty() {
        None
    } else {
        Some(clean)
    }
}

fn sanitize_mod_filename(raw: &str, fallback_id: u64) -> String {
    let trimmed = raw.trim();
    let basename = Path::new(trimmed)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");
    let cleaned: String = basename
        .chars()
        .filter(|c| !matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|'))
        .collect();
    if cleaned.is_empty() || cleaned == "." || cleaned == ".." {
        format!("cf-mod-{fallback_id}.jar")
    } else {
        cleaned
    }
}

fn target_subdir_for_file(file_name: &str) -> &'static str {
    let lower = file_name.to_ascii_lowercase();
    if lower.ends_with(".zip") {
        "resourcepacks"
    } else {
        "mods"
    }
}

async fn build_http_client() -> anyhow::Result<reqwest::Client> {
    reqwest::Client::builder()
        .user_agent("Solara-Launcher/0.0.82 (+https://app.solara-inc.in)")
        .timeout(std::time::Duration::from_secs(60))
        .connect_timeout(std::time::Duration::from_secs(15))
        .build()
        .context("Failed to initialize HTTP client")
}

#[tauri::command]
pub async fn curseforge_install_content(
    request: CurseForgeContentInstallRequest,
) -> Result<CurseForgeContentInstallResult> {
    Ok(Box::pin(curseforge_install_content_inner(request)).await?)
}

async fn curseforge_install_content_inner(
    request: CurseForgeContentInstallRequest,
) -> anyhow::Result<CurseForgeContentInstallResult> {
    if !is_allowed_download_url(&request.download_url) {
        bail!("Untrusted CurseForge download URL");
    }

    let subdir = match request.content_type.to_ascii_lowercase().as_str() {
        "resourcepack" | "resourcepacks" => "resourcepacks",
        "shader" | "shaders" | "shaderpack" | "shaderpacks" => "shaderpacks",
        "datapack" | "datapacks" => "datapacks",
        _ => "mods",
    };

    let raw_name = request
        .file_name
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| {
            request
                .download_url
                .rsplit('/')
                .next()
                .unwrap_or("curseforge-file.jar")
        });
    let safe_name = sanitize_mod_filename(raw_name, request.file_id);

    let instance_dir = theseus::instance::get_full_path(&request.instance_id).await?;
    let target_dir = instance_dir.join(subdir);
    tokio::fs::create_dir_all(&target_dir).await?;

    let client = build_http_client().await?;
    let resp = client
        .get(&request.download_url)
        .send()
        .await
        .context("Failed to download CurseForge file")?
        .error_for_status()
        .context("CurseForge CDN rejected file download")?;

    let bytes = resp
        .bytes()
        .await
        .context("Failed to read CurseForge file bytes")?;

    let target_path = target_dir.join(&safe_name);
    let tmp_path = target_dir.join(format!(".solara-cf-{}.tmp", Uuid::new_v4()));
    tokio::fs::write(&tmp_path, &bytes).await?;
    tokio::fs::rename(&tmp_path, &target_path).await?;

    let _ = theseus::instance::sync_content_files(&request.instance_id).await;

    Ok(CurseForgeContentInstallResult {
        instance_id: request.instance_id,
        file_name: safe_name.clone(),
        relative_path: format!("{subdir}/{safe_name}"),
    })
}

#[tauri::command]
pub async fn curseforge_install_modpack(
    request: CurseForgeInstallRequest,
) -> Result<CurseForgeInstallResult> {
    Ok(Box::pin(curseforge_install_modpack_inner(request)).await?)
}

async fn curseforge_install_modpack_inner(
    request: CurseForgeInstallRequest,
) -> anyhow::Result<CurseForgeInstallResult> {
    if !is_allowed_download_url(&request.download_url) {
        bail!("Unsupported or non-HTTPS CurseForge download URL");
    }

    let client = build_http_client().await?;
    let pack_title = if request.name.trim().is_empty() {
        format!("CurseForge Pack {}", request.mod_id)
    } else {
        request.name.trim().to_string()
    };

    let loading_bar = init_loading(
        LoadingBarType::PackDownload {
            instance_id: format!("cf-{}-{}", request.mod_id, request.file_id),
            pack_name: pack_title.clone(),
            icon: request.icon_url.clone(),
            pack_id: Some(request.mod_id.to_string()),
            pack_version: Some(request.file_id.to_string()),
        },
        100.0,
        &format!("Downloading {pack_title}"),
    )
    .await?;

    let result = async {
        // 1. Download modpack zip archive
        let zip_resp = client
            .get(&request.download_url)
            .send()
            .await
            .context("Failed to download CurseForge modpack archive")?
            .error_for_status()
            .context("CurseForge modpack download returned an HTTP error")?;

        let zip_bytes = zip_resp
            .bytes()
            .await
            .context("Failed to read CurseForge modpack bytes")?;

        let _ = emit_loading(&loading_bar, 12.0, Some("Reading CurseForge manifest"));

        // 2. Parse manifest.json from the archive
        let manifest: CurseForgeManifest = {
            let cursor = std::io::Cursor::new(&zip_bytes);
            let mut archive = zip::ZipArchive::new(cursor)
                .context("CurseForge modpack is not a valid ZIP archive")?;
            let mut manifest_entry = archive
                .by_name("manifest.json")
                .context("CurseForge modpack archive is missing manifest.json")?;
            let mut buf = String::new();
            manifest_entry
                .read_to_string(&mut buf)
                .context("Failed to read manifest.json from archive")?;
            serde_json::from_str(&buf).context("Invalid manifest.json in CurseForge modpack")?
        };

        let game_version = manifest.minecraft.version.trim().to_string();
        if game_version.is_empty() {
            bail!("CurseForge manifest.json is missing minecraft.version");
        }

        let (loader, loader_version) = parse_curseforge_loader(&manifest.minecraft.mod_loaders);
        let final_name = if !pack_title.is_empty() {
            pack_title.to_string()
        } else {
            manifest
                .name
                .clone()
                .unwrap_or_else(|| format!("CurseForge Pack {}", request.mod_id))
        };

        // 3. Cache icon if provided
        let cached_icon_path = if let Some(icon_url) = request.icon_url.as_deref() {
            theseus::instance::cache_remote_icon(icon_url)
                .await
                .ok()
                .map(|p| p.to_string_lossy().to_string())
        } else {
            None
        };

        let _ = emit_loading(
            &loading_bar,
            10.0,
            Some(&format!("Creating {} ({}) instance", final_name, game_version)),
        );

        // 4. Create base instance with Minecraft + ModLoader via Theseus
        let snapshot = create_instance(
            final_name.clone(),
            game_version.clone(),
            loader,
            loader_version.clone(),
            cached_icon_path,
            None,
            InstanceLink::Unmanaged,
        )
        .await?;

        let job_uuid = Uuid::parse_str(&snapshot.job_id)
            .context("Invalid install job UUID returned by create_instance")?;

        let mut instance_id = snapshot.instance_id.clone();
        loop {
            let current = get_job(job_uuid).await?;
            if let Some(id) = current.instance_id.clone() {
                instance_id = Some(id);
            }
            match current.status {
                InstallJobStatus::Succeeded => break,
                InstallJobStatus::Failed
                | InstallJobStatus::Canceled
                | InstallJobStatus::Interrupted => {
                    let err_msg = current
                        .error
                        .map(|e| e.message)
                        .unwrap_or_else(|| "Instance creation failed".to_string());
                    bail!("Failed to create base Minecraft instance: {err_msg}");
                }
                InstallJobStatus::Queued | InstallJobStatus::Running => {
                    tokio::time::sleep(std::time::Duration::from_millis(250)).await;
                }
            }
        }

        let instance_id = instance_id.context("Instance ID was not populated after creation")?;
        let instance_dir = theseus::instance::get_full_path(&instance_id).await?;

        let _ = emit_loading(&loading_bar, 18.0, Some("Extracting modpack overrides"));

        // 5. Extract overrides safely on a blocking thread
        let zip_bytes_for_extract = zip_bytes.clone();
        let overrides_prefix = manifest.overrides.clone();
        let instance_dir_for_extract = instance_dir.clone();
        tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
            let cursor = std::io::Cursor::new(&zip_bytes_for_extract);
            let mut archive = zip::ZipArchive::new(cursor)?;
            for i in 0..archive.len() {
                let mut file = archive.by_index(i)?;
                if file.is_dir() {
                    continue;
                }
                let Ok(entry_name) = file.name() else {
                    continue;
                };
                let Some(rel_path) = safe_relative_override_path(&entry_name, &overrides_prefix)
                else {
                    continue;
                };
                let dest = instance_dir_for_extract.join(&rel_path);
                if let Some(parent) = dest.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                let mut out = std::fs::File::create(&dest)?;
                std::io::copy(&mut file, &mut out)?;
            }
            Ok(())
        })
        .await
        .context("Override extraction task failed")??;

        // 6. Resolve & download required mod files through Solara's private CurseForge proxy
        let required_files: Vec<&CurseForgeManifestFile> =
            manifest.files.iter().filter(|f| f.required).collect();

        let mut resolved_all: Vec<ResolvedCurseForgeFile> = Vec::with_capacity(required_files.len());
        let mut seen_file_ids = HashSet::new();
        let file_ids: Vec<u64> = required_files
            .iter()
            .filter_map(|f| {
                if seen_file_ids.insert(f.file_id) {
                    Some(f.file_id)
                } else {
                    None
                }
            })
            .collect();

        if !file_ids.is_empty() {
            let _ = emit_loading(
                &loading_bar,
                5.0,
                Some(&format!("Resolving {} modpack files", file_ids.len())),
            );

            for chunk in file_ids.chunks(150) {
                let resolve_url = format!("{SOLARA_CURSEFORGE_API_BASE}/resolve-files");
                let resp = client
                    .post(&resolve_url)
                    .json(&serde_json::json!({ "fileIds": chunk }))
                    .send()
                    .await
                    .context("Failed to contact Solara CurseForge file resolver")?
                    .error_for_status()
                    .context("Solara CurseForge file resolver returned an error")?;
                let parsed: ResolveFilesResponse = resp
                    .json()
                    .await
                    .context("Invalid response from Solara CurseForge file resolver")?;
                resolved_all.extend(parsed.files);
            }
        }

        let mut restricted_mods: Vec<CurseForgeRestrictedMod> = Vec::new();
        let mut downloadable: Vec<ResolvedCurseForgeFile> = Vec::new();
        let mut resolved_ids = HashSet::new();

        for f in resolved_all {
            resolved_ids.insert(f.id);
            if f.restricted || f.download_url.is_none() {
                restricted_mods.push(CurseForgeRestrictedMod {
                    mod_id: f.mod_id,
                    file_id: f.id,
                    file_name: if f.file_name.is_empty() {
                        None
                    } else {
                        Some(f.file_name)
                    },
                });
            } else if let Some(url) = f.download_url.as_deref() {
                if is_allowed_download_url(url) {
                    downloadable.push(f);
                } else {
                    restricted_mods.push(CurseForgeRestrictedMod {
                        mod_id: f.mod_id,
                        file_id: f.id,
                        file_name: Some(f.file_name),
                    });
                }
            }
        }

        for req_file in &required_files {
            if !resolved_ids.contains(&req_file.file_id) {
                restricted_mods.push(CurseForgeRestrictedMod {
                    mod_id: req_file.project_id,
                    file_id: req_file.file_id,
                    file_name: None,
                });
            }
        }

        let total_downloads = downloadable.len();
        let per_file_progress = if total_downloads > 0 {
            50.0 / (total_downloads as f64)
        } else {
            50.0
        };

        let semaphore = Arc::new(Semaphore::new(8));
        let mut handles = Vec::with_capacity(total_downloads);

        for file in downloadable {
            let sem = Arc::clone(&semaphore);
            let client_clone = client.clone();
            let instance_dir_clone = instance_dir.clone();
            let bar_clone = loading_bar.clone();
            handles.push(tokio::spawn(async move {
                let _permit = sem.acquire_owned().await?;
                let url = file.download_url.as_deref().unwrap_or_default();
                let clean_name = sanitize_mod_filename(&file.file_name, file.id);
                let subdir = target_subdir_for_file(&clean_name);
                let dest_dir = instance_dir_clone.join(subdir);
                tokio::fs::create_dir_all(&dest_dir).await?;
                let dest_path = dest_dir.join(&clean_name);

                let bytes = client_clone
                    .get(url)
                    .send()
                    .await
                    .with_context(|| format!("Failed downloading {clean_name}"))?
                    .error_for_status()
                    .with_context(|| format!("HTTP error downloading {clean_name}"))?
                    .bytes()
                    .await
                    .with_context(|| format!("Failed reading bytes for {clean_name}"))?;

                tokio::fs::write(&dest_path, &bytes).await?;
                let _ = emit_loading(
                    &bar_clone,
                    per_file_progress,
                    Some(&format!("Downloaded {clean_name}")),
                );
                Ok::<(), anyhow::Error>(())
            }));
        }

        let mut installed_mods = 0usize;
        for handle in handles {
            match handle.await {
                Ok(Ok(())) => installed_mods += 1,
                Ok(Err(err)) => {
                    tracing::warn!("CurseForge mod file download failed: {err:#}");
                }
                Err(join_err) => {
                    tracing::warn!("CurseForge mod download task panicked: {join_err}");
                }
            }
        }

        Ok(CurseForgeInstallResult {
            instance_id,
            name: final_name,
            game_version,
            loader: loader.as_str().to_string(),
            installed_mods,
            restricted_mods,
        })
    }
    .await;

    let _ = emit_loading(&loading_bar, 100.0, Some("Finished"));
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_curseforge_modloaders() {
        let forge = vec![CurseForgeManifestLoader {
            id: "forge-47.2.20".to_string(),
            primary: true,
        }];
        assert_eq!(
            parse_curseforge_loader(&forge),
            (ModLoader::Forge, Some("47.2.20".to_string()))
        );

        let fabric = vec![CurseForgeManifestLoader {
            id: "fabric-0.15.11".to_string(),
            primary: true,
        }];
        assert_eq!(
            parse_curseforge_loader(&fabric),
            (ModLoader::Fabric, Some("0.15.11".to_string()))
        );

        let neoforge = vec![CurseForgeManifestLoader {
            id: "neoforge-21.1.50".to_string(),
            primary: true,
        }];
        assert_eq!(
            parse_curseforge_loader(&neoforge),
            (ModLoader::NeoForge, Some("21.1.50".to_string()))
        );

        let quilt = vec![CurseForgeManifestLoader {
            id: "quilt-0.25.0".to_string(),
            primary: true,
        }];
        assert_eq!(
            parse_curseforge_loader(&quilt),
            (ModLoader::Quilt, Some("0.25.0".to_string()))
        );
    }

    #[test]
    fn rejects_zip_slip_override_paths() {
        assert_eq!(
            safe_relative_override_path("overrides/config/sodium.json", "overrides"),
            Some(PathBuf::from("config/sodium.json"))
        );
        assert_eq!(
            safe_relative_override_path("overrides/../../evil.exe", "overrides"),
            None
        );
        assert_eq!(
            safe_relative_override_path("other/config.txt", "overrides"),
            None
        );
    }

    #[test]
    fn enforces_https_forgecdn_urls() {
        assert!(is_allowed_download_url(
            "https://edge.forgecdn.net/files/5123/456/pack.zip"
        ));
        assert!(is_allowed_download_url(
            "https://mediafilez.forgecdn.net/files/5123/456/mod.jar"
        ));
        assert!(!is_allowed_download_url(
            "http://edge.forgecdn.net/files/5123/456/pack.zip"
        ));
        assert!(!is_allowed_download_url("https://evil.example.com/mod.jar"));
    }
}
