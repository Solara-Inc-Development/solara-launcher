use crate::update_policy;
use anyhow::{Context, anyhow, bail};
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use minisign_verify::{PublicKey, Signature};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::net::{IpAddr, Ipv4Addr};
use std::path::{Path, PathBuf};
use std::sync::{
    Mutex, OnceLock,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;
use tauri::{Emitter, Manager};
use tokio::sync::RwLock;

pub static READY: AtomicBool = AtomicBool::new(false);
pub static OPERATIONS: RwLock<()> = RwLock::const_new(());
static CHECK_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());
static STATUS: Mutex<Option<Value>> = Mutex::new(None);
static CYCLING: AtomicBool = AtomicBool::new(false);
static DOWNLOADING: AtomicBool = AtomicBool::new(false);
static INSTALL_AFTER_DOWNLOAD: AtomicBool = AtomicBool::new(false);

struct CycleGuard;
impl Drop for CycleGuard {
    fn drop(&mut self) {
        CYCLING.store(false, Ordering::SeqCst);
    }
}

struct DownloadGuard;
impl Drop for DownloadGuard {
    fn drop(&mut self) {
        DOWNLOADING.store(false, Ordering::SeqCst);
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct PendingUpdateMeta {
    version: String,
    signature: String,
    notes: Option<String>,
    size: u64,
    #[serde(default)]
    auto_install_attempted: bool,
}

pub fn format_public_version(version: &str) -> String {
    let trimmed = version.trim();
    if trimmed.is_empty() {
        return "V-0.0.00".into();
    }
    let clean = trimmed
        .strip_prefix("V-")
        .or_else(|| trimmed.strip_prefix("v-"))
        .or_else(|| trimmed.strip_prefix('V'))
        .or_else(|| trimmed.strip_prefix('v'))
        .unwrap_or(trimmed);
    format!("V-{clean}")
}

fn current_internal_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

fn compute_progress_percent(state: &str, downloaded: u64, total: Option<u64>) -> u64 {
    if matches!(state, "ready" | "installing" | "waiting-for-idle") {
        return 100;
    }
    match total {
        Some(t) if t > 0 => downloaded.saturating_mul(100).saturating_div(t).min(100),
        _ => 0,
    }
}

fn build_status(
    state: &str,
    version: Option<&str>,
    notes: Option<&str>,
    downloaded: u64,
    total: Option<u64>,
    error: Option<&str>,
) -> Value {
    let public_version = version.map(format_public_version);
    let progress_percent = compute_progress_percent(state, downloaded, total);
    json!({
        "state": state,
        "stage": state,
        "version": version,
        "publicVersion": public_version,
        "currentVersion": format_public_version(current_internal_version()),
        "notes": notes,
        "downloaded": downloaded,
        "total": total,
        "progressPercent": progress_percent,
        "installQueued": INSTALL_AFTER_DOWNLOAD.load(Ordering::SeqCst),
        "error": error,
        "checkedAt": chrono::Utc::now().to_rfc3339(),
    })
}

fn set_status(
    app: Option<&tauri::AppHandle>,
    state: &str,
    version: Option<&str>,
    notes: Option<&str>,
    downloaded: u64,
    total: Option<u64>,
    error: Option<&str>,
) -> Value {
    let value = build_status(state, version, notes, downloaded, total, error);
    if let Ok(mut guard) = STATUS.lock() {
        *guard = Some(value.clone());
    }
    if let Some(handle) = app {
        let _ = handle.emit("app-update-status", &value);
    }
    value
}

fn has_pending_update(val: &Value) -> bool {
    matches!(
        val["stage"].as_str(),
        Some("available" | "downloading" | "ready" | "waiting-for-idle")
    ) && val["version"]
        .as_str()
        .is_some_and(|v| !v.trim().is_empty())
}

fn preserved_checking_status(
    app: Option<&tauri::AppHandle>,
    prev: &Value,
) -> Value {
    if has_pending_update(prev) {
        set_status(
            app,
            "checking",
            prev["version"].as_str(),
            prev["notes"].as_str(),
            prev["downloaded"].as_u64().unwrap_or(0),
            prev["total"].as_u64(),
            None,
        )
    } else {
        set_status(app, "checking", None, None, 0, None, None)
    }
}

fn apply_periodic_check_error(
    app: Option<&tauri::AppHandle>,
    prev: &Value,
    err_msg: Option<&str>,
) -> Value {
    if has_pending_update(prev) {
        let stage = prev["stage"].as_str().unwrap_or("available");
        set_status(
            app,
            stage,
            prev["version"].as_str(),
            prev["notes"].as_str(),
            prev["downloaded"].as_u64().unwrap_or(0),
            prev["total"].as_u64(),
            err_msg,
        )
    } else {
        set_status(app, "retry-later", None, None, 0, None, err_msg)
    }
}

pub fn snapshot() -> Value {
    STATUS
        .lock()
        .ok()
        .and_then(|s| s.clone())
        .unwrap_or_else(|| build_status("idle", None, None, 0, None, None))
}

async fn idle() -> bool {
    if !READY.load(Ordering::SeqCst) {
        return false;
    }
    let running_processes = theseus::process::get_all()
        .await
        .map(|procs| !procs.is_empty())
        .unwrap_or(true);
    !running_processes
}

pub fn configured_pubkey() -> Option<&'static str> {
    let key = option_env!("SOLARA_UPDATER_PUBLIC_KEY")
        .unwrap_or(include_str!("../updater.pub"))
        .trim();
    if key.is_empty() { None } else { Some(key) }
}

pub fn configured_endpoints() -> anyhow::Result<Vec<reqwest::Url>> {
    Ok(vec![update_policy::GITHUB_LATEST_MANIFEST_URL.parse()?])
}

pub fn verify_minisign_signature(
    pubkey_b64: &str,
    signature_b64: &str,
    payload: &[u8],
) -> anyhow::Result<()> {
    let pubkey_box = String::from_utf8(
        BASE64_STANDARD
            .decode(pubkey_b64.trim())
            .context("Invalid base64 updater public key")?,
    )
    .context("Updater public key is not valid UTF-8")?;
    let sig_box = String::from_utf8(
        BASE64_STANDARD
            .decode(signature_b64.trim())
            .context("Invalid base64 update signature")?,
    )
    .context("Update signature is not valid UTF-8")?;

    let pubkey = PublicKey::decode(&pubkey_box)
        .map_err(|e| anyhow!("Failed to parse Minisign public key: {e}"))?;
    let signature = Signature::decode(&sig_box)
        .map_err(|e| anyhow!("Failed to parse Minisign signature: {e}"))?;
    pubkey
        .verify(payload, &signature, false)
        .map_err(|e| anyhow!("Minisign signature verification failed: {e}"))?;
    Ok(())
}

static MANIFEST_CLIENT_IPV4: OnceLock<reqwest::Client> = OnceLock::new();
static MANIFEST_CLIENT_DEFAULT: OnceLock<reqwest::Client> = OnceLock::new();
static DOWNLOAD_CLIENT_IPV4: OnceLock<reqwest::Client> = OnceLock::new();

fn build_manifest_client(force_ipv4: bool) -> reqwest::Client {
    let mut builder = reqwest::Client::builder()
        .user_agent(theseus::launcher_user_agent())
        .connect_timeout(Duration::from_secs(3))
        .timeout(Duration::from_secs(8))
        .tcp_nodelay(true)
        .pool_max_idle_per_host(4)
        .pool_idle_timeout(Duration::from_secs(90));
    if force_ipv4 {
        builder = builder.local_address(Some(IpAddr::V4(Ipv4Addr::UNSPECIFIED)));
    }
    builder.build().unwrap_or_else(|_| reqwest::Client::new())
}

fn build_download_client() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent(theseus::launcher_user_agent())
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(600))
        .tcp_nodelay(true)
        .local_address(Some(IpAddr::V4(Ipv4Addr::UNSPECIFIED)))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
}

