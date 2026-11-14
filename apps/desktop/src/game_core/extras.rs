//! Solara extended game core: packs, worlds, screenshots, disk logs, Modrinth updates, Java detection, and skins/capes.
use super::{
    auth::Session,
    content,
    download::{Cancellation, FileCache},
    files::{self, is_link, Files},
    instances::{write_json, Instances},
    metadata::{relative_path, Artifact},
};
use anyhow::{bail, Context, Result};
use base64::Engine as _;
use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::{HashMap, HashSet},
    io::{Cursor, Read},
    path::{Path, PathBuf},
    process::Stdio,
};
use tokio::process::Command;

// ---------------------------------------------------------------------------
// 1. Resource Packs, Shader Packs, Datapacks & Unified Content Listing
// ---------------------------------------------------------------------------

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackEntry {
    pub kind: String,
    pub path: String,
    pub file_name: String,
    pub name: String,
    pub description: Option<String>,
    pub pack_format: Option<i64>,
    pub icon_data_url: Option<String>,
    pub enabled: bool,
    pub directory: bool,
    pub size: u64,
    pub world: Option<String>,
}

fn read_pack_metadata_from_zip(path: &Path) -> (Option<String>, Option<i64>, Option<String>) {
    let Ok(file) = std::fs::File::open(path) else {
        return (None, None, None);
    };
    let Ok(len) = file.metadata().map(|m| m.len()) else {
        return (None, None, None);
    };
    if len > 512 * 1024 * 1024 {
        return (None, None, None);
    }
    let Ok(mut zip) = zip::ZipArchive::new(file) else {
        return (None, None, None);
    };
    let (mut desc, mut fmt) = (None, None);
    if let Ok(mut entry) = zip.by_name("pack.mcmeta") {
        if entry.size() <= 128 * 1024 {
            let mut text = String::new();
            if entry.read_to_string(&mut text).is_ok() {
                if let Ok(v) = serde_json::from_str::<Value>(&text) {
                    fmt = v["pack"]["pack_format"].as_i64();
                    desc = extract_mc_text(&v["pack"]["description"]);
                }
            }
        }
    }
    let mut icon_url = None;
    if let Ok(icon) = zip.by_name("pack.png") {
        if icon.size() > 0 && icon.size() <= 256 * 1024 {
            let mut buf = Vec::new();
            if icon.take(256 * 1024 + 1).read_to_end(&mut buf).is_ok() && buf.starts_with(b"\x89PNG\r\n\x1a\n") {
                let b64 = base64::engine::general_purpose::STANDARD.encode(&buf);
                icon_url = Some(format!("data:image/png;base64,{b64}"));
            }
        }
    }
    (desc, fmt, icon_url)
}

fn read_pack_metadata_from_dir(dir: &Path) -> (Option<String>, Option<i64>, Option<String>) {
    let (mut desc, mut fmt, mut icon_url) = (None, None, None);
    let mcmeta = dir.join("pack.mcmeta");
    if let Ok(meta) = std::fs::symlink_metadata(&mcmeta) {
        if meta.is_file() && !is_link(&meta) && meta.len() <= 128 * 1024 {
            if let Ok(text) = std::fs::read_to_string(&mcmeta) {
                if let Ok(v) = serde_json::from_str::<Value>(&text) {
                    fmt = v["pack"]["pack_format"].as_i64();
                    desc = extract_mc_text(&v["pack"]["description"]);
                }
            }
        }
    }
    let icon = dir.join("pack.png");
    if let Ok(meta) = std::fs::symlink_metadata(&icon) {
        if meta.is_file() && !is_link(&meta) && meta.len() > 0 && meta.len() <= 256 * 1024 {
            if let Ok(buf) = std::fs::read(&icon) {
                if buf.starts_with(b"\x89PNG\r\n\x1a\n") {
                    let b64 = base64::engine::general_purpose::STANDARD.encode(&buf);
                    icon_url = Some(format!("data:image/png;base64,{b64}"));
                }
            }
        }
    }
    (desc, fmt, icon_url)
}

fn extract_mc_text(val: &Value) -> Option<String> {
    if let Some(s) = val.as_str() {
        let cleaned = strip_mc_formatting(s);
        return (!cleaned.is_empty()).then_some(cleaned);
    }
    if let Some(obj) = val.as_object() {
        let mut parts = Vec::new();
        if let Some(t) = obj.get("text").and_then(Value::as_str) {
            parts.push(t.to_string());
        }
        if let Some(extra) = obj.get("extra").and_then(Value::as_array) {
            for item in extra {
                if let Some(sub) = extract_mc_text(item) {
                    parts.push(sub);
                }
            }
        }
        let joined = strip_mc_formatting(&parts.join(""));
        return (!joined.is_empty()).then_some(joined);
    }
    if let Some(arr) = val.as_array() {
        let joined = arr.iter().filter_map(extract_mc_text).collect::<Vec<_>>().join("");
        return (!joined.is_empty()).then_some(joined);
    }
    None
}

fn strip_mc_formatting(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '§' {
            let _ = chars.next();
            continue;
        }
        if !c.is_control() || c == ' ' {
            out.push(c);
        }
    }
    out.trim().chars().take(500).collect()
}

pub async fn list_packs(
    store: &Instances,
    files: &Files,
    id: &str,
    kind: &str,
    world: Option<&str>,
) -> Result<Vec<PackEntry>> {
    let folder = match kind {
        "resourcepack" => "resourcepacks".to_string(),
        "shader" => "shaderpacks".to_string(),
        "datapack" => {
            if let Some(w) = world.map(str::trim).filter(|s| !s.is_empty()) {
                relative_path(&format!("saves/{w}"))?;
                format!("saves/{w}/datapacks")
            } else {
                "datapacks".to_string()
            }
        }
        _ => bail!("Unsupported pack category"),
    };
    let abs_folder = store.game_dir(id).await?.join(&folder);
    if !tokio::fs::try_exists(&abs_folder).await? {
        return Ok(vec![]);
    }
    let entries = files.list(store, id, &folder).await?;
    let mut out = Vec::new();
    for entry in entries {
        let lower = entry.name.to_ascii_lowercase();
        let is_zip = lower.ends_with(".zip") || lower.ends_with(".zip.disabled");
        if !entry.directory && !is_zip {
            continue;
        }
        if entry.name.starts_with('.') {
            continue;
        }
        let enabled = if entry.directory {
            !lower.ends_with(".disabled")
        } else {
            lower.ends_with(".zip")
        };
        let path = files::resolve(store, id, &entry.path, false).await?;
        let is_dir = entry.directory;
        let (desc, fmt, icon_url) = tokio::task::spawn_blocking(move || {
            if is_dir {
                read_pack_metadata_from_dir(&path)
            } else {
                read_pack_metadata_from_zip(&path)
            }
        })
        .await?;
        let display_name = entry
            .name
            .trim_end_matches(".disabled")
            .trim_end_matches(".zip")
            .to_string();
        out.push(PackEntry {
            kind: kind.to_string(),
            path: entry.path,
            file_name: entry.name,
            name: display_name,
            description: desc,
            pack_format: fmt,
            icon_data_url: icon_url,
            enabled,
            directory: entry.directory,
            size: entry.size,
            world: world.map(String::from),
        });
    }
    out.sort_by_key(|p| p.name.to_lowercase());
    Ok(out)
}

