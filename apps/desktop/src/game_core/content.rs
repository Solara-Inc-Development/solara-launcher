//! Independently authored Modrinth content and .mrpack modpack installation.
//! Every remote file is verified against both SHA-1 and SHA-512 from cdn.modrinth.com.
use super::{
    download::{self, Cancellation, FileCache},
    files,
    install::Progress,
    instances::{Instance, Instances},
    loaders,
    metadata::{self, Artifact},
};
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha512};
use std::{
    collections::HashSet,
    io::{Cursor, Read, Write},
    path::Path,
};
use tokio::io::AsyncReadExt;

const API_LIMIT: usize = 4 * 1024 * 1024;
const MRPACK_LIMIT: usize = 64 * 1024 * 1024;
const FILE_LIMIT: u64 = 512 * 1024 * 1024;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VerifiedFile {
    pub destination: String,
    pub artifact: Artifact,
    pub sha512: String,
}

pub struct PreparedModpack {
    pub name: String,
    pub game_version: String,
    pub loader: String,
    pub loader_version: Option<String>,
    pub mrpack_bytes: Vec<u8>,
    pub files: Vec<VerifiedFile>,
}

pub fn provider_id(value: &str) -> Result<()> {
    if value.is_empty()
        || value.len() > 100
        || !value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_-".contains(&b))
    {
        bail!("Invalid content identifier")
    }
    Ok(())
}

pub fn cdn_url(value: &str) -> Result<reqwest::Url> {
    let url = reqwest::Url::parse(value)?;
    if url.scheme() != "https"
        || !url.username().is_empty()
        || url.password().is_some()
        || url.port().is_some()
        || url.fragment().is_some()
        || url.host_str() != Some("cdn.modrinth.com")
    {
        bail!("Untrusted content download origin")
    }
    Ok(url)
}

fn sha1_hex(value: &str) -> Result<String> {
    if value.len() != 40 || !value.bytes().all(|b| b.is_ascii_hexdigit()) {
        bail!("Invalid content SHA-1 checksum")
    }
    Ok(value.to_ascii_lowercase())
}

fn sha512_hex(value: &str) -> Result<String> {
    if value.len() != 128 || !value.bytes().all(|b| b.is_ascii_hexdigit()) {
        bail!("Invalid content SHA-512 checksum")
    }
    Ok(value.to_ascii_lowercase())
}

fn forbidden_extension(path: &str) -> bool {
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
                    | "js"
                    | "jse"
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

pub fn allowed_content_destination(path: &str) -> Result<()> {
    metadata::relative_path(path)?;
    if forbidden_extension(path) {
        bail!("Executable files are not permitted in instance content")
    }
    let Some((top, rest)) = path.split_once('/') else {
        bail!("Content file must be placed inside a supported instance folder")
    };
    if rest.is_empty()
        || !matches!(
            top,
            "mods" | "resourcepacks" | "shaderpacks" | "config" | "datapacks" | "defaultconfigs"
        )
    {
        bail!("Content file target folder is not permitted")
    }
    if top == "mods" {
        let lower = rest.to_ascii_lowercase();
        if rest.contains('/') || !(lower.ends_with(".jar") || lower.ends_with(".jar.disabled")) {
            bail!("Mod files must be JAR files in the instance mods folder")
        }
    }
    Ok(())
}

pub fn allowed_override_destination(path: &str) -> Result<()> {
    metadata::relative_path(path)?;
    if forbidden_extension(path) {
        bail!("Executable files are not permitted in modpack overrides")
    }
    let top = path.split('/').next().unwrap_or("");
    if matches!(top, ".solara-trash" | "runtime" | "natives") {
        bail!("Modpack overrides cannot modify protected launcher directories")
    }
    Ok(())
}

fn http_client() -> Result<reqwest::Client> {
    Ok(reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(60))
        .user_agent("SolaraLauncher/independent-core")
        .build()?)
}