fn manifest_client_ipv4() -> &'static reqwest::Client {
    MANIFEST_CLIENT_IPV4.get_or_init(|| build_manifest_client(true))
}

fn manifest_client_default() -> &'static reqwest::Client {
    MANIFEST_CLIENT_DEFAULT.get_or_init(|| build_manifest_client(false))
}

fn download_client() -> &'static reqwest::Client {
    DOWNLOAD_CLIENT_IPV4.get_or_init(build_download_client)
}

fn update_cache_dir(app: &tauri::AppHandle) -> anyhow::Result<PathBuf> {
    Ok(app.path().app_cache_dir()?.join("app-update"))
}

async fn clear_cached_update(cache_dir: &Path) {
    let _ = tokio::fs::remove_file(cache_dir.join("pending.bin")).await;
    let _ = tokio::fs::remove_file(cache_dir.join("pending.bin.tmp")).await;
    let _ = tokio::fs::remove_file(cache_dir.join("pending.json")).await;
    let _ = tokio::fs::remove_file(cache_dir.join("solaralauncher-update-setup.exe")).await;
}

async fn read_pending_meta(cache_dir: &Path) -> Option<PendingUpdateMeta> {
    let raw = tokio::fs::read_to_string(cache_dir.join("pending.json"))
        .await
        .ok()?;
    serde_json::from_str::<PendingUpdateMeta>(&raw).ok()
}