pub async fn toggle_pack(
    store: &Instances,
    files: &Files,
    id: &str,
    path: &str,
    enabled: bool,
) -> Result<()> {
    relative_path(path)?;
    let valid_parent = path.starts_with("mods/")
        || path.starts_with("resourcepacks/")
        || path.starts_with("shaderpacks/")
        || path.starts_with("datapacks/")
        || (path.starts_with("saves/") && path.contains("/datapacks/"));
    if !valid_parent {
        bail!("Item is outside supported content folders");
    }
    let lower = path.to_ascii_lowercase();
    let target = if enabled && lower.ends_with(".disabled") {
        path[..path.len() - ".disabled".len()].to_string()
    } else if !enabled && !lower.ends_with(".disabled") {
        format!("{path}.disabled")
    } else {
        bail!("Item is already in the requested state");
    };
    files.rename(store, id, path, &target).await
}

pub async fn import_pack(
    store: &Instances,
    id: &str,
    kind: &str,
    world: Option<&str>,
    file_name: &str,
    bytes: &[u8],
) -> Result<PackEntry> {
    let trimmed = file_name.trim();
    if trimmed.len() <= 4
        || trimmed.starts_with('.')
        || trimmed.contains('/')
        || trimmed.contains('\\')
        || !trimmed.to_ascii_lowercase().ends_with(".zip")
    {
        bail!("Choose a valid .zip archive");
    }
    if bytes.is_empty() || bytes.len() > 256 * 1024 * 1024 {
        bail!("Choose a .zip file up to 256 MiB");
    }
    // Validate ZIP archive safely
    {
        let mut zip = zip::ZipArchive::new(Cursor::new(bytes))
            .context("Selected file is not a valid .zip archive")?;
        if zip.is_empty() || zip.len() > 20_000 {
            bail!("Archive is empty or contains too many entries");
        }
        for i in 0..zip.len() {
            let entry = zip.by_index_raw(i)?;
            let name = entry.name().replace('\\', "/");
            if name.starts_with('/')
                || name.contains(':')
                || name.contains('\0')
                || name.split('/').any(|seg| seg == "..")
            {
                bail!("Archive contains an unsafe entry path");
            }
        }
    }
    let folder = match kind {
        "resourcepack" => "resourcepacks".to_string(),
        "shader" => "shaderpacks".to_string(),
        "datapack" => {
            if let Some(w) = world.map(str::trim).filter(|s| !s.is_empty()) {
                relative_path(&format!("saves/{w}"))?;
                format!("saves/{w}/datapacks")
            } else {
                "datapacks".to_string()
            }
        }
        _ => bail!("Unsupported pack category"),
    };
    let rel = format!("{folder}/{trimmed}");
    let target = files::resolve_create_parents(store, id, &rel).await?;
    if tokio::fs::symlink_metadata(&target).await.is_ok() {
        bail!("A file with that name already exists");
    }
    let temp = target.with_file_name(format!(".solara-pack-{}.tmp", uuid::Uuid::new_v4()));
    {
        use tokio::io::AsyncWriteExt;
        let mut f = tokio::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)
            .await?;
        f.write_all(bytes).await?;
        f.sync_all().await?;
    }
    if let Err(e) = tokio::fs::rename(&temp, &target).await {
        let _ = tokio::fs::remove_file(&temp).await;
        return Err(e.into());
    }
    let target_clone = target.clone();
    let (desc, fmt, icon_url) =
        tokio::task::spawn_blocking(move || read_pack_metadata_from_zip(&target_clone)).await?;
    Ok(PackEntry {
        kind: kind.to_string(),
        path: rel,
        file_name: trimmed.to_string(),
        name: trimmed.trim_end_matches(".zip").to_string(),
        description: desc,
        pack_format: fmt,
        icon_data_url: icon_url,
        enabled: true,
        directory: false,
        size: bytes.len() as u64,
        world: world.map(String::from),
    })
}

// ---------------------------------------------------------------------------
// 2. Smart Mod & Content Update Checker + Safe Updater (Modrinth Hash API)
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContentUpdateItem {
    pub current_path: String,
    pub current_file_name: String,
    pub project_id: String,
    pub project_name: String,
    pub current_version_id: String,
    pub current_version_number: String,
    pub latest_version_id: String,
    pub latest_version_number: String,
    pub latest_version_name: String,
    pub release_channel: String,
    pub changelog: String,
    pub destination: String,
    pub download_url: String,
    pub sha1: String,
    pub sha512: String,
    pub size: u64,
}

fn sha1_file_hex(path: &Path) -> Result<String> {
    let mut f = std::fs::File::open(path)?;
    let mut hasher = sha1_smol::Sha1::new();
    let mut buf = vec![0u8; 65536];
    loop {
        let n = f.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hasher.digest().to_string())
}

