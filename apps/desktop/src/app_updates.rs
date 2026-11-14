use crate::{engine, update_policy};
use anyhow::{anyhow, Context};
use serde_json::{json, Value};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Mutex,
};
use tauri::{Emitter, Manager};
use tauri_plugin_updater::UpdaterExt;
use tokio::sync::RwLock;

pub static READY: AtomicBool = AtomicBool::new(false);
// Mutating commands hold a reader; final replacement holds the writer. Thus a
// launch/setup cannot race the final idle check and installer hand-off.
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

fn preserved_checking_status(app: Option<&tauri::AppHandle>, prev: &Value) -> Value {
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
    READY.load(Ordering::SeqCst)
        && engine::snapshot()
            .await
            .map(|s| !update_policy::busy(&s))
            .unwrap_or(false)
}

fn configured_pubkey() -> Option<&'static str> {
    let key = option_env!("SOLARA_UPDATER_PUBLIC_KEY")
        .unwrap_or(include_str!("../updater.pub"))
        .trim();
    if key.is_empty() {
        None
    } else {
        Some(key)
    }
}

fn configured_endpoints() -> anyhow::Result<Vec<reqwest::Url>> {
    Ok(vec![update_policy::GITHUB_LATEST_MANIFEST_URL.parse()?])
}

fn build_updater(
    app: &tauri::AppHandle,
    key: &str,
    timeout_secs: u64,
) -> anyhow::Result<tauri_plugin_updater::Updater> {
    Ok(app
        .updater_builder()
        .pubkey(key)
        .timeout(std::time::Duration::from_secs(timeout_secs))
        .endpoints(configured_endpoints()?)?
        .build()?)
}