async fn write_pending_meta(cache_dir: &Path, meta: &PendingUpdateMeta) -> anyhow::Result<()> {
    let raw = serde_json::to_vec_pretty(meta)?;
    let tmp = cache_dir.join("pending.json.tmp");
    tokio::fs::write(&tmp, raw).await?;
    tokio::fs::rename(&tmp, cache_dir.join("pending.json")).await?;
    Ok(())
}

async fn read_verified_cached_update(
    cache_dir: &Path,
    pubkey: &str,
) -> Option<(PendingUpdateMeta, Vec<u8>)> {
    let meta = read_pending_meta(cache_dir).await?;
    let bytes = tokio::fs::read(cache_dir.join("pending.bin")).await.ok()?;
    if verify_minisign_signature(pubkey, &meta.signature, &bytes).is_ok() {
        Some((meta, bytes))
    } else {
        None
    }
}

fn should_auto_install_cached(
    meta: &PendingUpdateMeta,
    current_version: &str,
) -> bool {
    !meta.auto_install_attempted
        && update_policy::is_newer_version(&meta.version, current_version)
}

async fn fetch_manifest_from(
    client: &reqwest::Client,
    url: &'static str,
) -> anyhow::Result<Value> {
    let resp = client
        .get(url)
        .header("Accept", "application/json")
        .send()
        .await?
        .error_for_status()?;
    let manifest: Value = resp.json().await?;
    update_policy::validate_latest_manifest(&manifest)
        .map_err(|e| anyhow!("Invalid update manifest from {url}: {e}"))?;
    Ok(manifest)
}

async fn fetch_latest_manifest() -> anyhow::Result<Value> {
    // Race pooled IPv4-first and dual-stack clients across GitHub Releases CDN and Solara API
    // so broken Windows IPv6 SYN stalls (21s+) never delay update checks. Every candidate
    // response still passes strict `validate_latest_manifest` checks (trusted URL + signature).
    let (tx, mut rx) = tokio::sync::mpsc::channel::<anyhow::Result<Value>>(3);
    let attempts: [(&'static reqwest::Client, &'static str); 3] = [
        (manifest_client_ipv4(), update_policy::GITHUB_LATEST_MANIFEST_URL),
        (manifest_client_default(), update_policy::GITHUB_LATEST_MANIFEST_URL),
        (manifest_client_ipv4(), update_policy::SOLARA_API_UPDATE_MANIFEST_URL),
    ];

    let mut handles = Vec::with_capacity(attempts.len());
    for (client, url) in attempts {
        let tx = tx.clone();
        handles.push(tokio::spawn(async move {
            let res = fetch_manifest_from(client, url).await;
            let _ = tx.send(res).await;
        }));
    }
    drop(tx);

    let mut last_err = anyhow!("Failed to fetch update manifest");
    while let Some(result) = rx.recv().await {
        match result {
            Ok(manifest) => {
                for handle in handles {
                    handle.abort();
                }
                return Ok(manifest);
            }
            Err(err) => {
                last_err = err;
            }
        }
    }
    Err(last_err)
}