pub async fn check_content_updates(store: &Instances, id: &str) -> Result<Vec<ContentUpdateItem>> {
    let instance = store.get(id).await?;
    let game_dir = store.game_dir(id).await?;
    let mut candidates: Vec<(String, String, PathBuf)> = Vec::new();
    for (folder, exts) in [
        ("mods", vec![".jar", ".jar.disabled"]),
        ("resourcepacks", vec![".zip", ".zip.disabled"]),
        ("shaderpacks", vec![".zip", ".zip.disabled"]),
    ] {
        let dir = game_dir.join(folder);
        if !tokio::fs::try_exists(&dir).await? {
            continue;
        }
        let mut rd = tokio::fs::read_dir(&dir).await?;
        while let Some(entry) = rd.next_entry().await? {
            if candidates.len() >= 300 {
                break;
            }
            let name = entry.file_name().to_string_lossy().to_string();
            let lower = name.to_ascii_lowercase();
            if exts.iter().any(|e| lower.ends_with(e)) {
                let meta = tokio::fs::symlink_metadata(entry.path()).await?;
                if meta.is_file() && !is_link(&meta) && meta.len() > 0 && meta.len() <= 256 * 1024 * 1024 {
                    candidates.push((folder.to_string(), format!("{folder}/{name}"), entry.path()));
                }
            }
        }
    }
    if candidates.is_empty() {
        return Ok(vec![]);
    }
    let hashed = tokio::task::spawn_blocking(move || {
        let mut map: HashMap<String, (String, String)> = HashMap::new();
        for (folder, rel, path) in candidates {
            if let Ok(h) = sha1_file_hex(&path) {
                map.insert(h, (folder, rel));
            }
        }
        map
    })
    .await?;
    if hashed.is_empty() {
        return Ok(vec![]);
    }
    let hashes: Vec<String> = hashed.keys().cloned().collect();
    let http = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(30))
        .user_agent("SolaraLauncher/independent-core")
        .build()?;

    let current_res = http
        .post("https://api.modrinth.com/v2/version_files")
        .json(&json!({ "hashes": hashes, "algorithm": "sha1" }))
        .send()
        .await
        .context("Content update service is unavailable")?;
    if !current_res.status().is_success() {
        bail!("Content provider returned {}", current_res.status().as_u16());
    }
    let current_map: Value = current_res.json().await?;
    let Some(current_obj) = current_map.as_object() else {
        return Ok(vec![]);
    };
    let matched_hashes: Vec<String> = current_obj.keys().cloned().collect();
    if matched_hashes.is_empty() {
        return Ok(vec![]);
    }

    let loaders = if matches!(instance.loader.as_str(), "fabric" | "quilt" | "forge" | "neoforge") {
        if instance.loader == "quilt" {
            vec!["quilt", "fabric"]
        } else {
            vec![instance.loader.as_str()]
        }
    } else {
        vec!["minecraft"]
    };

    let update_res = http
        .post("https://api.modrinth.com/v2/version_files/update")
        .json(&json!({
            "hashes": matched_hashes,
            "algorithm": "sha1",
            "loaders": loaders,
            "game_versions": [instance.game_version]
        }))
        .send()
        .await
        .context("Content update check failed")?;
    if !update_res.status().is_success() {
        bail!("Content provider returned {}", update_res.status().as_u16());
    }
    let latest_map: Value = update_res.json().await?;
    let Some(latest_obj) = latest_map.as_object() else {
        return Ok(vec![]);
    };

    let mut project_ids = HashSet::new();
    for (hash, latest_ver) in latest_obj {
        if let Some(cur_ver) = current_obj.get(hash) {
            if cur_ver["id"].as_str() != latest_ver["id"].as_str() {
                if let Some(pid) = latest_ver["project_id"].as_str() {
                    if content::provider_id(pid).is_ok() {
                        project_ids.insert(pid.to_string());
                    }
                }
            }
        }
    }
    let mut project_titles: HashMap<String, String> = HashMap::new();
    if !project_ids.is_empty() {
        let ids_vec: Vec<String> = project_ids.into_iter().collect();
        if let Ok(resp) = http
            .get("https://api.modrinth.com/v2/projects")
            .query(&[("ids", serde_json::to_string(&ids_vec)?)])
            .send()
            .await
        {
            if resp.status().is_success() {
                if let Ok(Value::Array(arr)) = resp.json::<Value>().await {
                    for p in arr {
                        if let (Some(pid), Some(title)) = (p["id"].as_str(), p["title"].as_str()) {
                            project_titles.insert(pid.to_string(), title.chars().take(160).collect());
                        }
                    }
                }
            }
        }
    }

    let mut updates = Vec::new();
    for (hash, latest_ver) in latest_obj {
        let Some(cur_ver) = current_obj.get(hash) else {
            continue;
        };
        let cur_vid = cur_ver["id"].as_str().unwrap_or("");
        let new_vid = latest_ver["id"].as_str().unwrap_or("");
        if cur_vid.is_empty() || new_vid.is_empty() || cur_vid == new_vid {
            continue;
        }
        let Some((folder, rel_path)) = hashed.get(hash) else {
            continue;
        };
        let Some(files_arr) = latest_ver["files"].as_array() else {
            continue;
        };
        let Some(chosen) = files_arr
            .iter()
            .find(|f| f["primary"].as_bool() == Some(true))
            .or_else(|| files_arr.first())
        else {
            continue;
        };
        let Some(filename) = chosen["filename"].as_str() else {
            continue;
        };
        if filename.contains('/') || filename.contains('\\') {
            continue;
        }
        let was_disabled = rel_path.to_ascii_lowercase().ends_with(".disabled");
        let dest = if was_disabled {
            format!("{folder}/{filename}.disabled")
        } else {
            format!("{folder}/{filename}")
        };
        if content::allowed_content_destination(&dest).is_err() {
            continue;
        }
        let Some(url) = chosen["url"].as_str().filter(|u| content::cdn_url(u).is_ok()) else {
            continue;
        };
        let Some(size) = chosen["size"].as_u64().filter(|n| *n > 0 && *n <= 512 * 1024 * 1024) else {
            continue;
        };
        let Some(sha1) = chosen["hashes"]["sha1"].as_str() else {
            continue;
        };
        let Some(sha512) = chosen["hashes"]["sha512"].as_str() else {
            continue;
        };
        let pid = latest_ver["project_id"].as_str().unwrap_or("").to_string();
        let project_name = project_titles
            .get(&pid)
            .cloned()
            .unwrap_or_else(|| pid.clone());
        updates.push(ContentUpdateItem {
            current_path: rel_path.clone(),
            current_file_name: Path::new(rel_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(rel_path)
                .to_string(),
            project_id: pid,
            project_name,
            current_version_id: cur_vid.to_string(),
            current_version_number: cur_ver["version_number"].as_str().unwrap_or("").to_string(),
            latest_version_id: new_vid.to_string(),
            latest_version_number: latest_ver["version_number"].as_str().unwrap_or("").to_string(),
            latest_version_name: latest_ver["name"].as_str().unwrap_or("").to_string(),
            release_channel: latest_ver["version_type"].as_str().unwrap_or("release").to_string(),
            changelog: latest_ver["changelog"]
                .as_str()
                .unwrap_or("")
                .chars()
                .take(4000)
                .collect(),
            destination: dest,
            download_url: url.to_string(),
            sha1: sha1.to_ascii_lowercase(),
            sha512: sha512.to_ascii_lowercase(),
            size,
        });
    }
    updates.sort_by_key(|u| u.project_name.to_lowercase());
    Ok(updates)
}

pub async fn apply_content_updates(
    store: &Instances,
    cache: &FileCache,
    id: &str,
    items: &[ContentUpdateItem],
    cancel: &Cancellation,
    mut report: impl FnMut(String, u64, u64) + Send,
) -> Result<()> {
    Box::pin(async move {
        let total: u64 = items.iter().map(|i| i.size).sum();
        let mut completed = 0u64;
        for (idx, item) in items.iter().enumerate() {
            cancel.check()?;
            content::allowed_content_destination(&item.destination)?;
            let artifact = Artifact {
                url: item.download_url.clone(),
                sha1: item.sha1.clone(),
                size: item.size,
                path: Some(item.destination.clone()),
            };
            artifact.validate()?;
            let phase = format!("Updating {} ({}/{})", item.project_name, idx + 1, items.len());
            let cached = cache
                .obtain(&artifact, cancel, |done, _| {
                    report(phase.clone(), completed.saturating_add(done), total);
                })
                .await?;
            if !content::verify_sha512(&cached, &item.sha512, item.size).await? {
                let _ = tokio::fs::remove_file(&cached).await;
                bail!("Updated file for {} failed SHA-512 verification", item.project_name);
            }
            let target_final = files::resolve_create_parents(store, id, &item.destination).await?;
            let temp = target_final.with_file_name(format!(".solara-update-{}.tmp", uuid::Uuid::new_v4()));
            tokio::fs::copy(&cached, &temp).await?;
            if let Err(e) = tokio::fs::rename(&temp, &target_final).await {
                let _ = tokio::fs::remove_file(&temp).await;
                return Err(e.into());
            }
            // Only remove previous file AFTER new verified file is safely in place
            if item.current_path != item.destination {
                if let Ok(old_path) = files::resolve(store, id, &item.current_path, false).await {
                    let _ = tokio::fs::remove_file(&old_path).await;
                }
            }
            completed = completed.saturating_add(item.size);
        }
        report("Updates applied".into(), total, total);
        Ok(())
    })
    .await
}

// ---------------------------------------------------------------------------
// 3. Worlds Manager & Pure-Rust Bounded NBT Header Reader
// ---------------------------------------------------------------------------

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldEntry {
    pub folder: String,
    pub name: String,
    pub game_mode: String,
    pub difficulty: String,
    pub version_name: String,
    pub hardcore: bool,
    pub last_played_ms: u64,
    pub size: u64,
    pub icon_data_url: Option<String>,
    pub locked: bool,
    pub datapack_count: usize,
}

