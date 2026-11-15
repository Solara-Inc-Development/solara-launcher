use crate::api::Result;
use crate::app_updates;
use serde_json::Value;
use tauri::{ResourceId, Runtime, Webview};

#[derive(Default)]
pub struct PendingUpdateData;

#[tauri::command]
pub async fn check_app_update(app: tauri::AppHandle) -> Result<Value> {
    Box::pin(async move { Ok(app_updates::trigger(&app).await?) }).await
}

#[tauri::command]
pub async fn install_app_update(app: tauri::AppHandle) -> Result<Value> {
    Box::pin(async move { Ok(app_updates::install_now(&app).await?) }).await
}

#[tauri::command]
pub fn get_app_update_status() -> Value {
    app_updates::snapshot()
}

#[tauri::command]
pub async fn get_update_size<R: Runtime>(
    _webview: Webview<R>,
    _rid: ResourceId,
) -> Result<Option<u64>> {
    let snap = app_updates::snapshot();
    Ok(snap["total"].as_u64())
}

#[tauri::command]
pub async fn enqueue_update_for_installation(
    app: tauri::AppHandle,
    _rid: Option<ResourceId>,
) -> Result<()> {
    Box::pin(async move {
        app_updates::install_now(&app).await?;
        Ok(())
    })
    .await
}

#[tauri::command]
pub fn remove_enqueued_update<R: Runtime>(_webview: Webview<R>) {}