async fn download_and_verify_to_cache(
    app: &tauri::AppHandle,
    cache_dir: &Path,
    key: &str,
    version: &str,
    notes: Option<&str>,
    download_url: &str,
    signature: &str,
) -> anyhow::Result<Vec<u8>> {
    tokio::fs::create_dir_all(cache_dir).await?;
    set_status(Some(app), "downloading", Some(version), notes, 0, None, None);

    let mut resp = match download_client()
        .get(download_url)
        .send()
        .await
        .and_then(|r| r.error_for_status())
    {
        Ok(r) => r,
        Err(_) => {
            // Fallback to default dual-stack client if IPv4-only route fails
            let fallback = reqwest::Client::builder()
                .user_agent(theseus::launcher_user_agent())
                .timeout(Duration::from_secs(600))
                .build()?;
            fallback
                .get(download_url)
                .send()
                .await
                .and_then(|r| r.error_for_status())
                .map_err(|e| {
                    let msg = e.to_string();
                    set_status(
                        Some(app),
                        "available",
                        Some(version),
                        notes,
                        0,
                        None,
                        Some(&msg),
                    );
                    e
                })?
        }
    };

    let total = resp.content_length();
    let mut downloaded = 0u64;
    let mut last_emitted = 0u64;
    let mut buf = Vec::with_capacity(total.unwrap_or(32 * 1024 * 1024) as usize);
    while let Some(chunk) = resp.chunk().await? {
        downloaded = downloaded.saturating_add(chunk.len() as u64);
        buf.extend_from_slice(&chunk);
        if downloaded.saturating_sub(last_emitted) >= 262_144
            || total.is_some_and(|t| downloaded >= t)
        {
            last_emitted = downloaded;
            set_status(
                Some(app),
                "downloading",
                Some(version),
                notes,
                downloaded,
                total,
                None,
            );
        }
    }

    verify_minisign_signature(key, signature, &buf).map_err(|e| {
        let msg = e.to_string();
        set_status(
            Some(app),
            "available",
            Some(version),
            notes,
            0,
            None,
            Some(&msg),
        );
        e
    })?;

    let tmp_bin = cache_dir.join("pending.bin.tmp");
    tokio::fs::write(&tmp_bin, &buf).await?;
    tokio::fs::rename(&tmp_bin, cache_dir.join("pending.bin")).await?;

    let meta = PendingUpdateMeta {
        version: version.to_string(),
        signature: signature.to_string(),
        notes: notes.map(String::from),
        size: buf.len() as u64,
        auto_install_attempted: false,
    };
    write_pending_meta(cache_dir, &meta).await?;

    set_status(
        Some(app),
        "ready",
        Some(version),
        notes,
        buf.len() as u64,
        Some(buf.len() as u64),
        None,
    );

    Ok(buf)
}

async fn apply_verified_bytes(
    app: &tauri::AppHandle,
    cache_dir: &Path,
    version: &str,
    notes: Option<&str>,
    bytes: &[u8],
    require_idle: bool,
) -> anyhow::Result<Value> {
    let _exclusive = OPERATIONS.write().await;
    if require_idle && !idle().await {
        let msg = "Close running Minecraft instances or active downloads before restarting to update.";
        set_status(
            Some(app),
            "waiting-for-idle",
            Some(version),
            notes,
            bytes.len() as u64,
            Some(bytes.len() as u64),
            Some(msg),
        );
        bail!(msg);
    }

    set_status(
        Some(app),
        "installing",
        Some(version),
        notes,
        bytes.len() as u64,
        Some(bytes.len() as u64),
        None,
    );

    #[cfg(target_os = "windows")]
    {
        tokio::fs::create_dir_all(cache_dir).await?;
        let installer_path = cache_dir.join("solaralauncher-update-setup.exe");
        tokio::fs::write(&installer_path, bytes).await?;
        std::process::Command::new(&installer_path)
            .args(["/S", "/UPDATE", "/R"])
            .spawn()
            .map_err(|e| {
                let msg = e.to_string();
                set_status(
                    Some(app),
                    "ready",
                    Some(version),
                    notes,
                    bytes.len() as u64,
                    Some(bytes.len() as u64),
                    Some(&msg),
                );
                e
            })?;
        let _ = tokio::fs::remove_file(cache_dir.join("pending.bin")).await;
        let _ = tokio::fs::remove_file(cache_dir.join("pending.json")).await;
        app.exit(0);
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = tokio::fs::remove_file(cache_dir.join("pending.bin")).await;
        let _ = tokio::fs::remove_file(cache_dir.join("pending.json")).await;
    }

    Ok(snapshot())
}