fn dir_size_bounded(dir: &Path, depth: u8) -> u64 {
    if depth > 6 {
        return 0;
    }
    let Ok(rd) = std::fs::read_dir(dir) else {
        return 0;
    };
    let mut sum = 0u64;
    for entry in rd.flatten() {
        let Ok(meta) = std::fs::symlink_metadata(entry.path()) else {
            continue;
        };
        if is_link(&meta) {
            continue;
        }
        if meta.is_file() {
            sum = sum.saturating_add(meta.len());
        } else if meta.is_dir() {
            sum = sum.saturating_add(dir_size_bounded(&entry.path(), depth + 1));
        }
    }
    sum
}

#[derive(Default)]
pub struct NbtLevelSummary {
    level_name: Option<String>,
    game_type: Option<i32>,
    difficulty: Option<u8>,
    hardcore: bool,
    version_name: Option<String>,
    last_played_ms: Option<i64>,
}

struct NbtReader<'a> {
    buf: &'a [u8],
    pos: usize,
}
impl<'a> NbtReader<'a> {
    fn read_u8(&mut self) -> Option<u8> {
        let b = *self.buf.get(self.pos)?;
        self.pos += 1;
        Some(b)
    }
    fn read_u16(&mut self) -> Option<u16> {
        let bytes = self.buf.get(self.pos..self.pos + 2)?;
        self.pos += 2;
        Some(u16::from_be_bytes([bytes[0], bytes[1]]))
    }
    fn read_i32(&mut self) -> Option<i32> {
        let bytes = self.buf.get(self.pos..self.pos + 4)?;
        self.pos += 4;
        Some(i32::from_be_bytes(bytes.try_into().ok()?))
    }
    fn read_i64(&mut self) -> Option<i64> {
        let bytes = self.buf.get(self.pos..self.pos + 8)?;
        self.pos += 8;
        Some(i64::from_be_bytes(bytes.try_into().ok()?))
    }
    fn read_str(&mut self) -> Option<String> {
        let len = self.read_u16()? as usize;
        let bytes = self.buf.get(self.pos..self.pos + len)?;
        self.pos += len;
        Some(String::from_utf8_lossy(bytes).to_string())
    }
    fn skip(&mut self, n: usize) -> Option<()> {
        if self.pos.checked_add(n)? > self.buf.len() {
            return None;
        }
        self.pos += n;
        Some(())
    }
    fn skip_payload(&mut self, tag: u8, depth: u8) -> Option<()> {
        if depth > 16 {
            return None;
        }
        match tag {
            0 => Some(()),
            1 => self.skip(1),
            2 => self.skip(2),
            3 | 5 => self.skip(4),
            4 | 6 => self.skip(8),
            7 => {
                let len = self.read_i32()?.max(0) as usize;
                self.skip(len)
            }
            8 => {
                let len = self.read_u16()? as usize;
                self.skip(len)
            }
            9 => {
                let item_tag = self.read_u8()?;
                let len = self.read_i32()?.max(0) as usize;
                for _ in 0..len.min(100_000) {
                    self.skip_payload(item_tag, depth + 1)?;
                }
                Some(())
            }
            10 => {
                loop {
                    let t = self.read_u8()?;
                    if t == 0 {
                        break;
                    }
                    let _ = self.read_str()?;
                    self.skip_payload(t, depth + 1)?;
                }
                Some(())
            }
            11 => {
                let len = self.read_i32()?.max(0) as usize;
                self.skip(len.checked_mul(4)?)
            }
            12 => {
                let len = self.read_i32()?.max(0) as usize;
                self.skip(len.checked_mul(8)?)
            }
            _ => None,
        }
    }
}

pub fn parse_level_dat_bytes(compressed: &[u8]) -> Option<NbtLevelSummary> {
    let mut decoder = GzDecoder::new(compressed);
    let mut raw = Vec::new();
    decoder.by_ref().take(2 * 1024 * 1024).read_to_end(&mut raw).ok()?;
    let mut r = NbtReader { buf: &raw, pos: 0 };
    if r.read_u8()? != 10 {
        return None;
    }
    let _root_name = r.read_str()?;
    let mut summary = NbtLevelSummary::default();
    while let Some(tag) = r.read_u8() {
        if tag == 0 {
            break;
        }
        let name = r.read_str()?;
        if tag == 10 && name == "Data" {
            while let Some(sub_tag) = r.read_u8() {
                if sub_tag == 0 {
                    break;
                }
                let sub_name = r.read_str()?;
                match (sub_tag, sub_name.as_str()) {
                    (8, "LevelName") => summary.level_name = r.read_str(),
                    (3, "GameType") => summary.game_type = r.read_i32(),
                    (1, "Difficulty") => summary.difficulty = r.read_u8(),
                    (1, "hardcore") => summary.hardcore = r.read_u8()? != 0,
                    (4, "LastPlayed") => summary.last_played_ms = r.read_i64(),
                    (10, "Version") => {
                        while let Some(vtag) = r.read_u8() {
                            if vtag == 0 {
                                break;
                            }
                            let vname = r.read_str()?;
                            if vtag == 8 && vname == "Name" {
                                summary.version_name = r.read_str();
                            } else {
                                r.skip_payload(vtag, 2)?;
                            }
                        }
                    }
                    _ => r.skip_payload(sub_tag, 1)?,
                }
            }
            break;
        } else {
            r.skip_payload(tag, 0)?;
        }
    }
    Some(summary)
}

pub async fn list_worlds(store: &Instances, id: &str, running: bool) -> Result<Vec<WorldEntry>> {
    let saves_dir = store.game_dir(id).await?.join("saves");
    if !tokio::fs::try_exists(&saves_dir).await? {
        return Ok(vec![]);
    }
    let mut rd = tokio::fs::read_dir(&saves_dir).await?;
    let mut out = Vec::new();
    while let Some(entry) = rd.next_entry().await? {
        if out.len() >= 200 {
            break;
        }
        let meta = tokio::fs::symlink_metadata(entry.path()).await?;
        if is_link(&meta) || !meta.is_dir() {
            continue;
        }
        let folder_name = entry.file_name().to_string_lossy().to_string();
        if folder_name.starts_with('.') {
            continue;
        }
        let world_dir = entry.path();
        let item = tokio::task::spawn_blocking(move || -> WorldEntry {
            let level_path = world_dir.join("level.dat");
            let summary = std::fs::read(&level_path)
                .ok()
                .and_then(|b| parse_level_dat_bytes(&b))
                .unwrap_or_default();
            let icon_path = world_dir.join("icon.png");
            let icon_data_url = std::fs::symlink_metadata(&icon_path)
                .ok()
                .filter(|m| m.is_file() && !is_link(m) && m.len() <= 512 * 1024)
                .and_then(|_| std::fs::read(&icon_path).ok())
                .filter(|b| b.starts_with(b"\x89PNG\r\n\x1a\n"))
                .map(|b| format!("data:image/png;base64,{}", base64::engine::general_purpose::STANDARD.encode(&b)));
            let game_mode = match (summary.hardcore, summary.game_type) {
                (true, Some(0)) => "Hardcore",
                (_, Some(0)) => "Survival",
                (_, Some(1)) => "Creative",
                (_, Some(2)) => "Adventure",
                (_, Some(3)) => "Spectator",
                _ => "Unknown",
            }
            .to_string();
            let difficulty = match summary.difficulty {
                Some(0) => "Peaceful",
                Some(1) => "Easy",
                Some(2) => "Normal",
                Some(3) => "Hard",
                _ => "Unknown",
            }
            .to_string();
            let fallback_modified_ms = meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            let last_played_ms = summary
                .last_played_ms
                .filter(|ms| *ms > 0)
                .map(|ms| ms as u64)
                .unwrap_or(fallback_modified_ms);
            let datapack_count = std::fs::read_dir(world_dir.join("datapacks"))
                .map(|rd| {
                    rd.flatten()
                        .filter(|e| {
                            let name = e.file_name().to_string_lossy().to_string();
                            if name.starts_with('.') {
                                return false;
                            }
                            std::fs::symlink_metadata(e.path())
                                .ok()
                                .filter(|m| !is_link(m))
                                .is_some_and(|m| m.is_dir() || name.to_ascii_lowercase().ends_with(".zip"))
                        })
                        .count()
                })
                .unwrap_or(0);
            WorldEntry {
                name: summary
                    .level_name
                    .filter(|s| !s.trim().is_empty())
                    .unwrap_or_else(|| folder_name.clone()),
                folder: folder_name,
                game_mode,
                difficulty,
                version_name: summary.version_name.unwrap_or_default(),
                hardcore: summary.hardcore,
                last_played_ms,
                size: dir_size_bounded(&world_dir, 0),
                icon_data_url,
                locked: running,
                datapack_count,
            }
        })
        .await?;
        out.push(item);
    }
    out.sort_by(|a, b| b.last_played_ms.cmp(&a.last_played_ms).then(a.name.cmp(&b.name)));
    Ok(out)
}