async fn fetch_api_json(url: reqwest::Url) -> Result<Value> {
    if url.scheme() != "https" || url.host_str() != Some("api.modrinth.com") {
        bail!("Untrusted content metadata origin")
    }
    let mut response = http_client()?.get(url).send().await?;
    if !response.status().is_success() {
        bail!("Content provider returned {}", response.status().as_u16())
    }
    if response
        .content_length()
        .is_some_and(|n| n > API_LIMIT as u64)
    {
        bail!("Content provider response exceeds limit")
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await? {
        if bytes.len() + chunk.len() > API_LIMIT {
            bail!("Content provider response exceeds limit")
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(serde_json::from_slice(&bytes)?)
}

fn extract_version_file(version: &Value, folder: &str) -> Result<VerifiedFile> {
    let files = version["files"]
        .as_array()
        .context("Content version has no files")?;
    let chosen = files
        .iter()
        .find(|f| f["primary"].as_bool() == Some(true))
        .or_else(|| files.first())
        .context("Content version has no downloadable file")?;
    let name = chosen["filename"]
        .as_str()
        .context("Content filename missing")?;
    if name.contains('/') || name.contains('\\') {
        bail!("Invalid content filename")
    }
    let destination = format!("{folder}/{name}");
    allowed_content_destination(&destination)?;
    let url = chosen["url"].as_str().context("Content file URL missing")?;
    cdn_url(url)?;
    let size = chosen["size"]
        .as_u64()
        .filter(|n| (1..=FILE_LIMIT).contains(n))
        .context("Invalid content file size")?;
    let sha1 = sha1_hex(
        chosen["hashes"]["sha1"]
            .as_str()
            .context("Content SHA-1 missing")?,
    )?;
    let sha512 = sha512_hex(
        chosen["hashes"]["sha512"]
            .as_str()
            .context("Content SHA-512 missing")?,
    )?;
    let artifact = Artifact {
        url: url.into(),
        sha1,
        size,
        path: Some(destination.clone()),
    };
    artifact.validate()?;
    Ok(VerifiedFile {
        destination,
        artifact,
        sha512,
    })
}

fn version_matches_instance(version: &Value, game_version: &str, loader: &str, kind: &str) -> bool {
    let games_ok = version["game_versions"]
        .as_array()
        .is_some_and(|arr| arr.iter().any(|g| g.as_str() == Some(game_version)));
    if !games_ok {
        return false;
    }
    if kind != "mod" {
        return true;
    }
    version["loaders"].as_array().is_some_and(|arr| {
        arr.iter().any(|l| {
            l.as_str() == Some(loader) || (loader == "quilt" && l.as_str() == Some("fabric"))
        })
    })
}

async fn select_project_version(
    project_id: &str,
    game_version: &str,
    loader: &str,
    kind: &str,
    requested: Option<&str>,
) -> Result<Value> {
    provider_id(project_id)?;
    let mut url = reqwest::Url::parse(&format!(
        "https://api.modrinth.com/v2/project/{project_id}/version"
    ))?;
    url.query_pairs_mut()
        .append_pair("game_versions", &serde_json::to_string(&[game_version])?);
    let raw = fetch_api_json(url).await?;
    let list = raw.as_array().context("Invalid provider version list")?;
    let compatible: Vec<&Value> = list
        .iter()
        .filter(|v| {
            v["project_id"].as_str().is_some()
                && version_matches_instance(v, game_version, loader, kind)
        })
        .collect();
    if let Some(req_id) = requested.filter(|s| !s.is_empty()) {
        provider_id(req_id)?;
        return compatible
            .into_iter()
            .find(|v| v["id"].as_str() == Some(req_id))
            .cloned()
            .with_context(|| {
                format!(
                    "Requested version is not compatible with Minecraft {game_version} ({loader})"
                )
            });
    }
    if kind == "mod" && loader == "quilt" {
        if let Some(native_quilt) = compatible
            .iter()
            .find(|v| {
                v["version_type"].as_str() == Some("release")
                    && v["loaders"]
                        .as_array()
                        .is_some_and(|a| a.iter().any(|x| x.as_str() == Some("quilt")))
            })
            .or_else(|| {
                compatible.iter().find(|v| {
                    v["loaders"]
                        .as_array()
                        .is_some_and(|a| a.iter().any(|x| x.as_str() == Some("quilt")))
                })
            })
        {
            return Ok((*native_quilt).clone());
        }
    }
    compatible
        .iter()
        .find(|v| v["version_type"].as_str() == Some("release"))
        .or_else(|| compatible.first())
        .map(|v| (*v).clone())
        .with_context(|| {
            format!("No compatible {kind} version found for Minecraft {game_version} ({loader})")
        })
}

pub async fn plan_project_install(
    instance: &Instance,
    project_id: &str,
    content_type: &str,
    version_id: Option<&str>,
) -> Result<(String, Vec<VerifiedFile>)> {
    provider_id(project_id)?;
    let folder = match content_type {
        "mod" => {
            if !matches!(
                instance.loader.as_str(),
                "fabric" | "quilt" | "forge" | "neoforge"
            ) {
                bail!("Choose a Fabric, Quilt, Forge, or NeoForge instance to install mods")
            }
            "mods"
        }
        "resourcepack" => "resourcepacks",
        "shader" => "shaderpacks",
        _ => bail!("Unsupported content type"),
    };
    let root = select_project_version(
        project_id,
        &instance.game_version,
        &instance.loader,
        content_type,
        version_id,
    )
    .await?;
    let label = root["name"]
        .as_str()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or(project_id)
        .chars()
        .take(120)
        .collect::<String>();
    let mut planned = Vec::new();
    let mut seen_projects = HashSet::new();
    if let Some(root_pid) = root["project_id"].as_str() {
        seen_projects.insert(root_pid.to_string());
    }
    seen_projects.insert(project_id.to_string());

    if content_type == "mod" {
        let mut queue = Vec::new();
        if let Some(deps) = root["dependencies"].as_array() {
            for dep in deps {
                if dep["dependency_type"].as_str() == Some("required") {
                    queue.push(dep.clone());
                }
            }
        }
        let mut resolved_deps = 0usize;
        while let Some(dep) = queue.pop() {
            if resolved_deps >= 24 {
                bail!("Mod dependency graph exceeds safety limit")
            }
            let dep_project = dep["project_id"].as_str().unwrap_or("");
            if !dep_project.is_empty() && seen_projects.contains(dep_project) {
                continue;
            }
            let dep_version = if let Some(pid) = (!dep_project.is_empty()).then_some(dep_project) {
                select_project_version(pid, &instance.game_version, &instance.loader, "mod", None)
                    .await?
            } else if let Some(vid) = dep["version_id"].as_str().filter(|s| !s.is_empty()) {
                provider_id(vid)?;
                let v = fetch_api_json(reqwest::Url::parse(&format!(
                    "https://api.modrinth.com/v2/version/{vid}"
                ))?)
                .await?;
                if !version_matches_instance(&v, &instance.game_version, &instance.loader, "mod") {
                    let pid = v["project_id"]
                        .as_str()
                        .context("Required dependency project ID missing")?;
                    select_project_version(
                        pid,
                        &instance.game_version,
                        &instance.loader,
                        "mod",
                        None,
                    )
                    .await?
                } else {
                    v
                }
            } else {
                continue;
            };
            if let Some(actual_pid) = dep_version["project_id"].as_str() {
                if !seen_projects.insert(actual_pid.to_string()) {
                    continue;
                }
            }
            resolved_deps += 1;
            if let Some(sub_deps) = dep_version["dependencies"].as_array() {
                for sub in sub_deps {
                    if sub["dependency_type"].as_str() == Some("required") {
                        queue.push(sub.clone());
                    }
                }
            }
            planned.push(extract_version_file(&dep_version, folder)?);
        }
    }

    planned.push(extract_version_file(&root, folder)?);
    let mut seen_dest = HashSet::new();
    planned.retain(|f| seen_dest.insert(f.destination.clone()));
    Ok((label, planned))
}

pub async fn verify_sha512(path: &Path, expected_sha512: &str, expected_size: u64) -> Result<bool> {
    let expected = sha512_hex(expected_sha512)?;
    let meta = match tokio::fs::symlink_metadata(path).await {
        Ok(m) => m,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(e) => return Err(e.into()),
    };
    if !meta.is_file() || files::is_link(&meta) || meta.len() != expected_size {
        return Ok(false);
    }
    let mut file = tokio::fs::File::open(path).await?;
    let mut hasher = Sha512::new();
    let mut buffer = [0u8; 65536];
    loop {
        let read = file.read(&mut buffer).await?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()) == expected)
}

pub async fn install_files(
    store: &Instances,
    cache: &FileCache,
    instance_id: &str,
    files: &[VerifiedFile],
    cancel: &Cancellation,
    mut report: impl FnMut(Progress) + Send,
) -> Result<()> {
    cancel.check()?;
    let total = files.iter().try_fold(0u64, |sum, f| {
        sum.checked_add(f.artifact.size)
            .context("Content install size overflow")
    })?;
    if total > 8 * 1024 * 1024 * 1024 {
        bail!("Content installation exceeds size limit")
    }
    let mut completed = 0u64;
    for (idx, item) in files.iter().enumerate() {
        cancel.check()?;
        allowed_content_destination(&item.destination)?;
        let target_final =
            files::resolve_create_parents(store, instance_id, &item.destination).await?;
        if download::verified_file(&target_final, &item.artifact).await?
            && verify_sha512(&target_final, &item.sha512, item.artifact.size).await?
        {
            completed = completed.saturating_add(item.artifact.size);
            report(Progress {
                phase: format!("Verified file {} of {}", idx + 1, files.len()),
                verified_bytes: completed,
                total_bytes: total,
            });
            continue;
        }
        let phase = format!("Downloading file {} of {}", idx + 1, files.len());
        let cached = cache
            .obtain(&item.artifact, cancel, |done, _| {
                report(Progress {
                    phase: phase.clone(),
                    verified_bytes: completed.saturating_add(done),
                    total_bytes: total,
                })
            })
            .await?;
        if !verify_sha512(&cached, &item.sha512, item.artifact.size).await? {
            let _ = tokio::fs::remove_file(&cached).await;
            bail!("Content download SHA-512 integrity check failed")
        }
        let target_final =
            files::resolve_create_parents(store, instance_id, &item.destination).await?;
        let temp =
            target_final.with_file_name(format!(".solara-content-{}.tmp", uuid::Uuid::new_v4()));
        let copy_res = tokio::fs::copy(&cached, &temp).await;
        if let Err(e) = copy_res {
            let _ = tokio::fs::remove_file(&temp).await;
            return Err(e.into());
        }
        if let Err(e) = tokio::fs::rename(&temp, &target_final).await {
            let _ = tokio::fs::remove_file(&temp).await;
            return Err(e.into());
        }
        completed = completed.saturating_add(item.artifact.size);
    }
    report(Progress {
        phase: "Content installed".into(),
        verified_bytes: total,
        total_bytes: total,
    });
    Ok(())
}

pub fn parse_mrpack(
    bytes: &[u8],
) -> Result<(String, String, String, Option<String>, Vec<VerifiedFile>)> {
    if bytes.is_empty() || bytes.len() > MRPACK_LIMIT {
        bail!("Modpack archive size is invalid")
    }
    let tail = bytes.len().min(65557);
    let tail_slice = &bytes[bytes.len() - tail..];
    let eocd = tail_slice
        .windows(4)
        .rposition(|w| w == b"PK\x05\x06")
        .context("Modpack file is not a valid ZIP archive")?;
    if eocd + 22 > tail_slice.len() {
        bail!("Invalid modpack archive directory")
    }
    let entry_count = u16::from_le_bytes([tail_slice[eocd + 10], tail_slice[eocd + 11]]);
    let central_size = u32::from_le_bytes(tail_slice[eocd + 12..eocd + 16].try_into()?);
    if entry_count > 4096 || central_size > 8 * 1024 * 1024 {
        bail!("Modpack archive metadata exceeds safety limit")
    }

    let mut zip = zip::ZipArchive::new(Cursor::new(bytes))?;
    if zip.len() > 4096 {
        bail!("Modpack archive file count exceeds limit")
    }
    let index_value: Value = {
        let mut index_entry = zip
            .by_name("modrinth.index.json")
            .context("modrinth.index.json is missing from the modpack")?;
        if index_entry.size() > API_LIMIT as u64 {
            bail!("modrinth.index.json exceeds size limit")
        }
        let mut raw = Vec::new();
        index_entry
            .by_ref()
            .take(API_LIMIT as u64 + 1)
            .read_to_end(&mut raw)?;
        if raw.len() > API_LIMIT {
            bail!("modrinth.index.json exceeds size limit")
        }
        serde_json::from_slice(&raw)?
    };
    if index_value["formatVersion"].as_u64() != Some(1)
        || index_value["game"].as_str() != Some("minecraft")
    {
        bail!("Unsupported modpack format")
    }
    let raw_name = index_value["name"]
        .as_str()
        .context("Modpack name missing")?
        .chars()
        .filter(|c| !c.is_control())
        .collect::<String>();
    let name: String = raw_name.trim().chars().take(80).collect();
    if name.is_empty() {
        bail!("Modpack name is empty")
    }
    let deps = index_value["dependencies"]
        .as_object()
        .context("Modpack dependencies missing")?;
    for key in deps.keys() {
        if !matches!(
            key.as_str(),
            "minecraft" | "fabric-loader" | "quilt-loader" | "forge" | "neoforge"
        ) {
            bail!("Modpack requires an unsupported dependency: {key}");
        }
    }
    let loader_count = ["fabric-loader", "quilt-loader", "forge", "neoforge"]
        .into_iter()
        .filter(|k| deps.contains_key(*k))
        .count();
    if loader_count > 1 {
        bail!("Modpack cannot require multiple loaders simultaneously");
    }
    let game_version = deps
        .get("minecraft")
        .and_then(Value::as_str)
        .context("Modpack Minecraft version missing")?
        .to_string();
    metadata::version_id(&game_version)?;
    let (loader, loader_version) =
        if let Some(v) = deps.get("fabric-loader").and_then(Value::as_str) {
            loaders::loader_id(v)?;
            ("fabric".to_string(), Some(v.to_string()))
        } else if let Some(v) = deps.get("quilt-loader").and_then(Value::as_str) {
            loaders::loader_id(v)?;
            ("quilt".to_string(), Some(v.to_string()))
        } else if let Some(v) = deps.get("neoforge").and_then(Value::as_str) {
            loaders::loader_id(v)?;
            ("neoforge".to_string(), Some(v.to_string()))
        } else if let Some(v) = deps.get("forge").and_then(Value::as_str) {
            loaders::loader_id(v)?;
            ("forge".to_string(), Some(v.to_string()))
        } else {
            ("vanilla".to_string(), None)
        };

    let files_arr = index_value["files"]
        .as_array()
        .context("Modpack file list missing")?;
    if files_arr.len() > 1000 {
        bail!("Modpack file count exceeds limit")
    }
    let mut planned = Vec::new();
    let mut seen_paths = HashSet::new();
    for item in files_arr {
        if item["env"]["client"].as_str() == Some("unsupported") {
            continue;
        }
        let path = item["path"].as_str().context("Modpack file path missing")?;
        allowed_content_destination(path)?;
        if !seen_paths.insert(path.to_string()) {
            bail!("Duplicate modpack file destination")
        }
        let size = item["fileSize"]
            .as_u64()
            .filter(|n| (1..=FILE_LIMIT).contains(n))
            .context("Invalid modpack entry size")?;
        let sha1 = sha1_hex(
            item["hashes"]["sha1"]
                .as_str()
                .context("Modpack file SHA-1 missing")?,
        )?;
        let sha512 = sha512_hex(
            item["hashes"]["sha512"]
                .as_str()
                .context("Modpack file SHA-512 missing")?,
        )?;
        let downloads = item["downloads"]
            .as_array()
            .context("Modpack file downloads missing")?;
        let url = downloads
            .iter()
            .filter_map(Value::as_str)
            .find(|u| cdn_url(u).is_ok())
            .context("Modpack file has no trusted cdn.modrinth.com download URL")?;
        let artifact = Artifact {
            url: url.into(),
            sha1,
            size,
            path: Some(path.into()),
        };
        artifact.validate()?;
        planned.push(VerifiedFile {
            destination: path.into(),
            artifact,
            sha512,
        });
    }

    let mut total_overrides = 0u64;
    for i in 0..zip.len() {
        let entry = zip.by_index(i)?;
        if entry
            .unix_mode()
            .is_some_and(|mode| mode & 0o170000 == 0o120000)
        {
            bail!("Modpack archive symlinks are not allowed")
        }
        let name = entry.name();
        if name == "modrinth.index.json" || entry.is_dir() {
            continue;
        }
        let Some(rel) = name
            .strip_prefix("overrides/")
            .or_else(|| name.strip_prefix("client-overrides/"))
        else {
            continue;
        };
        if rel.is_empty() {
            continue;
        }
        allowed_override_destination(rel)?;
        if entry.size() > 64 * 1024 * 1024 {
            bail!("Modpack override file exceeds limit")
        }
        total_overrides = total_overrides
            .checked_add(entry.size())
            .context("Modpack override size overflow")?;
        if total_overrides > 256 * 1024 * 1024 {
            bail!("Modpack overrides exceed total extraction limit")
        }
    }

    Ok((name, game_version, loader, loader_version, planned))
}

pub fn extract_mrpack_overrides(bytes: &[u8], game_dir: &Path) -> Result<()> {
    let (_, _, _, _, _) = parse_mrpack(bytes)?;
    let meta = std::fs::symlink_metadata(game_dir)?;
    if !meta.is_dir() || files::is_link(&meta) {
        bail!("Unsafe instance game directory")
    }
    let mut zip = zip::ZipArchive::new(Cursor::new(bytes))?;
    for prefix in ["overrides/", "client-overrides/"] {
        for i in 0..zip.len() {
            let mut entry = zip.by_index(i)?;
            if entry.is_dir() {
                continue;
            }
            let Some(rel) = entry.name().strip_prefix(prefix) else {
                continue;
            };
            if rel.is_empty() {
                continue;
            }
            allowed_override_destination(rel)?;
            let mut cur = game_dir.to_path_buf();
            let parts: Vec<_> = rel.split('/').collect();
            for (idx, part) in parts.iter().enumerate() {
                cur.push(part);
                if idx + 1 < parts.len() {
                    match std::fs::symlink_metadata(&cur) {
                        Ok(m) => {
                            if !m.is_dir() || files::is_link(&m) {
                                bail!("Unsafe override parent folder")
                            }
                        }
                        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                            std::fs::create_dir(&cur)?;
                        }
                        Err(e) => return Err(e.into()),
                    }
                } else if let Ok(m) = std::fs::symlink_metadata(&cur) {
                    if !m.is_file() || files::is_link(&m) {
                        bail!("Unsafe override target file")
                    }
                }
            }
            let mut content = Vec::new();
            entry
                .by_ref()
                .take(64 * 1024 * 1024 + 1)
                .read_to_end(&mut content)?;
            if content.len() as u64 != entry.size() {
                bail!("Modpack override entry size mismatch")
            }
            let temp = cur.with_file_name(format!(".solara-override-{}.tmp", uuid::Uuid::new_v4()));
            let write_res = (|| -> Result<()> {
                let mut f = std::fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&temp)?;
                f.write_all(&content)?;
                f.sync_all()?;
                drop(f);
                std::fs::rename(&temp, &cur)?;
                Ok(())
            })();
            if let Err(e) = write_res {
                let _ = std::fs::remove_file(&temp);
                return Err(e);
            }
        }
    }
    Ok(())
}