fn spawn_background_download(
    app: tauri::AppHandle,
    version: String,
    notes: Option<String>,
    download_url: String,
    signature: String,
) {
    if DOWNLOADING.swap(true, Ordering::SeqCst) {
        return;
    }
    tauri::async_runtime::spawn(async move {
        let _guard = DownloadGuard;
        let Some(key) = configured_pubkey() else {
            return;
        };
        let Ok(cache_dir) = update_cache_dir(&app) else {
            return;
        };

        match download_and_verify_to_cache(
            &app,
            &cache_dir,
            key,
            &version,
            notes.as_deref(),
            &download_url,
            &signature,
        )
        .await
        {
            Ok(bytes) => {
                // Evergreen automatic update: as soon as the signed update is downloaded and
                // verified, automatically apply it right away if the launcher is idle (no game
                // running) or if the user explicitly queued install.
                let user_queued = INSTALL_AFTER_DOWNLOAD.swap(false, Ordering::SeqCst);
                if (user_queued || idle().await) && !CYCLING.swap(true, Ordering::SeqCst) {
                    let _cycle = CycleGuard;
                    if let Some(mut meta) = read_pending_meta(&cache_dir).await {
                        meta.auto_install_attempted = true;
                        let _ = write_pending_meta(&cache_dir, &meta).await;
                    }
                    let _ = apply_verified_bytes(
                        &app,
                        &cache_dir,
                        &version,
                        notes.as_deref(),
                        &bytes,
                        true,
                    )
                    .await;
                }
            }
            Err(err) => {
                INSTALL_AFTER_DOWNLOAD.store(false, Ordering::SeqCst);
                tracing::warn!("Background update download failed: {err}");
            }
        }
    });
}

/// Checks if a verified newer update was already downloaded in a prior session and
/// automatically installs it on launcher startup.
pub async fn try_apply_cached_update_on_startup(app: &tauri::AppHandle) -> bool {
    if !cfg!(windows) || cfg!(debug_assertions) {
        return false;
    }
    let Some(key) = configured_pubkey() else {
        return false;
    };
    let Ok(cache_dir) = update_cache_dir(app) else {
        return false;
    };

    let Some(meta) = read_pending_meta(&cache_dir).await else {
        return false;
    };

    // Clean up stale cache if we are already running this or a newer version
    if !update_policy::is_newer_version(&meta.version, current_internal_version()) {
        clear_cached_update(&cache_dir).await;
        return false;
    }

    let Some((mut verified_meta, bytes)) =
        read_verified_cached_update(&cache_dir, key).await
    else {
        clear_cached_update(&cache_dir).await;
        return false;
    };

    if !should_auto_install_cached(&verified_meta, current_internal_version()) {
        // Previously attempted on startup and cancelled; keep ready for one-click install.
        set_status(
            Some(app),
            "ready",
            Some(&verified_meta.version),
            verified_meta.notes.as_deref(),
            bytes.len() as u64,
            Some(bytes.len() as u64),
            None,
        );
        return false;
    }

    // Mark attempted before spawning installer so cancelling UAC never causes a boot loop.
    verified_meta.auto_install_attempted = true;
    let _ = write_pending_meta(&cache_dir, &verified_meta).await;

    set_status(
        Some(app),
        "installing",
        Some(&verified_meta.version),
        verified_meta.notes.as_deref(),
        bytes.len() as u64,
        Some(bytes.len() as u64),
        None,
    );

    // Give the splash screen a brief moment to render the installing status before handing off.
    tokio::time::sleep(Duration::from_millis(350)).await;

    if !CYCLING.swap(true, Ordering::SeqCst) {
        let _cycle = CycleGuard;
        return apply_verified_bytes(
            app,
            &cache_dir,
            &verified_meta.version,
            verified_meta.notes.as_deref(),
            &bytes,
            false,
        )
        .await
        .is_ok();
    }
    false
}