pub async fn rename_world(
    store: &Instances,
    files: &Files,
    id: &str,
    old_folder: &str,
    new_folder: &str,
) -> Result<()> {
    let clean_new = new_folder.trim();
    if clean_new.is_empty() || clean_new.contains('/') || clean_new.contains('\\') {
        bail!("Enter a valid world folder name");
    }
    let from = format!("saves/{old_folder}");
    let to = format!("saves/{clean_new}");
    relative_path(&from)?;
    relative_path(&to)?;
    files.rename(store, id, &from, &to).await
}

// ---------------------------------------------------------------------------
// 4. Screenshots & Disk Logs
// ---------------------------------------------------------------------------

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScreenshotEntry {
    pub instance_id: String,
    pub instance_name: String,
    pub file_name: String,
    pub path: String,
    pub full_path: String,
    pub size: u64,
    pub modified_ms: u64,
}

fn screenshot_mime(name: &str) -> Option<&'static str> {
    let lower = name.to_ascii_lowercase();
    if lower.ends_with(".png") {
        Some("image/png")
    } else if lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
        Some("image/jpeg")
    } else if lower.ends_with(".webp") {
        Some("image/webp")
    } else {
        None
    }
}

pub async fn list_screenshots(store: &Instances, instance_id: Option<&str>) -> Result<Vec<ScreenshotEntry>> {
    let instances = if let Some(id) = instance_id {
        vec![store.get(id).await?]
    } else {
        store.list().await?
    };
    let mut out = Vec::new();
    for inst in instances {
        let dir = store.game_dir(&inst.id).await?.join("screenshots");
        let Ok(dir_meta) = tokio::fs::symlink_metadata(&dir).await else {
            continue;
        };
        if is_link(&dir_meta) || !dir_meta.is_dir() {
            continue;
        }
        let mut rd = tokio::fs::read_dir(&dir).await?;
        while let Some(entry) = rd.next_entry().await? {
            if out.len() >= 1000 {
                break;
            }
            let entry_path = entry.path();
            let meta = tokio::fs::symlink_metadata(&entry_path).await?;
            if is_link(&meta) || !meta.is_file() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().to_string();
            if screenshot_mime(&name).is_none() {
                continue;
            }
            let modified_ms = meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            out.push(ScreenshotEntry {
                instance_id: inst.id.clone(),
                instance_name: inst.name.clone(),
                path: format!("screenshots/{name}"),
                full_path: entry_path.to_string_lossy().to_string(),
                file_name: name,
                size: meta.len(),
                modified_ms,
            });
        }
    }
    out.sort_by(|a, b| b.modified_ms.cmp(&a.modified_ms));
    Ok(out)
}