pub async fn prepare_modpack(project_id: &str, version_id: &str) -> Result<PreparedModpack> {
    provider_id(project_id)?;
    provider_id(version_id)?;
    let version = fetch_api_json(reqwest::Url::parse(&format!(
        "https://api.modrinth.com/v2/version/{version_id}"
    ))?)
    .await?;
    if version["project_id"].as_str() != Some(project_id) {
        bail!("Modpack version does not belong to the selected project")
    }
    let files = version["files"]
        .as_array()
        .context("Modpack version has no files")?;
    let pack_file = files
        .iter()
        .find(|f| {
            f["filename"]
                .as_str()
                .is_some_and(|n| n.ends_with(".mrpack"))
        })
        .context("Selected version does not contain a .mrpack archive")?;
    let url = cdn_url(
        pack_file["url"]
            .as_str()
            .context("Modpack download URL missing")?,
    )?;
    let expected_size = pack_file["size"]
        .as_u64()
        .filter(|n| (1..=MRPACK_LIMIT as u64).contains(n))
        .context("Invalid modpack archive size")?;
    let expected_sha1 = sha1_hex(
        pack_file["hashes"]["sha1"]
            .as_str()
            .context("Modpack SHA-1 missing")?,
    )?;
    let expected_sha512 = sha512_hex(
        pack_file["hashes"]["sha512"]
            .as_str()
            .context("Modpack SHA-512 missing")?,
    )?;

    let mut response = http_client()?.get(url).send().await?;
    if !response.status().is_success() {
        bail!("Modpack download is unavailable")
    }
    if response
        .content_length()
        .is_some_and(|n| n != expected_size)
    {
        bail!("Modpack download size mismatch")
    }
    let mut mrpack_bytes = Vec::with_capacity(expected_size as usize);
    let mut sha1 = sha1_smol::Sha1::new();
    let mut sha512 = Sha512::new();
    while let Some(chunk) = response.chunk().await? {
        if mrpack_bytes.len() + chunk.len() > expected_size as usize {
            bail!("Modpack archive exceeds expected size")
        }
        sha1.update(&chunk);
        sha512.update(&chunk);
        mrpack_bytes.extend_from_slice(&chunk);
    }
    if mrpack_bytes.len() as u64 != expected_size
        || sha1.digest().to_string() != expected_sha1
        || format!("{:x}", sha512.finalize()) != expected_sha512
    {
        bail!("Modpack archive integrity check failed")
    }
    let (name, game_version, loader, raw_loader_version, planned_files) =
        parse_mrpack(&mrpack_bytes)?;
    if !metadata::versions()
        .await?
        .iter()
        .any(|v| v.id == game_version)
    {
        bail!("Modpack Minecraft version is not in the official version list")
    }
    let resolved_loader = match loader.as_str() {
        "vanilla" => None,
        "fabric" | "quilt" | "forge" | "neoforge" => Some(
            loaders::resolve_version(
                &loader,
                &game_version,
                raw_loader_version.as_deref().unwrap_or(""),
            )
            .await?,
        ),
        _ => bail!("Unsupported modpack loader"),
    };
    Ok(PreparedModpack {
        name,
        game_version,
        loader,
        loader_version: resolved_loader,
        mrpack_bytes,
        files: planned_files,
    })
}