/// Checks for an available signed update and automatically downloads & verifies it in the
/// background so clicking "Update Now" (or reopening the launcher) installs immediately.
pub async fn check_only(app: &tauri::AppHandle) -> anyhow::Result<Value> {
    let _check_guard = CHECK_LOCK.lock().await;
    let Some(key) = configured_pubkey() else {
        set_status(Some(app), "unconfigured", None, None, 0, None, None);
        return Ok(
            json!({ "available": false, "stage": "unconfigured", "error": "unconfigured" }),
        );
    };

    let prev = snapshot();
    let prev_stage = prev["stage"].as_str().unwrap_or("");
    if matches!(prev_stage, "downloading" | "installing") {
        return Ok(json!({
            "available": true,
            "stage": prev_stage,
            "version": prev["version"],
            "publicVersion": prev["publicVersion"],
            "currentVersion": prev["currentVersion"],
            "notes": prev["notes"],
            "downloaded": prev["downloaded"],
            "total": prev["total"],
            "progressPercent": prev["progressPercent"],
        }));
    }

    preserved_checking_status(Some(app), &prev);
    let manifest = match fetch_latest_manifest().await {
        Ok(m) => m,
        Err(e) => {
            let msg = e.to_string();
            if has_pending_update(&prev) {
                let _ = apply_periodic_check_error(Some(app), &prev, Some(&msg));
            } else {
                set_status(Some(app), "error", None, None, 0, None, Some(&msg));
            }
            return Err(e);
        }
    };

    let remote_version = manifest["version"].as_str().unwrap_or_default();
    if !update_policy::is_newer_version(
        remote_version,
        current_internal_version(),
    ) {
        if let Ok(cache_dir) = update_cache_dir(app) {
            clear_cached_update(&cache_dir).await;
        }
        set_status(Some(app), "up-to-date", None, None, 0, None, None);
        return Ok(json!({
            "available": false,
            "stage": "up-to-date",
            "currentVersion": format_public_version(current_internal_version()),
        }));
    }

    let version = remote_version.to_string();
    let public_version = format_public_version(&version);
    let notes = manifest["notes"].as_str().map(String::from);
    let download_url = manifest["platforms"]["windows-x86_64"]["url"]
        .as_str()
        .unwrap_or_default()
        .to_string();
    let signature = manifest["platforms"]["windows-x86_64"]["signature"]
        .as_str()
        .unwrap_or_default()
        .to_string();

    let cached_ready = if let Ok(cache_dir) = update_cache_dir(app) {
        let bytes = tokio::fs::read(cache_dir.join("pending.bin"))
            .await
            .ok()
            .filter(|b| verify_minisign_signature(key, &signature, b).is_ok());
        if let Some(b) = bytes {
            let existing = read_pending_meta(&cache_dir).await;
            let meta = PendingUpdateMeta {
                version: version.clone(),
                signature: signature.clone(),
                notes: notes.clone(),
                size: b.len() as u64,
                auto_install_attempted: existing
                    .filter(|m| m.version == version)
                    .map(|m| m.auto_install_attempted)
                    .unwrap_or(false),
            };
            let _ = write_pending_meta(&cache_dir, &meta).await;
            Some(b.len() as u64)
        } else {
            None
        }
    } else {
        None
    };

    if let Some(size) = cached_ready {
        set_status(
            Some(app),
            "ready",
            Some(&version),
            notes.as_deref(),
            size,
            Some(size),
            None,
        );
        return Ok(json!({
            "available": true,
            "stage": "ready",
            "version": version,
            "publicVersion": public_version,
            "currentVersion": format_public_version(current_internal_version()),
            "notes": notes,
            "downloaded": size,
            "total": size,
            "progressPercent": 100,
        }));
    }

    set_status(
        Some(app),
        "available",
        Some(&version),
        notes.as_deref(),
        0,
        None,
        None,
    );

    // Automatically download & verify the signed installer in the background on Windows release builds
    if cfg!(windows) && !cfg!(debug_assertions) && !download_url.is_empty() {
        spawn_background_download(
            app.clone(),
            version.clone(),
            notes.clone(),
            download_url,
            signature,
        );
    }

    Ok(json!({
        "available": true,
        "stage": "available",
        "version": version,
        "publicVersion": public_version,
        "currentVersion": format_public_version(current_internal_version()),
        "notes": notes,
        "downloaded": 0,
        "total": null,
        "progressPercent": 0,
    }))
}