pub async fn read_screenshot_image(
    store: &Instances,
    instance_id: &str,
    rel_path: &str,
) -> Result<String> {
    let normalized = rel_path.trim().replace('\\', "/");
    if !normalized.starts_with("screenshots/") {
        bail!("Screenshot path must stay inside the screenshots folder");
    }
    let mime = screenshot_mime(&normalized).context("Unsupported screenshot image format")?;
    let target = files::resolve(store, instance_id, &normalized, false).await?;
    let meta = tokio::fs::symlink_metadata(&target)
        .await
        .context("Screenshot file not found")?;
    if is_link(&meta) || !meta.is_file() {
        bail!("Invalid screenshot file");
    }
    if meta.len() > 24 * 1024 * 1024 {
        bail!("Screenshot exceeds the 24 MB preview limit");
    }
    let bytes = tokio::fs::read(&target).await?;
    Ok(format!(
        "data:{mime};base64,{}",
        base64::engine::general_purpose::STANDARD.encode(&bytes)
    ))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogFileEntry {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub modified_ms: u64,
    pub gzipped: bool,
}

pub async fn list_disk_logs(store: &Instances, id: &str) -> Result<Vec<LogFileEntry>> {
    let dir = store.game_dir(id).await?.join("logs");
    if !tokio::fs::try_exists(&dir).await? {
        return Ok(vec![]);
    }
    let mut rd = tokio::fs::read_dir(&dir).await?;
    let mut out = Vec::new();
    while let Some(entry) = rd.next_entry().await? {
        let meta = tokio::fs::symlink_metadata(entry.path()).await?;
        if is_link(&meta) || !meta.is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if !(name.ends_with(".log") || name.ends_with(".log.gz") || name.ends_with(".txt")) {
            continue;
        }
        let modified_ms = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        let gzipped = name.ends_with(".gz");
        out.push(LogFileEntry {
            path: format!("logs/{name}"),
            name,
            size: meta.len(),
            modified_ms,
            gzipped,
        });
    }
    out.sort_by(|a, b| {
        (b.name == "latest.log")
            .cmp(&(a.name == "latest.log"))
            .then(b.modified_ms.cmp(&a.modified_ms))
    });
    Ok(out)
}

pub async fn read_disk_log(store: &Instances, id: &str, name: &str) -> Result<String> {
    let clean = name.trim();
    if clean.contains('/') || clean.contains('\\') || clean.is_empty() {
        bail!("Invalid log file name");
    }
    let path = files::resolve(store, id, &format!("logs/{clean}"), false).await?;
    let meta = tokio::fs::symlink_metadata(&path).await?;
    if !meta.is_file() || is_link(&meta) || meta.len() > 16 * 1024 * 1024 {
        bail!("Log file exceeds size limit");
    }
    let bytes = tokio::fs::read(&path).await?;
    if clean.ends_with(".gz") {
        let out = tokio::task::spawn_blocking(move || -> Result<String> {
            let mut dec = GzDecoder::new(Cursor::new(bytes));
            let mut buf = Vec::new();
            dec.by_ref().take(2 * 1024 * 1024).read_to_end(&mut buf)?;
            Ok(String::from_utf8_lossy(&buf).to_string())
        })
        .await??;
        Ok(out)
    } else {
        let slice = if bytes.len() > 2 * 1024 * 1024 {
            &bytes[bytes.len() - 2 * 1024 * 1024..]
        } else {
            &bytes[..]
        };
        Ok(String::from_utf8_lossy(slice).to_string())
    }
}

// ---------------------------------------------------------------------------
// 5. Java Runtime Detection & Storage Summary
// ---------------------------------------------------------------------------

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectedJava {
    pub path: String,
    pub version: String,
    pub major: u32,
    pub vendor: String,
    pub managed: bool,
}

async fn probe_java(path: &Path, managed: bool) -> Option<DetectedJava> {
    let mut cmd = Command::new(path);
    cmd.arg("-version")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    #[cfg(windows)]
    cmd.creation_flags(0x08000000);
    let output = tokio::time::timeout(std::time::Duration::from_secs(4), cmd.output())
        .await
        .ok()?
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let re = regex::Regex::new(r#"(?m)^(?:openjdk|java) (?:version )?"([^"]+)""#).ok()?;
    let ver_str = re.captures(&text)?.get(1)?.as_str().to_string();
    let mut parts = ver_str.split(['.', '_', '-', '+']);
    let first: u32 = parts.next()?.parse().ok()?;
    let major = if first == 1 {
        parts.next().and_then(|s| s.parse().ok()).unwrap_or(8)
    } else {
        first
    };
    let vendor = if text.contains("Temurin") || text.contains("Adoptium") {
        "Eclipse Temurin"
    } else if text.contains("Microsoft") {
        "Microsoft Build of OpenJDK"
    } else if text.contains("Zulu") || text.contains("Azul") {
        "Azul Zulu"
    } else if text.contains("Corretto") {
        "Amazon Corretto"
    } else if text.contains("OpenJDK") {
        "OpenJDK"
    } else {
        "Oracle / Java"
    }
    .to_string();
    Some(DetectedJava {
        path: path.to_string_lossy().to_string(),
        version: ver_str,
        major,
        vendor,
        managed,
    })
}

pub async fn detect_java_runtimes(managed_root: &Path) -> Vec<DetectedJava> {
    let mut candidates: Vec<(PathBuf, bool)> = Vec::new();
    let exe_name = if cfg!(windows) { "java.exe" } else { "java" };

    if let Ok(mut rd) = tokio::fs::read_dir(managed_root).await {
        while let Ok(Some(e)) = rd.next_entry().await {
            let p = e.path().join("bin").join(exe_name);
            if p.is_file() {
                candidates.push((p, true));
            }
        }
    }

    if let Ok(path_var) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path_var) {
            let p = dir.join(exe_name);
            if p.is_file() {
                candidates.push((p, false));
            }
        }
    }

    #[cfg(windows)]
    let standard_roots = [
        r"C:\Program Files\Java",
        r"C:\Program Files\Eclipse Adoptium",
        r"C:\Program Files\Microsoft",
        r"C:\Program Files\Zulu",
        r"C:\Program Files\Amazon Corretto",
    ];
    #[cfg(not(windows))]
    let standard_roots = ["/usr/lib/jvm", "/opt/java"];

    for root in standard_roots {
        if let Ok(mut rd) = tokio::fs::read_dir(root).await {
            while let Ok(Some(e)) = rd.next_entry().await {
                let p = e.path().join("bin").join(exe_name);
                if p.is_file() {
                    candidates.push((p, false));
                }
            }
        }
    }

    let mut seen = HashSet::new();
    let mut found = Vec::new();
    for (p, managed) in candidates {
        let canonical = std::fs::canonicalize(&p).unwrap_or(p);
        if !seen.insert(canonical.clone()) {
            continue;
        }
        if let Some(info) = probe_java(&canonical, managed).await {
            found.push(info);
        }
    }
    found.sort_by(|a, b| b.managed.cmp(&a.managed).then(b.major.cmp(&a.major)));
    found
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageSummary {
    pub data_dir: String,
    pub instances_dir: String,
    pub total_bytes: u64,
    pub cache_bytes: u64,
    pub instances_bytes: u64,
    pub java_bytes: u64,
    pub skins_bytes: u64,
    pub backups_bytes: u64,
}

pub async fn storage_summary(data_dir: &Path) -> StorageSummary {
    let data = data_dir.to_path_buf();
    tokio::task::spawn_blocking(move || {
        let instances_dir = data.join("custom-instances");
        let cache_dir = data.join("verified-game-files");
        let java_dir = data.join("java-runtimes");
        let skins_dir = data.join("skins");
        let mut backups_bytes = 0u64;
        if let Ok(rd) = std::fs::read_dir(&instances_dir) {
            for e in rd.flatten() {
                let b = e.path().join("backups");
                if b.is_dir() {
                    backups_bytes = backups_bytes.saturating_add(dir_size_bounded(&b, 0));
                }
            }
        }
        let cache_bytes = dir_size_bounded(&cache_dir, 0);
        let instances_bytes = dir_size_bounded(&instances_dir, 0);
        let java_bytes = dir_size_bounded(&java_dir, 0);
        let skins_bytes = dir_size_bounded(&skins_dir, 0);
        let total_bytes = instances_bytes
            .saturating_add(cache_bytes)
            .saturating_add(java_bytes)
            .saturating_add(skins_bytes);
        StorageSummary {
            data_dir: data.to_string_lossy().to_string(),
            instances_dir: instances_dir.to_string_lossy().to_string(),
            total_bytes,
            cache_bytes,
            instances_bytes,
            java_bytes,
            skins_bytes,
            backups_bytes,
        }
    })
    .await
    .unwrap_or_else(|_| StorageSummary {
        data_dir: data_dir.to_string_lossy().to_string(),
        instances_dir: data_dir.join("custom-instances").to_string_lossy().to_string(),
        total_bytes: 0,
        cache_bytes: 0,
        instances_bytes: 0,
        java_bytes: 0,
        skins_bytes: 0,
        backups_bytes: 0,
    })
}

pub async fn clear_unused_cache(data_dir: &Path) -> Result<u64> {
    let cache_dir = data_dir.join("verified-game-files");
    if !tokio::fs::try_exists(&cache_dir).await? {
        return Ok(0);
    }
    let cleared = tokio::task::spawn_blocking(move || -> u64 {
        let size = dir_size_bounded(&cache_dir, 0);
        let _ = std::fs::remove_dir_all(&cache_dir);
        let _ = std::fs::create_dir_all(&cache_dir);
        size
    })
    .await?;
    Ok(cleared)
}

// ---------------------------------------------------------------------------
// 6. Minecraft Profile Skins & Capes + Saved Skins Library
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SavedSkin {
    pub id: String,
    pub name: String,
    pub variant: String,
    #[serde(alias = "dataUrl")]
    pub png_data_url: String,
    #[serde(alias = "createdAt")]
    pub updated_at: String,
}

pub fn validate_minecraft_skin_png(bytes: &[u8]) -> Result<(u32, u32)> {
    if bytes.len() < 24 || bytes.len() > 512 * 1024 || &bytes[0..8] != b"\x89PNG\r\n\x1a\n" {
        bail!("Skin must be a valid PNG file smaller than 512 KiB");
    }
    if &bytes[12..16] != b"IHDR" {
        bail!("Invalid PNG header in skin texture");
    }
    let w = u32::from_be_bytes(bytes[16..20].try_into()?);
    let h = u32::from_be_bytes(bytes[20..24].try_into()?);
    if !((w == 64 && h == 64) || (w == 64 && h == 32)) {
        bail!("Minecraft skins must be 64x64 or 64x32 pixels (got {w}x{h})");
    }
    Ok((w, h))
}

fn decode_skin_png_input(input: &str) -> Result<Vec<u8>> {
    let b64 = input
        .trim()
        .strip_prefix("data:image/png;base64,")
        .unwrap_or(input.trim());
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(b64)
        .context("Could not decode PNG skin bytes")?;
    validate_minecraft_skin_png(&bytes)?;
    Ok(bytes)
}

pub async fn saved_skins_list(root: &Path) -> Result<Vec<SavedSkin>> {
    let index_path = root.join("index.json");
    if !tokio::fs::try_exists(&index_path).await? {
        return Ok(vec![]);
    }
    let bytes = tokio::fs::read(&index_path).await?;
    let mut items: Vec<SavedSkin> = serde_json::from_slice(&bytes)?;
    items.truncate(200);
    Ok(items)
}

pub async fn saved_skins_save(
    root: &Path,
    id: Option<&str>,
    name: &str,
    variant: &str,
    png_data_url: &str,
) -> Result<SavedSkin> {
    let clean_name = name.trim();
    if clean_name.is_empty() || clean_name.chars().count() > 60 || clean_name.chars().any(char::is_control) {
        bail!("Choose a skin name up to 60 characters");
    }
    if !matches!(variant, "classic" | "slim") {
        bail!("Skin model must be classic or slim");
    }
    let bytes = decode_skin_png_input(png_data_url)?;
    let normalized = format!("data:image/png;base64,{}", base64::engine::general_purpose::STANDARD.encode(&bytes));
    tokio::fs::create_dir_all(root).await?;
    let mut list = saved_skins_list(root).await.unwrap_or_default();
    let entry = if let Some(existing_id) = id.filter(|s| !s.is_empty()) {
        if let Some(found) = list.iter_mut().find(|s| s.id == existing_id) {
            found.name = clean_name.to_string();
            found.variant = variant.to_string();
            found.png_data_url = normalized;
            found.updated_at = chrono::Utc::now().to_rfc3339();
            found.clone()
        } else {
            let item = SavedSkin {
                id: uuid::Uuid::new_v4().to_string(),
                name: clean_name.to_string(),
                variant: variant.to_string(),
                png_data_url: normalized,
                updated_at: chrono::Utc::now().to_rfc3339(),
            };
            list.insert(0, item.clone());
            item
        }
    } else {
        let item = SavedSkin {
            id: uuid::Uuid::new_v4().to_string(),
            name: clean_name.to_string(),
            variant: variant.to_string(),
            png_data_url: normalized,
            updated_at: chrono::Utc::now().to_rfc3339(),
        };
        list.insert(0, item.clone());
        item
    };
    write_json(&root.join("index.json"), &list).await?;
    Ok(entry)
}

pub async fn saved_skins_delete(root: &Path, id: &str) -> Result<()> {
    let mut list = saved_skins_list(root).await.unwrap_or_default();
    list.retain(|s| s.id != id);
    tokio::fs::create_dir_all(root).await?;
    write_json(&root.join("index.json"), &list).await
}

async fn fetch_minecraft_texture_data_url(url_str: &str) -> Option<String> {
    let url = reqwest::Url::parse(url_str).ok()?;
    if url.scheme() != "https" || url.host_str() != Some("textures.minecraft.net") {
        return None;
    }
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .ok()?;
    let resp = client.get(url).send().await.ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let bytes = resp.bytes().await.ok()?;
    if bytes.len() > 512 * 1024 || !bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        return None;
    }
    Some(format!(
        "data:image/png;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(&bytes)
    ))
}