/// Checks for an available signed update without downloading or applying it automatically.
/// Users always confirm via "Update Now" or topbar "Restart to Update".
pub async fn check_only(app: &tauri::AppHandle) -> anyhow::Result<Value> {
    let Some(key) = configured_pubkey() else {
        set_status(Some(app), "unconfigured", None, None, 0, None, None);
        return Ok(json!({ "available": false, "stage": "unconfigured", "error": "unconfigured" }));
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
    let updater = build_updater(app, key, 60)?;
    let update = match updater.check().await {
        Ok(u) => u,
        Err(e) => {
            let msg = e.to_string();
            if has_pending_update(&prev) {
                let _ = apply_periodic_check_error(Some(app), &prev, Some(&msg));
            } else {
                set_status(Some(app), "error", None, None, 0, None, Some(&msg));
            }
            return Err(e.into());
        }
    };

    let Some(update) = update else {
        if let Ok(cache) = app.path().app_cache_dir() {
            let _ = tokio::fs::remove_file(cache.join("app-update/pending.bin")).await;
        }
        set_status(Some(app), "up-to-date", None, None, 0, None, None);
        return Ok(json!({
            "available": false,
            "stage": "up-to-date",
            "currentVersion": format_public_version(current_internal_version()),
        }));
    };

    let version = update.version.clone();
    let public_version = format_public_version(&version);
    let notes = update.body.clone();
    let cached_ready = if let Ok(cache) = app.path().app_cache_dir() {
        tokio::fs::read(cache.join("app-update/pending.bin"))
            .await
            .ok()
            .filter(|bytes| update.verify(bytes).is_ok())
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

/// Explicitly downloads (if not already cached), verifies signature, and installs the update
/// only when the user clicks "Update Now" or "Restart to Update".
pub async fn install_now(app: &tauri::AppHandle) -> anyhow::Result<Value> {
    if CYCLING.swap(true, Ordering::SeqCst) {
        return Ok(snapshot());
    }
    let _cycle = CycleGuard;

    let Some(key) = configured_pubkey() else {
        let val = set_status(Some(app), "unconfigured", None, None, 0, None, Some("Updater key not configured"));
        return Ok(val);
    };

    let prev = snapshot();
    let _ = preserved_checking_status(Some(app), &prev);
    let updater = build_updater(app, key, 600)?;
    let Some(update) = updater.check().await.context("Failed to contact update service")? else {
        if let Ok(cache) = app.path().app_cache_dir() {
            let _ = tokio::fs::remove_file(cache.join("app-update/pending.bin")).await;
        }
        return Ok(set_status(Some(app), "up-to-date", None, None, 0, None, None));
    };

    let version = update.version.clone();
    let notes = update.body.clone();
    let cache = app.path().app_cache_dir()?.join("app-update");
    tokio::fs::create_dir_all(&cache).await?;
    let file = cache.join("pending.bin");
    let cached = tokio::fs::read(&file)
        .await
        .ok()
        .filter(|bytes| update.verify(bytes).is_ok());

    let bytes = if let Some(bytes) = cached {
        bytes
    } else {
        let mut downloaded = 0u64;
        set_status(
            Some(app),
            "downloading",
            Some(&version),
            notes.as_deref(),
            0,
            None,
            None,
        );
        let ver_ref = version.clone();
        let notes_ref = notes.clone();
        let bytes = update
            .download(
                |chunk, total| {
                    downloaded = downloaded.saturating_add(chunk as u64);
                    set_status(
                        Some(app),
                        "downloading",
                        Some(&ver_ref),
                        notes_ref.as_deref(),
                        downloaded,
                        total,
                        None,
                    );
                },
                || {},
            )
            .await
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
        // Only verified bytes are cached; signature/version verified again inside download & install.
        tokio::fs::write(&file, &bytes).await?;
        bytes
    };

    let _exclusive = OPERATIONS.write().await;
    if !idle().await {
        let msg = "Close running Minecraft instances, local servers, or active downloads before restarting to update.";
        set_status(
            Some(app),
            "waiting-for-idle",
            Some(&version),
            notes.as_deref(),
            bytes.len() as u64,
            Some(bytes.len() as u64),
            Some(msg),
        );
        return Err(anyhow!(msg));
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
    // Tauri verifies signature + version again, quietly replaces this same app,
    // and relaunches it on Windows. Worlds/config/account data remain untouched.
    update.install(&bytes).map_err(|e| {
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
    let _ = tokio::fs::remove_file(file).await;
    Ok(snapshot())
}

pub async fn trigger(app: &tauri::AppHandle) -> anyhow::Result<Value> {
    READY.store(true, Ordering::SeqCst);
    check_only(app).await
}

pub fn start(app: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        if !cfg!(windows) || cfg!(debug_assertions) {
            set_status(Some(&app), "development", None, None, 0, None, None);
            return;
        }
        loop {
            if READY.load(Ordering::SeqCst) {
                let auto_enabled = engine::preferences()
                    .await
                    .map(|p| p.auto_update_enabled)
                    .unwrap_or(true);
                if !auto_enabled {
                    tokio::time::sleep(std::time::Duration::from_secs(60)).await;
                    continue;
                }
                let before_check = snapshot();
                // Only check for availability on startup / periodic interval; never auto-install silently.
                if check_only(&app).await.is_err() {
                    let cur = snapshot();
                    let err_msg = cur["error"].as_str().map(|s| s.to_string());
                    let fallback_prev = if has_pending_update(&cur) {
                        cur
                    } else {
                        before_check
                    };
                    apply_periodic_check_error(Some(&app), &fallback_prev, err_msg.as_deref());
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
        assert_eq!(format_public_version("0.0.39"), "V-0.0.39");
        assert_eq!(format_public_version("V-0.0.39"), "V-0.0.39");
        assert_eq!(format_public_version("v0.0.39"), "V-0.0.39");
        assert_eq!(format_public_version(""), "V-0.0.00");
    }

    #[test]
    fn build_status_exposes_stage_and_public_version() {
        let s = build_status("available", Some("0.0.39"), Some("Bug fixes"), 0, None, None);
        assert_eq!(s["stage"], "available");
        assert_eq!(s["state"], "available");
        assert_eq!(s["version"], "0.0.39");
        assert_eq!(s["publicVersion"], "V-0.0.39");
        assert_eq!(s["notes"], "Bug fixes");
    }

    #[test]
    fn pending_update_survives_periodic_check_error() {
        let pending = build_status("available", Some("0.0.39"), Some("Cobalt Blue UI"), 0, None, None);
        let next = apply_periodic_check_error(None, &pending, Some("network timeout"));
        assert_eq!(next["stage"], "available");
        assert_eq!(next["version"], "0.0.39");
        assert_eq!(next["publicVersion"], "V-0.0.39");
        assert_eq!(next["notes"], "Cobalt Blue UI");
        assert_eq!(next["error"], "network timeout");
    }

    #[test]
    fn idle_check_error_transitions_to_retry_later() {
        let idle = build_status("up-to-date", None, None, 0, None, None);
        let next = apply_periodic_check_error(None, &idle, Some("network timeout"));
        assert_eq!(next["stage"], "retry-later");
        assert!(next["version"].is_null());
        assert_eq!(next["error"], "network timeout");
    }

    #[test]
    fn updater_endpoints_point_to_github_releases_cdn() {
        let endpoints = configured_endpoints().expect("configured updater endpoints");
        assert_eq!(endpoints.len(), 1);
        assert_eq!(
            endpoints[0].as_str(),
            "https://github.com/Solara-Inc-Development/solara-launcher/releases/latest/download/latest.json"
        );
    }
}