/// Immediately installs a cached verified update (or queues auto-install if a background
/// download is already running, or downloads and installs right away) when the user clicks
/// "Update Now" or "Restart to Update".
pub async fn install_now(app: &tauri::AppHandle) -> anyhow::Result<Value> {
    // If background download is already running, queue automatic install on completion!
    if DOWNLOADING.load(Ordering::SeqCst) {
        INSTALL_AFTER_DOWNLOAD.store(true, Ordering::SeqCst);
        let prev = snapshot();
        return Ok(set_status(
            Some(app),
            "downloading",
            prev["version"].as_str(),
            prev["notes"].as_str(),
            prev["downloaded"].as_u64().unwrap_or(0),
            prev["total"].as_u64(),
            None,
        ));
    }

    if CYCLING.swap(true, Ordering::SeqCst) {
        return Ok(snapshot());
    }
    let _cycle = CycleGuard;

    let Some(key) = configured_pubkey() else {
        let val = set_status(
            Some(app),
            "unconfigured",
            None,
            None,
            0,
            None,
            Some("Updater key not configured"),
        );
        return Ok(val);
    };

    let cache_dir = update_cache_dir(app)?;
    tokio::fs::create_dir_all(&cache_dir).await?;

    // Fast path: if a verified newer update is already cached on disk, install it immediately
    // without waiting for another network fetch!
    if let Some((meta, bytes)) = read_verified_cached_update(&cache_dir, key).await {
        if update_policy::is_newer_version(&meta.version, current_internal_version()) {
            return apply_verified_bytes(
                app,
                &cache_dir,
                &meta.version,
                meta.notes.as_deref(),
                &bytes,
                true,
            )
            .await;
        } else {
            clear_cached_update(&cache_dir).await;
        }
    }

    let prev = snapshot();
    let _ = preserved_checking_status(Some(app), &prev);
    let manifest = fetch_latest_manifest()
        .await
        .context("Failed to contact update service")?;
    let remote_version = manifest["version"].as_str().unwrap_or_default();
    if !update_policy::is_newer_version(
        remote_version,
        current_internal_version(),
    ) {
        clear_cached_update(&cache_dir).await;
        return Ok(set_status(
            Some(app),
            "up-to-date",
            None,
            None,
            0,
            None,
            None,
        ));
    }

    let version = remote_version.to_string();
    let notes = manifest["notes"].as_str().map(String::from);
    let download_url = manifest["platforms"]["windows-x86_64"]["url"]
        .as_str()
        .ok_or_else(|| anyhow!("Missing Windows update URL"))?;
    let signature = manifest["platforms"]["windows-x86_64"]["signature"]
        .as_str()
        .ok_or_else(|| anyhow!("Missing Windows update signature"))?;

    let cached = tokio::fs::read(cache_dir.join("pending.bin"))
        .await
        .ok()
        .filter(|bytes| verify_minisign_signature(key, signature, bytes).is_ok());

    let bytes = if let Some(bytes) = cached {
        bytes
    } else {
        let _dl_guard = if DOWNLOADING.swap(true, Ordering::SeqCst) {
            return Ok(snapshot());
        } else {
            DownloadGuard
        };
        download_and_verify_to_cache(
            app,
            &cache_dir,
            key,
            &version,
            notes.as_deref(),
            download_url,
            signature,
        )
        .await?
    };

    apply_verified_bytes(
        app,
        &cache_dir,
        &version,
        notes.as_deref(),
        &bytes,
        true,
    )
    .await
}

pub async fn trigger(app: &tauri::AppHandle) -> anyhow::Result<Value> {
    READY.store(true, Ordering::SeqCst);
    check_only(app).await
}