pub async fn fetch_profile_skins_and_capes(session: &Session) -> Result<Value> {
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(20))
        .build()?;
    let resp = client
        .get("https://api.minecraftservices.com/minecraft/profile")
        .bearer_auth(&session.access_token)
        .send()
        .await
        .context("Minecraft profile service is unavailable")?;
    if !resp.status().is_success() {
        bail!("Could not load Minecraft profile skins and capes");
    }
    let mut profile: Value = resp.json().await?;
    if let Some(skins) = profile["skins"].as_array_mut() {
        for skin in skins {
            if let Some(url) = skin["url"].as_str() {
                if let Some(data_url) = fetch_minecraft_texture_data_url(url).await {
                    skin["dataUrl"] = json!(data_url);
                }
            }
        }
    }
    if let Some(capes) = profile["capes"].as_array_mut() {
        for cape in capes {
            if let Some(url) = cape["url"].as_str() {
                if let Some(data_url) = fetch_minecraft_texture_data_url(url).await {
                    cape["dataUrl"] = json!(data_url);
                }
            }
        }
    }
    Ok(profile)
}

pub async fn upload_minecraft_skin(session: &Session, variant: &str, png_data_url: &str) -> Result<Value> {
    if !matches!(variant, "classic" | "slim") {
        bail!("Skin variant must be classic or slim");
    }
    let bytes = decode_skin_png_input(png_data_url)?;
    // Construct multipart/form-data cleanly without adding extra crates
    let boundary = format!("----SolaraSkinBoundary{}", uuid::Uuid::new_v4().simple());
    let mut body = Vec::with_capacity(bytes.len() + 512);
    use std::io::Write;
    write!(
        body,
        "--{boundary}\r\nContent-Disposition: form-data; name=\"variant\"\r\n\r\n{variant}\r\n--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"skin.png\"\r\nContent-Type: image/png\r\n\r\n"
    )?;
    body.extend_from_slice(&bytes);
    write!(body, "\r\n--{boundary}--\r\n")?;

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(30))
        .build()?;
    let resp = client
        .post("https://api.minecraftservices.com/minecraft/profile/skins")
        .bearer_auth(&session.access_token)
        .header("Content-Type", format!("multipart/form-data; boundary={boundary}"))
        .body(body)
        .send()
        .await
        .context("Minecraft skin upload service is unavailable")?;
    if !resp.status().is_success() {
        bail!("Minecraft rejected the skin upload ({})", resp.status().as_u16());
    }
    Ok(resp.json().await.unwrap_or_else(|_| json!({ "ok": true })))
}

