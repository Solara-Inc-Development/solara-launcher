use crate::{engine, update_policy};
use serde_json::{json, Value};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Mutex,
};
use tauri::Manager;
use tauri_plugin_updater::UpdaterExt;
use tokio::sync::RwLock;

pub static READY: AtomicBool = AtomicBool::new(false);
// Mutating commands hold a reader; final replacement holds the writer. Thus a
// launch/setup cannot race the final idle check and installer hand-off.
pub static OPERATIONS: RwLock<()> = RwLock::const_new(());
static STATUS: Mutex<Option<Value>> = Mutex::new(None);

fn status(state: &str, downloaded: u64, total: Option<u64>) {
    if let Ok(mut value) = STATUS.lock() {
        *value = Some(json!({"state":state,"downloaded":downloaded,"total":total}));
    }
}
pub fn snapshot() -> Value {
    STATUS
        .lock()
        .ok()
        .and_then(|s| s.clone())
        .unwrap_or_else(|| json!({"state":"waiting"}))
}
async fn idle() -> bool {
    READY.load(Ordering::SeqCst)
        && engine::snapshot()
            .await
            .map(|s| !update_policy::busy(&s))
            .unwrap_or(false)
}

async fn cycle(app: &tauri::AppHandle) -> anyhow::Result<()> {
    if !idle().await {
        status("waiting-for-idle", 0, None);
        return Ok(());
    }
    let key = option_env!("SOLARA_UPDATER_PUBLIC_KEY")
        .unwrap_or(include_str!("../updater.pub"))
        .trim();
    if key.is_empty() {
        status("unconfigured", 0, None);
        return Ok(());
    }
    status("checking", 0, None);
    let updater = app
        .updater_builder()
        .pubkey(key)
        .timeout(std::time::Duration::from_secs(600))
        .endpoints(vec![
            "https://api.solara-inc.in/v1/launcher/update/windows/x86_64".parse()?,
        ])?
        .build()?;
    let Some(update) = updater.check().await? else {
        let _ = tokio::fs::remove_file(app.path().app_cache_dir()?.join("app-update/pending.bin"))
            .await;
        status("up-to-date", 0, None);
        return Ok(());
    };
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
        status("downloading", 0, None);
        let bytes = update
            .download(
                |chunk, total| {
                    downloaded = downloaded.saturating_add(chunk as u64);
                    status("downloading", downloaded, total)
                },
                || {},
            )
            .await?;
        // Only verified bytes are cached. Reverify with fresh metadata before reuse.
        tokio::fs::write(&file, &bytes).await?;
        bytes
    };
    let _exclusive = OPERATIONS.write().await;
    if !idle().await {
        status(
            "waiting-for-idle",
            bytes.len() as u64,
            Some(bytes.len() as u64),
        );
        return Ok(());
    }
    status("installing", bytes.len() as u64, Some(bytes.len() as u64));
    // Tauri verifies signature + version again, quietly replaces this same app,
    // and relaunches it on Windows. Worlds/config/account data are outside it.
    update.install(&bytes)?;
    let _ = tokio::fs::remove_file(file).await;
    Ok(())
}

pub fn start(app: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        if !cfg!(windows) || cfg!(debug_assertions) {
            status("development", 0, None);
            return;
        }
        loop {
            if READY.load(Ordering::SeqCst) {
                if cycle(&app).await.is_err() {
                    status("retry-later", 0, None)
                }
                let deferred = snapshot()["state"] == "waiting-for-idle";
                tokio::time::sleep(std::time::Duration::from_secs(if deferred {
                    60
                } else {
                    900
                }))
                .await;
            } else {
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
        }
    });
}