pub fn start(app: tauri::AppHandle) {
    READY.store(true, Ordering::SeqCst);
    tauri::async_runtime::spawn(async move {
        if !cfg!(windows) || cfg!(debug_assertions) {
            set_status(Some(&app), "development", None, None, 0, None, None);
            return;
        }

        // If a verified newer update was already downloaded in an earlier session,
        // automatically apply it right on startup!
        if try_apply_cached_update_on_startup(&app).await {
            return;
        }

        // Evergreen instant startup check: immediately check GitHub Releases CDN at 0ms on
        // launch (protected by CHECK_LOCK so concurrent frontend checks reuse the result)
        // so even an older solaralauncher.exe installer immediately fetches and applies the
        // newest version on first launch.
        loop {
            if READY.load(Ordering::SeqCst) {
                let before_check = snapshot();
                if check_only(&app).await.is_err() {
                    let cur = snapshot();
                    let err_msg = cur["error"].as_str().map(|s| s.to_string());
                    let fallback_prev = if has_pending_update(&cur) {
                        cur
                    } else {
                        before_check
                    };
                    apply_periodic_check_error(
                        Some(&app),
                        &fallback_prev,
                        err_msg.as_deref(),
                    );
                }
                tokio::time::sleep(std::time::Duration::from_secs(900)).await;
            } else {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_version_formats_canonical_v_prefix() {
        assert_eq!(format_public_version("0.0.63"), "V-0.0.63");
        assert_eq!(format_public_version("V-0.0.63"), "V-0.0.63");
        assert_eq!(format_public_version("v0.0.63"), "V-0.0.63");
        assert_eq!(format_public_version(""), "V-0.0.00");
    }

    #[test]
    fn build_status_exposes_stage_and_public_version() {
        let s = build_status(
            "available",
            Some("0.0.63"),
            Some("Bug fixes"),
            0,
            None,
            None,
        );
        assert_eq!(s["stage"], "available");
        assert_eq!(s["state"], "available");
        assert_eq!(s["version"], "0.0.63");
        assert_eq!(s["publicVersion"], "V-0.0.63");
        assert_eq!(s["notes"], "Bug fixes");
        assert_eq!(s["progressPercent"], 0);
    }

    #[test]
    fn build_status_computes_progress_percent_for_downloading_and_ready() {
        let dl = build_status(
            "downloading",
            Some("0.0.74"),
            None,
            5_000_000,
            Some(20_000_000),
            None,
        );
        assert_eq!(dl["progressPercent"], 25);

        let ready = build_status(
            "ready",
            Some("0.0.74"),
            None,
            20_000_000,
            Some(20_000_000),
            None,
        );
        assert_eq!(ready["progressPercent"], 100);
    }

    #[test]
    fn pending_meta_auto_install_policy_guards_retries_and_version() {
        let fresh = PendingUpdateMeta {
            version: "0.0.75".into(),
            signature: "sig".into(),
            notes: Some("Release".into()),
            size: 1234,
            auto_install_attempted: false,
        };
        assert!(should_auto_install_cached(&fresh, "0.0.74"));
        assert!(!should_auto_install_cached(&fresh, "0.0.75"));
        assert!(!should_auto_install_cached(&fresh, "0.0.76"));

        let attempted = PendingUpdateMeta {
            auto_install_attempted: true,
            ..fresh
        };
        assert!(!should_auto_install_cached(&attempted, "0.0.74"));
    }

    #[test]
    fn pending_update_survives_periodic_check_error() {
        let pending = build_status(
            "available",
            Some("0.0.63"),
            Some("Solara update"),
            0,
            None,
            None,
        );
        let next =
            apply_periodic_check_error(None, &pending, Some("network timeout"));
        assert_eq!(next["stage"], "available");
        assert_eq!(next["version"], "0.0.63");
        assert_eq!(next["publicVersion"], "V-0.0.63");
        assert_eq!(next["notes"], "Solara update");
        assert_eq!(next["error"], "network timeout");
    }

    #[test]
    fn idle_check_error_transitions_to_retry_later() {
        let idle = build_status("up-to-date", None, None, 0, None, None);
        let next =
            apply_periodic_check_error(None, &idle, Some("network timeout"));
        assert_eq!(next["stage"], "retry-later");
        assert!(next["version"].is_null());
        assert_eq!(next["error"], "network timeout");
    }

    #[test]
    fn updater_endpoints_and_pubkey_are_configured() {
        theseus::install_crypto_provider();
        let endpoints =
            configured_endpoints().expect("configured updater endpoints");
        assert_eq!(endpoints.len(), 1);
        assert_eq!(
            endpoints[0].as_str(),
            "https://github.com/Solara-Inc-Development/solara-launcher/releases/latest/download/latest.json"
        );
        let pubkey = configured_pubkey().expect("updater.pub present");
        assert!(!pubkey.is_empty());

        let api_url = reqwest::Url::parse(
            update_policy::SOLARA_API_UPDATE_MANIFEST_URL,
        )
        .expect("valid api manifest url");
        assert_eq!(api_url.scheme(), "https");
        assert_eq!(api_url.host_str(), Some("api.solara-inc.in"));
        let _ = manifest_client_ipv4();
        let _ = manifest_client_default();
        let _ = download_client();
    }
}