pub async fn set_minecraft_cape(session: &Session, cape_id: Option<&str>) -> Result<()> {
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(20))
        .build()?;
    let resp = match cape_id.map(str::trim).filter(|s| !s.is_empty()) {
        Some(id) => {
            uuid::Uuid::parse_str(id).context("Invalid cape identifier")?;
            client
                .put("https://api.minecraftservices.com/minecraft/profile/capes/active")
                .bearer_auth(&session.access_token)
                .json(&json!({ "capeId": id }))
                .send()
                .await
        }
        None => {
            client
                .delete("https://api.minecraftservices.com/minecraft/profile/capes/active")
                .bearer_auth(&session.access_token)
                .send()
                .await
        }
    }
    .context("Minecraft cape service is unavailable")?;
    if !resp.status().is_success() {
        bail!("Minecraft rejected the cape change ({})", resp.status().as_u16());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_skin_png_dimensions_and_rejects_invalid_bytes() {
        assert!(validate_minecraft_skin_png(b"not-a-png").is_err());
        let mut hdr = Vec::new();
        hdr.extend_from_slice(b"\x89PNG\r\n\x1a\n");
        hdr.extend_from_slice(&13u32.to_be_bytes());
        hdr.extend_from_slice(b"IHDR");
        hdr.extend_from_slice(&64u32.to_be_bytes());
        hdr.extend_from_slice(&64u32.to_be_bytes());
        hdr.extend_from_slice(&[8, 6, 0, 0, 0]);
        assert_eq!(validate_minecraft_skin_png(&hdr).unwrap(), (64, 64));

        // 128x128 invalid for standard skin
        let mut bad = hdr.clone();
        bad[16..20].copy_from_slice(&128u32.to_be_bytes());
        bad[20..24].copy_from_slice(&128u32.to_be_bytes());
        assert!(validate_minecraft_skin_png(&bad).is_err());
    }

    #[test]
    fn parses_gzipped_nbt_level_dat_summary() {
        use flate2::{write::GzEncoder, Compression};
        use std::io::Write;
        // Build minimal NBT: Compound("") -> Compound("Data") -> String("LevelName","Solara"), Int("GameType",1), Byte("hardcore",0)
        let mut nbt = Vec::new();
        nbt.push(10); // TAG_Compound
        nbt.extend_from_slice(&0u16.to_be_bytes()); // root name ""
        nbt.push(10); // TAG_Compound
        nbt.extend_from_slice(&4u16.to_be_bytes());
        nbt.extend_from_slice(b"Data");
        nbt.push(8); // TAG_String
        nbt.extend_from_slice(&9u16.to_be_bytes());
        nbt.extend_from_slice(b"LevelName");
        nbt.extend_from_slice(&6u16.to_be_bytes());
        nbt.extend_from_slice(b"Solara");
        nbt.push(3); // TAG_Int
        nbt.extend_from_slice(&8u16.to_be_bytes());
        nbt.extend_from_slice(b"GameType");
        nbt.extend_from_slice(&1i32.to_be_bytes());
        nbt.push(1); // TAG_Byte
        nbt.extend_from_slice(&10u16.to_be_bytes());
        nbt.extend_from_slice(b"Difficulty");
        nbt.push(3); // Hard
        nbt.push(0); // End Data
        nbt.push(0); // End Root

        let mut enc = GzEncoder::new(Vec::new(), Compression::default());
        enc.write_all(&nbt).unwrap();
        let gz = enc.finish().unwrap();

        let parsed = parse_level_dat_bytes(&gz).unwrap();
        assert_eq!(parsed.level_name.as_deref(), Some("Solara"));
        assert_eq!(parsed.game_type, Some(1));
        assert_eq!(parsed.difficulty, Some(3));
        assert!(!parsed.hardcore);
    }

    #[tokio::test]
    async fn saved_skins_library_persists_and_deletes_safely() {
        let root = std::env::temp_dir().join(format!("solara-skins-test-{}", uuid::Uuid::new_v4()));
        let mut png = Vec::new();
        png.extend_from_slice(b"\x89PNG\r\n\x1a\n");
        png.extend_from_slice(&13u32.to_be_bytes());
        png.extend_from_slice(b"IHDR");
        png.extend_from_slice(&64u32.to_be_bytes());
        png.extend_from_slice(&64u32.to_be_bytes());
        png.extend_from_slice(&[8, 6, 0, 0, 0]);
        let data_url = format!("data:image/png;base64,{}", base64::engine::general_purpose::STANDARD.encode(&png));

        let saved = saved_skins_save(&root, None, "Dawn Explorer", "slim", &data_url)
            .await
            .unwrap();
        assert_eq!(saved.name, "Dawn Explorer");
        assert_eq!(saved.variant, "slim");
        let serialized = serde_json::to_value(&saved).unwrap();
        assert!(serialized.get("pngDataUrl").is_some());
        assert!(serialized.get("updatedAt").is_some());

        let legacy_json = serde_json::json!([{
            "id": "legacy-1",
            "name": "Legacy",
            "variant": "classic",
            "dataUrl": data_url,
            "createdAt": "2026-01-01T00:00:00Z"
        }]);
        tokio::fs::write(root.join("index.json"), serde_json::to_vec(&legacy_json).unwrap())
            .await
            .unwrap();
        let legacy_list = saved_skins_list(&root).await.unwrap();
        assert_eq!(legacy_list.len(), 1);
        assert_eq!(legacy_list[0].name, "Legacy");
        assert!(legacy_list[0].png_data_url.starts_with("data:image/png;base64,"));

        let saved2 = saved_skins_save(&root, None, "Dawn Two", "classic", &data_url)
            .await
            .unwrap();
        saved_skins_delete(&root, &saved2.id).await.unwrap();
        saved_skins_delete(&root, "legacy-1").await.unwrap();
        assert!(saved_skins_list(&root).await.unwrap().is_empty());
        let _ = tokio::fs::remove_dir_all(root).await;
    }

    #[tokio::test]
    async fn storage_summary_reports_total_and_skins_bytes() {
        let root = std::env::temp_dir().join(format!("solara-storage-test-{}", uuid::Uuid::new_v4()));
        tokio::fs::create_dir_all(root.join("skins")).await.unwrap();
        tokio::fs::write(root.join("skins").join("index.json"), vec![0u8; 128])
            .await
            .unwrap();
        let summary = storage_summary(&root).await;
        assert_eq!(summary.skins_bytes, 128);
        assert_eq!(summary.total_bytes, 128);
        let _ = tokio::fs::remove_dir_all(root).await;
    }

    #[tokio::test]
    async fn list_screenshots_returns_metadata_only_and_reads_on_demand() {
        let root = std::env::temp_dir().join(format!("solara-shots-test-{}", uuid::Uuid::new_v4()));
        let store = Instances::new(root.clone());
        let inst = store
            .create(
                "Shot Test".into(),
                "1.21.1".into(),
                "vanilla".into(),
                None,
                4096,
                "auto".into(),
            )
            .await
            .unwrap();
        let shots_dir = store.game_dir(&inst.id).await.unwrap().join("screenshots");
        tokio::fs::create_dir_all(&shots_dir).await.unwrap();
        tokio::fs::write(shots_dir.join("2026-01-01.png"), b"\x89PNG\r\n\x1a\ntest")
            .await
            .unwrap();

        let entries = list_screenshots(&store, Some(&inst.id)).await.unwrap();
        assert_eq!(entries.len(), 1);
        let serialized = serde_json::to_value(&entries[0]).unwrap();
        assert!(serialized.get("dataUrl").is_none());
        assert!(serialized
            .get("fullPath")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .ends_with("2026-01-01.png"));

        let data_url = read_screenshot_image(&store, &inst.id, "screenshots/2026-01-01.png")
            .await
            .unwrap();
        assert!(data_url.starts_with("data:image/png;base64,"));
        assert!(read_screenshot_image(&store, &inst.id, "../instance.json")
            .await
            .is_err());

        #[cfg(unix)]
        {
            let outside = root.join("outside.png");
            tokio::fs::write(&outside, b"\x89PNG\r\n\x1a\nsecret").await.unwrap();
            std::os::unix::fs::symlink(&outside, shots_dir.join("linked.png")).unwrap();
            let after_link = list_screenshots(&store, Some(&inst.id)).await.unwrap();
            assert_eq!(after_link.len(), 1);
            assert!(read_screenshot_image(&store, &inst.id, "screenshots/linked.png")
                .await
                .is_err());
        }

        let _ = tokio::fs::remove_dir_all(&root).await;
    }
}