pub async fn apply_modpack_if_present(
    store: &Instances,
    cache: &FileCache,
    instance_id: &str,
    cancel: &Cancellation,
    mut report: impl FnMut(Progress) + Send,
) -> Result<()> {
    let pack_path = store.path(instance_id)?.join("modpack.mrpack");
    let meta = match tokio::fs::symlink_metadata(&pack_path).await {
        Ok(m) => m,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(e.into()),
    };
    if !meta.is_file() || files::is_link(&meta) || meta.len() > MRPACK_LIMIT as u64 {
        bail!("Invalid local modpack archive")
    }
    let bytes = tokio::fs::read(&pack_path).await?;
    let (_, game_version, loader, _, planned_files) = parse_mrpack(&bytes)?;
    let record = store.get(instance_id).await?;
    if record.game_version != game_version || record.loader != loader {
        bail!("Modpack archive does not match this instance")
    }
    install_files(
        store,
        cache,
        instance_id,
        &planned_files,
        cancel,
        &mut report,
    )
    .await?;
    cancel.check()?;
    report(Progress {
        phase: "Applying modpack configuration files".into(),
        verified_bytes: 0,
        total_bytes: 0,
    });
    let game_dir = store.game_dir(instance_id).await?;
    tokio::task::spawn_blocking(move || extract_mrpack_overrides(&bytes, &game_dir)).await??;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn build_mrpack_fixture(index: &Value, extra_files: &[(&str, &[u8])]) -> Vec<u8> {
        let mut cursor = Cursor::new(Vec::new());
        {
            let mut zip = zip::ZipWriter::new(&mut cursor);
            let opts = zip::write::SimpleFileOptions::default();
            zip.start_file("modrinth.index.json", opts).unwrap();
            zip.write_all(&serde_json::to_vec(index).unwrap()).unwrap();
            for (name, data) in extra_files {
                zip.start_file(*name, opts).unwrap();
                zip.write_all(data).unwrap();
            }
            zip.finish().unwrap();
        }
        cursor.into_inner()
    }

    #[test]
    fn validates_origins_destinations_and_rejects_executables() {
        assert!(cdn_url("https://cdn.modrinth.com/data/P7dR8mSH/versions/v1/mod.jar").is_ok());
        for bad in [
            "http://cdn.modrinth.com/data/a.jar",
            "https://evil.example.com/mod.jar",
            "https://user:pass@cdn.modrinth.com/mod.jar",
            "https://cdn.modrinth.com:8443/mod.jar",
        ] {
            assert!(cdn_url(bad).is_err(), "{bad}");
        }
        assert!(allowed_content_destination("mods/fabric-api.jar").is_ok());
        assert!(allowed_content_destination("resourcepacks/pack.zip").is_ok());
        assert!(allowed_content_destination("shaderpacks/shader.zip").is_ok());
        assert!(allowed_content_destination("datapacks/pack.zip").is_ok());
        for bad in [
            "mods/../escape.jar",
            "mods/nested/mod.jar",
            "mods/not-a-jar.txt",
            "mods/evil.exe",
            "config/run.bat",
            "saves/world/level.dat",
        ] {
            assert!(allowed_content_destination(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn mrpack_parser_verifies_index_and_extracts_safe_overrides_in_order() {
        let valid_index = json!({
            "formatVersion": 1,
            "game": "minecraft",
            "name": "QA Pack",
            "dependencies": {
                "minecraft": "1.21.1",
                "fabric-loader": "0.20.1"
            },
            "files": [{
                "path": "mods/example.jar",
                "hashes": {
                    "sha1": "a".repeat(40),
                    "sha512": "b".repeat(128)
                },
                "env": { "client": "required" },
                "downloads": ["https://cdn.modrinth.com/data/test/example.jar"],
                "fileSize": 1024
            }]
        });
        let pack = build_mrpack_fixture(
            &valid_index,
            &[
                ("overrides/config/settings.json", br#"{"from":"overrides"}"#),
                (
                    "client-overrides/config/settings.json",
                    br#"{"from":"client-overrides"}"#,
                ),
            ],
        );
        let (name, mc, loader, lver, files) = parse_mrpack(&pack).unwrap();
        assert_eq!(name, "QA Pack");
        assert_eq!(mc, "1.21.1");
        assert_eq!(loader, "fabric");
        assert_eq!(lver.as_deref(), Some("0.20.1"));
        assert_eq!(files.len(), 1);

        let dir = std::env::temp_dir().join(format!("solara-mrpack-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        extract_mrpack_overrides(&pack, &dir).unwrap();
        assert_eq!(
            std::fs::read_to_string(dir.join("config/settings.json")).unwrap(),
            r#"{"from":"client-overrides"}"#
        );
        std::fs::remove_dir_all(&dir).unwrap();

        let bad_override = build_mrpack_fixture(&valid_index, &[("overrides/evil.exe", b"mz")]);
        assert!(parse_mrpack(&bad_override).is_err());

        let traversal_override =
            build_mrpack_fixture(&valid_index, &[("overrides/../escape.txt", b"escape")]);
        assert!(parse_mrpack(&traversal_override).is_err());

        let mut unknown_dep = valid_index.clone();
        unknown_dep["dependencies"]["liteloader"] = json!("1.0");
        assert!(parse_mrpack(&build_mrpack_fixture(&unknown_dep, &[])).is_err());

        let mut dual_loader = valid_index.clone();
        dual_loader["dependencies"]["quilt-loader"] = json!("0.30.1");
        assert!(parse_mrpack(&build_mrpack_fixture(&dual_loader, &[])).is_err());
    }

    #[tokio::test]
    async fn install_files_and_apply_modpack_create_missing_folders_and_verify_hashes() {
        let root =
            std::env::temp_dir().join(format!("solara-content-install-{}", uuid::Uuid::new_v4()));
        let store = Instances::new(root.join("instances"));
        let cache_dir = root.join("cache");
        let cache = FileCache::new(cache_dir.clone()).unwrap();
        let inst = store
            .create(
                "Fresh Quilt Instance".into(),
                "1.21.1".into(),
                "quilt".into(),
                Some("0.30.1".into()),
                4096,
                "auto".into(),
            )
            .await
            .unwrap();

        let mod_bytes = b"verified-jar-payload";
        let mut s1 = sha1_smol::Sha1::new();
        s1.update(mod_bytes);
        let mod_sha1 = s1.digest().to_string();
        let mod_sha512 = format!("{:x}", Sha512::digest(mod_bytes));

        // Seed the verified cache with our exact SHA-1/SHA-512 payload.
        let shard = cache_dir.join(&mod_sha1[..2]);
        tokio::fs::create_dir_all(&shard).await.unwrap();
        tokio::fs::write(shard.join(&mod_sha1), mod_bytes)
            .await
            .unwrap();

        let planned = vec![VerifiedFile {
            destination: "mods/test-mod.jar".into(),
            artifact: Artifact {
                url: "https://cdn.modrinth.com/data/test/versions/1/test-mod.jar".into(),
                sha1: mod_sha1.clone(),
                size: mod_bytes.len() as u64,
                path: Some("mods/test-mod.jar".into()),
            },
            sha512: mod_sha512.clone(),
        }];

        // On a newly created instance, game/mods does not exist yet; install_files must create it safely.
        install_files(
            &store,
            &cache,
            &inst.id,
            &planned,
            &Cancellation::default(),
            |_| {},
        )
        .await
        .unwrap();

        let installed_mod = store
            .game_dir(&inst.id)
            .await
            .unwrap()
            .join("mods/test-mod.jar");
        assert_eq!(tokio::fs::read(&installed_mod).await.unwrap(), mod_bytes);

        // Also test apply_modpack_if_present on a brand new instance where game/mods and game/config/nested do not exist.
        let pack_inst = store
            .create(
                "Fresh Modpack Instance".into(),
                "1.21.1".into(),
                "quilt".into(),
                Some("0.30.1".into()),
                4096,
                "auto".into(),
            )
            .await
            .unwrap();
        let index = json!({
            "formatVersion": 1,
            "game": "minecraft",
            "name": "Fresh Modpack",
            "dependencies": {
                "minecraft": "1.21.1",
                "quilt-loader": "0.30.1"
            },
            "files": [{
                "path": "mods/pack-mod.jar",
                "hashes": {
                    "sha1": mod_sha1,
                    "sha512": mod_sha512
                },
                "env": { "client": "required" },
                "downloads": ["https://cdn.modrinth.com/data/test/versions/1/pack-mod.jar"],
                "fileSize": mod_bytes.len()
            }]
        });
        let mrpack = build_mrpack_fixture(
            &index,
            &[("overrides/config/nested/pack.toml", br#"enabled = true"#)],
        );
        tokio::fs::write(
            store.path(&pack_inst.id).unwrap().join("modpack.mrpack"),
            &mrpack,
        )
        .await
        .unwrap();

        apply_modpack_if_present(
            &store,
            &cache,
            &pack_inst.id,
            &Cancellation::default(),
            |_| {},
        )
        .await
        .unwrap();

        let pack_game = store.game_dir(&pack_inst.id).await.unwrap();
        assert_eq!(
            tokio::fs::read(pack_game.join("mods/pack-mod.jar"))
                .await
                .unwrap(),
            mod_bytes
        );
        assert_eq!(
            tokio::fs::read_to_string(pack_game.join("config/nested/pack.toml"))
                .await
                .unwrap(),
            "enabled = true"
        );

        let _ = tokio::fs::remove_dir_all(&root).await;
    }
}
