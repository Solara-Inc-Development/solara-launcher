use crate::update_policy;
use anyhow::{Context, anyhow, bail};
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use minisign_verify::{PublicKey, Signature};
use serde_json::{Value, json};
use std::net::{IpAddr, Ipv4Addr};
use std::sync::{
    Mutex, OnceLock,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;
use tauri::{Emitter, Manager};
use tokio::sync::RwLock;

pub static READY: AtomicBool = AtomicBool::new(false);
pub static OPERATIONS: RwLock<()> = RwLock::const_new(());
static STATUS: Mutex<Option<Value>> = Mutex::new(None);
static CYCLING: AtomicBool = AtomicBool::new(false);

struct CycleGuard;
impl Drop for CycleGuard {
    fn drop(&mut self) {
        CYCLING.store(false, Ordering::SeqCst);
    }
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

fn build_status(
    state: &str,
    version: Option<&str>,
    notes: Option<&str>,
    downloaded: u64,
    total: Option<u64>,
    error: Option<&str>,
) -> Value {
    let public_version = version.map(format_public_version);
    json!({
        "state": state,
        "stage": state,
        "version": version,
        "publicVersion": public_version,
        "currentVersion": format_public_version(current_internal_version()),
        "notes": notes,
        "downloaded": downloaded,
        "total": total,
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
        Some("available" | "ready" | "waiting-for-idle")
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

fn manifest_client_ipv4() -> &'static reqwest::Client {
    MANIFEST_CLIENT_IPV4.get_or_init(|| build_manifest_client(true))
}

fn manifest_client_default() -> &'static reqwest::Client {
    MANIFEST_CLIENT_DEFAULT.get_or_init(|| build_manifest_client(false))
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

/// Checks for an available signed update without downloading or applying it automatically.
/// Users always confirm via "Update Now" or topbar "Restart to Update".
pub async fn check_only(app: &tauri::AppHandle) -> anyhow::Result<Value> {
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
        if let Ok(cache) = app.path().app_cache_dir() {
            let _ = tokio::fs::remove_file(cache.join("app-update/pending.bin"))
                .await;
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
    let signature = manifest["platforms"]["windows-x86_64"]["signature"]
        .as_str()
        .unwrap_or_default();

    let cached_ready = if let Ok(cache) = app.path().app_cache_dir() {
        tokio::fs::read(cache.join("app-update/pending.bin"))
            .await
            .ok()
            .filter(|bytes| {
                verify_minisign_signature(key, signature, bytes).is_ok()
            })
            .map(|bytes| bytes.len() as u64)
    } else {
        None
    };

    let stage = if cached_ready.is_some() {
        "ready"
    } else {
        "available"
    };
    let size = cached_ready.unwrap_or(0);
    set_status(
        Some(app),
        stage,
        Some(&version),
        notes.as_deref(),
        size,
        cached_ready,
        None,
    );

    Ok(json!({
        "available": true,
        "stage": stage,
        "version": version,
        "publicVersion": public_version,
        "currentVersion": format_public_version(current_internal_version()),
        "notes": notes,
    }))
}

/// Explicitly downloads (if not already cached), verifies Minisign signature, and installs the update
/// only when the user clicks "Update Now" or "Restart to Update".
pub async fn install_now(app: &tauri::AppHandle) -> anyhow::Result<Value> {
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
        if let Ok(cache) = app.path().app_cache_dir() {
            let _ = tokio::fs::remove_file(cache.join("app-update/pending.bin"))
                .await;
        }
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

    let cache = app.path().app_cache_dir()?.join("app-update");
    tokio::fs::create_dir_all(&cache).await?;
    let file = cache.join("pending.bin");
    let cached = tokio::fs::read(&file)
        .await
        .ok()
        .filter(|bytes| verify_minisign_signature(key, signature, bytes).is_ok());

    let bytes = if let Some(bytes) = cached {
        bytes
    } else {
        set_status(
            Some(app),
            "downloading",
            Some(&version),
            notes.as_deref(),
            0,
            None,
            None,
        );
        let client = reqwest::Client::builder()
            .user_agent(theseus::launcher_user_agent())
            .timeout(std::time::Duration::from_secs(600))
            .build()?;
        let mut resp = client
            .get(download_url)
            .send()
            .await
            .and_then(|r| r.error_for_status())
            .map_err(|e| {
                let msg = e.to_string();
                set_status(
                    Some(app),
                    "available",
                    Some(&version),
                    notes.as_deref(),
                    0,
                    None,
                    Some(&msg),
                );
                e
            })?;
        let total = resp.content_length();
        let mut downloaded = 0u64;
        let mut buf = Vec::with_capacity(total.unwrap_or(64 * 1024 * 1024) as usize);
        while let Some(chunk) = resp.chunk().await? {
            downloaded = downloaded.saturating_add(chunk.len() as u64);
            buf.extend_from_slice(&chunk);
            set_status(
                Some(app),
                "downloading",
                Some(&version),
                notes.as_deref(),
                downloaded,
                total,
                None,
            );
        }
        verify_minisign_signature(key, signature, &buf).map_err(|e| {
            let msg = e.to_string();
            set_status(
                Some(app),
                "available",
                Some(&version),
                notes.as_deref(),
                0,
                None,
                Some(&msg),
            );
            e
        })?;
        tokio::fs::write(&file, &buf).await?;
        buf
    };

    let _exclusive = OPERATIONS.write().await;
    if !idle().await {
        let msg = "Close running Minecraft instances or active downloads before restarting to update.";
        set_status(
            Some(app),
            "waiting-for-idle",
            Some(&version),
            notes.as_deref(),
            bytes.len() as u64,
            Some(bytes.len() as u64),
            Some(msg),
        );
        bail!(msg);
    }

    set_status(
        Some(app),
        "installing",
        Some(&version),
        notes.as_deref(),
        bytes.len() as u64,
        Some(bytes.len() as u64),
        None,
    );

    #[cfg(target_os = "windows")]
    {
        let installer_path = cache.join("solaralauncher-update-setup.exe");
        tokio::fs::write(&installer_path, &bytes).await?;
        std::process::Command::new(&installer_path)
            .args(["/S", "/R"])
            .spawn()
            .map_err(|e| {
                let msg = e.to_string();
                set_status(
                    Some(app),
                    "ready",
                    Some(&version),
                    notes.as_deref(),
                    bytes.len() as u64,
                    Some(bytes.len() as u64),
                    Some(&msg),
                );
                e
            })?;
        let _ = tokio::fs::remove_file(&file).await;
        app.exit(0);
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = tokio::fs::remove_file(&file).await;
    }

    Ok(snapshot())
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
        // Frontend triggers an immediate check on mount; wait before starting periodic polling
        // so startup does not fire duplicate concurrent network checks.
        tokio::time::sleep(Duration::from_secs(15)).await;
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
    }
}
