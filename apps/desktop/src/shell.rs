use crate::{engine, identity};
use serde_json::Value;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Once,
};
use tauri::WindowEvent;
use tauri_plugin_opener::OpenerExt;

static PANIC_HOOK_INIT: Once = Once::new();

fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}

fn redact_panic_text(raw: &str) -> String {
    let bearer_re = regex::Regex::new(r"(?i)bearer\s+[A-Za-z0-9\-._~+/]+=*").ok();
    let kv_re = regex::Regex::new(
        r"(?i)(access_token|refresh_token|minecraft_token|password|secret|token|authorization)\s*[:=]\s*[^\s,;]+",
    )
    .ok();
    let mut cleaned = raw.replace(['\r', '\n'], " ");
    if let Some(re) = bearer_re {
        cleaned = re.replace_all(&cleaned, "Bearer [REDACTED]").into_owned();
    }
    if let Some(re) = kv_re {
        cleaned = re.replace_all(&cleaned, "$1=[REDACTED]").into_owned();
    }
    cleaned.chars().take(512).collect()
}

fn crash_log_path() -> std::path::PathBuf {
    dirs::data_local_dir()
        .map(|dir| dir.join("in.solara.launcher"))
        .unwrap_or_else(std::env::temp_dir)
        .join("crash.log")
}

fn install_panic_hook() {
    PANIC_HOOK_INIT.call_once(|| {
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            let msg = if let Some(s) = info.payload().downcast_ref::<&str>() {
                *s
            } else if let Some(s) = info.payload().downcast_ref::<String>() {
                s.as_str()
            } else {
                "non-string panic payload"
            };
            let loc = info
                .location()
                .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
                .unwrap_or_else(|| "unknown".into());
            let thread = std::thread::current()
                .name()
                .unwrap_or("unnamed")
                .to_string();
            let line = format!(
                "[{}] thread={} location={} panic={}\n",
                chrono::Utc::now().to_rfc3339(),
                thread,
                loc,
                redact_panic_text(msg)
            );
            let path = crash_log_path();
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            if let Ok(mut file) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)
            {
                use std::io::Write;
                let _ = file.write_all(line.as_bytes());
            }
            eprintln!("[solara-panic] {line}");
            previous(info);
        }));
    });
}
#[tauri::command]
async fn initialize_launcher() -> Result<(), String> {
    let _operation = crate::app_updates::OPERATIONS.read().await;
    engine::initialize().await.map_err(err)?;
    crate::app_updates::READY.store(true, Ordering::SeqCst);
    Ok(())
}
#[tauri::command]
async fn launcher_snapshot() -> Result<Value, String> {
    engine::snapshot().await.map_err(err)
}
#[tauri::command]
async fn minecraft_versions() -> Result<Value, String> {
    engine::versions().await.map_err(err)
}
#[tauri::command]
async fn loader_versions(game_version: String, loader: String) -> Result<Value, String> {
    engine::loader_versions(&game_version, &loader)
        .await
        .map_err(err)
}
#[tauri::command]
async fn create_instance(
    name: String,
    game_version: String,
    loader: String,
    loader_version: String,
    options: Option<engine::CreateInstanceOptions>,
) -> Result<Value, String> {
    Box::pin(async move {
        let _operation = crate::app_updates::OPERATIONS.read().await;
        engine::create_instance_with_options(
            name,
            game_version,
            loader,
            loader_version,
            options.unwrap_or_default(),
        )
        .await
        .map_err(err)
    })
    .await
}
#[tauri::command]
async fn update_instance_meta(id: String, patch: engine::InstanceMetaPatch) -> Result<Value, String> {
    let _operation = crate::app_updates::OPERATIONS.read().await;
    engine::update_instance_meta(&id, patch).await.map_err(err)
}
#[tauri::command]
async fn repair_instance(id: String) -> Result<String, String> {
    Box::pin(async move {
        let _operation = crate::app_updates::OPERATIONS.read().await;
        engine::repair_instance(id).await.map_err(err)
    })
    .await
}
#[tauri::command]
async fn duplicate_instance(id: String, name: Option<String>) -> Result<Value, String> {
    let _operation = crate::app_updates::OPERATIONS.read().await;
    engine::duplicate_instance(id, name).await.map_err(err)
}
#[tauri::command]
async fn retry_instance(id: String) -> Result<String, String> {
    let _operation = crate::app_updates::OPERATIONS.read().await;
    engine::retry_instance(id).await.map_err(err)
}
#[tauri::command]
async fn delete_instance(id: String) -> Result<(), String> {
    let _operation = crate::app_updates::OPERATIONS.read().await;
    engine::delete_instance(&id).await.map_err(err)
}
#[tauri::command]
async fn launch_instance(app: tauri::AppHandle, id: String) -> Result<Value, String> {
    Box::pin(async move {
        let _operation = crate::app_updates::OPERATIONS.read().await;
        let emit_app = app.clone();
        let emit_id = id.clone();
        let exit_app = app.clone();
        let exit_id = id.clone();
        let error_id = id.clone();
        let res = engine::launch_with_events(
            id,
            move |stage, detail| {
                use tauri::Emitter;
                let _ = emit_app.emit_to(
                    "main",
                    "launch-progress",
                    serde_json::json!({
                        "instanceId": emit_id,
                        "stage": stage,
                        "detail": detail,
                    }),
                );
            },
            move |status| {
                use tauri::Emitter;
                let _ = exit_app.emit_to("main", "game-exited", serde_json::json!({
                    "instanceId": exit_id,
                    "exitCode": status.code(),
                    "success": status.success(),
                }));
            },
        )
        .await;
        if let Err(ref e) = res {
            use tauri::Emitter;
            let _ = app.emit_to(
                "main",
                "launch-progress",
                serde_json::json!({
                    "instanceId": error_id,
                    "stage": "error",
                    "detail": err(anyhow::anyhow!("{e:#}")),
                }),
            );
        }
        res.map_err(err)
    })
    .await
}
#[tauri::command]
async fn stop_instance(id: String) -> Result<(), String> {
    let _operation = crate::app_updates::OPERATIONS.read().await;
    engine::stop(id).await.map_err(err)
}
#[tauri::command]
async fn edit_instance(
    id: String,
    name: String,
    memory: u32,
    java_path: String,
) -> Result<(), String> {
    let _operation = crate::app_updates::OPERATIONS.read().await;
    engine::edit(id, name, memory, java_path).await.map_err(err)
}
#[tauri::command]
async fn instance_files(id: String, path: String) -> Result<Value, String> {
    engine::list_files(&id, &path).await.map_err(err)
}
#[tauri::command]
async fn instance_text(id: String, path: String) -> Result<Value, String> {
    engine::read_text(&id, &path).await.map_err(err)
}
#[tauri::command]
async fn instance_image(id: String, path: String) -> Result<String, String> {
    engine::read_image(&id, &path).await.map_err(err)
}
#[tauri::command]
async fn instance_trash(id: String) -> Result<Value, String> {
    engine::list_trash(&id).await.map_err(err)
}
#[tauri::command]
async fn instance_mods(id: String) -> Result<Value, String> {
    engine::list_mods(&id).await.map_err(err)
}
#[tauri::command]
async fn toggle_instance_mod(id: String, path: String, enabled: bool) -> Result<(), String> {
    let _operation = crate::app_updates::OPERATIONS.read().await;
    engine::toggle_mod(&id, &path, enabled).await.map_err(err)
}
#[tauri::command]
async fn import_instance_mod(
    id: String,
    file_name: Option<String>,
    bytes_base64: Option<String>,
    source_path: Option<String>,
) -> Result<Value, String> {
    let _operation = crate::app_updates::OPERATIONS.read().await;
    engine::import_mod(
        &id,
        file_name.as_deref(),
        bytes_base64.as_deref(),
        source_path.as_deref(),
    )
    .await
    .map_err(err)
}
#[tauri::command]
async fn instance_backups(id: String) -> Result<Value, String> {
    engine::list_backups(&id).await.map_err(err)
}
#[tauri::command]
async fn instance_backup_action(
    id: String,
    action: String,
    backup_id: Option<String>,
    name: Option<String>,
) -> Result<Value, String> {
    let _operation = crate::app_updates::OPERATIONS.read().await;
    engine::backup_action(&id, &action, backup_id.as_deref(), name.as_deref())
        .await
        .map_err(err)
}
#[tauri::command]
async fn instance_file_action(
    id: String,
    action: String,
    path: String,
    target: Option<String>,
    content: Option<String>,
    expected_sha256: Option<String>,
) -> Result<Value, String> {
    let _operation = crate::app_updates::OPERATIONS.read().await;
    engine::file_action(
        &id,
        &action,
        &path,
        target.as_deref(),
        content.as_deref(),
        expected_sha256.as_deref(),
    )
    .await
    .map_err(err)
}
#[tauri::command]
async fn instance_logs(id: String) -> Result<String, String> {
    engine::logs(&id).await.map_err(err)
}
#[tauri::command]
async fn open_instance_folder(
    app: tauri::AppHandle,
    id: String,
    path: Option<String>,
) -> Result<(), String> {
    let e = engine::get().await.map_err(err)?;
    let req = path.as_deref().unwrap_or("").trim();
    let mut p = match crate::game_core::files::resolve(&e.instances, &id, req, false).await {
        Ok(resolved) => resolved,
        Err(_) if !req.is_empty() => {
            if let Ok(safe_new) =
                crate::game_core::files::resolve(&e.instances, &id, req, true).await
            {
                let _ = tokio::fs::create_dir_all(&safe_new).await;
                if safe_new.is_dir() {
                    safe_new
                } else {
                    crate::game_core::files::resolve(&e.instances, &id, "", false)
                        .await
                        .map_err(err)?
                }
            } else {
                crate::game_core::files::resolve(&e.instances, &id, "", false)
                    .await
                    .map_err(err)?
            }
        }
        Err(e) => return Err(err(e)),
    };
    if !p.is_dir() {
        p = p.parent().ok_or("File folder unavailable")?.to_path_buf();
    }
    app.opener()
        .open_path(p.to_string_lossy().to_string(), None::<String>)
        .map_err(err)
}
#[tauri::command]
async fn open_instance_file(
    app: tauri::AppHandle,
    id: String,
    path: String,
) -> Result<(), String> {
    let e = engine::get().await.map_err(err)?;
    let p = crate::game_core::files::resolve(&e.instances, &id, path.trim(), false)
        .await
        .map_err(err)?;
    if !p.is_file() {
        return Err("Target is not a valid file".into());
    }
    app.opener()
        .open_path(p.to_string_lossy().to_string(), None::<String>)
        .map_err(err)
}
#[tauri::command]
async fn install_content(
    instance_id: Option<String>,
    project_id: String,
    content_type: String,
    version_id: Option<String>,
    world: Option<String>,
) -> Result<Value, String> {
    Box::pin(async move {
        let _operation = crate::app_updates::OPERATIONS.read().await;
        engine::install_content(instance_id, project_id, content_type, version_id, world)
            .await
            .map_err(err)
    })
    .await
}
#[tauri::command]
async fn download_action(id: String, action: String) -> Result<(), String> {
    if action == "cancel" {
        return engine::download_action(&id, &action).await.map_err(err);
    }
    let _operation = crate::app_updates::OPERATIONS.read().await;
    engine::download_action(&id, &action).await.map_err(err)
}
#[tauri::command]
async fn clear_finished_downloads() -> Result<(), String> {
    engine::clear_finished_downloads().await.map_err(err)
}
#[tauri::command]
async fn launcher_settings() -> Result<engine::Preferences, String> {
    engine::preferences().await.map_err(err)
}
#[tauri::command]
async fn save_launcher_settings(
    memory: u32,
    java_path: String,
    discord: Option<bool>,
) -> Result<(), String> {
    let _operation = crate::app_updates::OPERATIONS.read().await;
    let _ = discord;
    engine::save_preferences(memory, java_path)
        .await
        .map_err(err)
}
#[tauri::command]
async fn save_launcher_settings_full(preferences: engine::Preferences) -> Result<engine::Preferences, String> {
    let _operation = crate::app_updates::OPERATIONS.read().await;
    engine::save_preferences_full(preferences).await.map_err(err)
}
#[tauri::command]
async fn instance_packs(id: String, kind: String, world: Option<String>) -> Result<Value, String> {
    engine::list_packs(&id, &kind, world.as_deref())
        .await
        .map_err(err)
}
#[tauri::command]
async fn toggle_instance_pack(
    id: String,
    kind: String,
    path: String,
    enabled: bool,
) -> Result<(), String> {
    let _operation = crate::app_updates::OPERATIONS.read().await;
    engine::toggle_pack(&id, &kind, &path, enabled)
        .await
        .map_err(err)
}
#[tauri::command]
async fn import_instance_pack(
    id: String,
    kind: String,
    world: Option<String>,
    file_name: Option<String>,
    bytes_base64: Option<String>,
    source_path: Option<String>,
) -> Result<Value, String> {
    let _operation = crate::app_updates::OPERATIONS.read().await;
    engine::import_pack(
        &id,
        &kind,
        world.as_deref(),
        file_name.as_deref(),
        bytes_base64.as_deref(),
        source_path.as_deref(),
    )
    .await
    .map_err(err)
}
#[tauri::command]
async fn check_instance_content_updates(id: String) -> Result<Value, String> {
    engine::check_content_updates(&id).await.map_err(err)
}
#[tauri::command]
async fn apply_instance_content_updates(id: String, items: Vec<Value>) -> Result<Value, String> {
    Box::pin(async move {
        let _operation = crate::app_updates::OPERATIONS.read().await;
        engine::apply_content_updates(&id, items).await.map_err(err)
    })
    .await
}
#[tauri::command]
async fn instance_worlds(id: String) -> Result<Value, String> {
    engine::list_worlds(&id).await.map_err(err)
}
#[tauri::command]
async fn rename_instance_world(id: String, folder: String, new_name: String) -> Result<(), String> {
    let _operation = crate::app_updates::OPERATIONS.read().await;
    engine::rename_world(&id, &folder, &new_name)
        .await
        .map_err(err)
}
#[tauri::command]
async fn launcher_screenshots(instance_id: Option<String>) -> Result<Value, String> {
    engine::list_screenshots(instance_id.as_deref())
        .await
        .map_err(err)
}
#[tauri::command]
async fn launcher_screenshot_image(instance_id: String, path: String) -> Result<String, String> {
    engine::read_screenshot_image(&instance_id, &path)
        .await
        .map_err(err)
}
#[tauri::command]
async fn instance_disk_logs(id: String) -> Result<Value, String> {
    engine::list_disk_logs(&id).await.map_err(err)
}
#[tauri::command]
async fn instance_disk_log(id: String, name: String) -> Result<String, String> {
    engine::read_disk_log(&id, &name).await.map_err(err)
}
#[tauri::command]
async fn detect_java_runtimes() -> Result<Value, String> {
    engine::detect_java_runtimes().await.map_err(err)
}
#[tauri::command]
async fn launcher_storage_summary() -> Result<Value, String> {
    engine::storage_summary().await.map_err(err)
}
#[tauri::command]
async fn clear_launcher_cache() -> Result<Value, String> {
    let _operation = crate::app_updates::OPERATIONS.read().await;
    engine::clear_unused_cache().await.map_err(err)
}
#[tauri::command]
async fn saved_skins_list() -> Result<Value, String> {
    engine::saved_skins_list().await.map_err(err)
}
#[tauri::command]
async fn saved_skins_save(
    id: Option<String>,
    name: String,
    variant: String,
    png_data_url: String,
) -> Result<Value, String> {
    engine::saved_skins_save(id, name, variant, png_data_url)
        .await
        .map_err(err)
}
#[tauri::command]
async fn saved_skins_delete(id: String) -> Result<(), String> {
    engine::saved_skins_delete(&id).await.map_err(err)
}
#[tauri::command]
async fn profile_skins_and_capes() -> Result<Value, String> {
    engine::fetch_profile_skins_and_capes().await.map_err(err)
}
#[tauri::command]
async fn upload_minecraft_skin(variant: String, png_data_url: String) -> Result<Value, String> {
    engine::upload_minecraft_skin(&variant, &png_data_url)
        .await
        .map_err(err)
}
#[tauri::command]
async fn set_minecraft_cape(cape_id: Option<String>) -> Result<(), String> {
    engine::set_minecraft_cape(cape_id.as_deref())
        .await
        .map_err(err)
}
#[tauri::command]
async fn export_diagnostics() -> Result<Value, String> {
    engine::export_diagnostics().await.map_err(err)
}
#[tauri::command]
async fn open_data_folder(app: tauri::AppHandle) -> Result<(), String> {
    let dir = engine::data_dir().await.map_err(err)?;
    app.opener()
        .open_path(dir.to_string_lossy().to_string(), None::<String>)
        .map_err(err)
}
#[tauri::command]
async fn create_local_server(
    name: String,
    version: String,
    java_path: String,
    memory: u32,
    port: u16,
    eula: bool,
) -> Result<String, String> {
    let _operation = crate::app_updates::OPERATIONS.read().await;
    engine::create_local(name, version, java_path, memory, port, eula)
        .await
        .map_err(err)
}
#[tauri::command]
async fn local_server_action(id: String, action: String) -> Result<(), String> {
    let _operation = crate::app_updates::OPERATIONS.read().await;
    engine::get()
        .await
        .map_err(err)?
        .local
        .action(&id, &action)
        .await
        .map_err(err)
}
#[tauri::command]
async fn local_server_logs(id: String) -> Result<String, String> {
    engine::get()
        .await
        .map_err(err)?
        .local
        .logs(&id)
        .await
        .map_err(err)
}
#[tauri::command]
async fn local_server_command(id: String, command: String) -> Result<(), String> {
    let _operation = crate::app_updates::OPERATIONS.read().await;
    engine::get()
        .await
        .map_err(err)?
        .local
        .command(&id, &command)
        .await
        .map_err(err)
}
#[tauri::command]
async fn solara_profile() -> Result<Option<Value>, String> {
    identity::profile().await.map_err(err)
}
#[tauri::command]
async fn solara_sign_in(app: tauri::AppHandle) -> Result<Value, String> {
    let _operation = crate::app_updates::OPERATIONS.read().await;
    identity::sign_in(|url| {
        app.opener().open_url(url, None::<String>)?;
        Ok(())
    })
    .await
    .map_err(err)
}
#[tauri::command]
async fn solara_sign_out() -> Result<(), String> {
    let _operation = crate::app_updates::OPERATIONS.read().await;
    identity::sign_out().await.map_err(err)
}
#[tauri::command]
async fn solara_request(
    method: String,
    path: String,
    body: Option<Value>,
) -> Result<Value, String> {
    identity::request(&method, &path, body).await.map_err(err)
}
static MINECRAFT_LOGIN: tokio::sync::Mutex<Option<crate::game_core::download::Cancellation>> =
    tokio::sync::Mutex::const_new(None);
#[tauri::command]
async fn minecraft_sign_out(id: String) -> Result<(), String> {
    let _operation = crate::app_updates::OPERATIONS.read().await;
    engine::minecraft_logout(&id).await.map_err(err)
}
#[tauri::command]
async fn select_minecraft_account(id: String) -> Result<(), String> {
    let _operation = crate::app_updates::OPERATIONS.read().await;
    engine::minecraft_select(&id).await.map_err(err)
}
#[tauri::command]
async fn cancel_minecraft_sign_in() -> Result<(), String> {
    if let Some(cancel) = MINECRAFT_LOGIN.lock().await.as_ref() {
        cancel.cancel()
    };
    Ok(())
}
#[tauri::command]
async fn minecraft_sign_in(app: tauri::AppHandle) -> Result<(), String> {
    let _operation = crate::app_updates::OPERATIONS.read().await;
    let cancel = crate::game_core::download::Cancellation::default();
    {
        let mut active = MINECRAFT_LOGIN.lock().await;
        if active.is_some() {
            return Err("A Minecraft sign-in is already open".into());
        };
        *active = Some(cancel.clone());
    }
    let notice_cancel = cancel.clone();
    let result = engine::minecraft_login(&cancel, |prompt| {
        use tauri::Emitter;
        if app
            .emit_to("main", "minecraft-login-code", &prompt)
            .is_err()
        {
            notice_cancel.cancel();
            return;
        }
        // Credentials stay in native memory. The user enters this one-time code
        // only on the official Microsoft site in their system browser.
        let _ = app.opener().open_url(prompt.url, None::<String>);
    })
    .await;
    *MINECRAFT_LOGIN.lock().await = None;
    result.map_err(err)
}
#[tauri::command]
fn launcher_capabilities() -> Value {
    engine::capabilities()
}
#[tauri::command]
fn open_external(app: tauri::AppHandle, url: String) -> Result<(), String> {
    let u = reqwest::Url::parse(&url).map_err(err)?;
    if u.scheme() != "https" || !u.username().is_empty() || u.password().is_some() {
        return Err("Only HTTPS web destinations are allowed".into());
    };
    app.opener()
        .open_url(u.to_string(), None::<String>)
        .map_err(err)
}
#[tauri::command]
fn window_action(window: tauri::Window, action: String) -> Result<(), String> {
    match action.as_str() {
        "minimize" => window.minimize().map_err(err),
        "maximize" => {
            if window.is_maximized().map_err(err)? {
                window.unmaximize().map_err(err)
            } else {
                window.maximize().map_err(err)
            }
        }
        "show" => window.show().map_err(err),
        "close" => window.close().map_err(err),
        _ => Err("Unsupported window action".into()),
    }
}
#[tauri::command]
fn start_dragging(window: tauri::Window) -> Result<(), String> {
    window.start_dragging().map_err(err)
}
#[tauri::command]
async fn check_update(app: tauri::AppHandle) -> Result<Value, String> {
    Box::pin(async move { crate::app_updates::trigger(&app).await.map_err(err) }).await
}
#[tauri::command]
async fn install_update(app: tauri::AppHandle) -> Result<Value, String> {
    Box::pin(async move { crate::app_updates::install_now(&app).await.map_err(err) }).await
}
#[tauri::command]
fn app_update_status() -> Value {
    crate::app_updates::snapshot()
}
pub fn run() {
    install_panic_hook();
    let closing = Arc::new(AtomicBool::new(false));
    let handler: Box<dyn Fn(tauri::ipc::Invoke<tauri::Wry>) -> bool + Send + Sync> =
        Box::new(tauri::generate_handler![
            initialize_launcher,
            launcher_snapshot,
            minecraft_versions,
            loader_versions,
            create_instance,
            update_instance_meta,
            repair_instance,
            duplicate_instance,
            delete_instance,
            launch_instance,
            retry_instance,
            stop_instance,
            edit_instance,
            instance_logs,
            instance_files,
            instance_text,
            instance_image,
            instance_trash,
            instance_mods,
            instance_file_action,
            toggle_instance_mod,
            import_instance_mod,
            instance_packs,
            toggle_instance_pack,
            import_instance_pack,
            check_instance_content_updates,
            apply_instance_content_updates,
            instance_worlds,
            rename_instance_world,
            launcher_screenshots,
            launcher_screenshot_image,
            instance_disk_logs,
            instance_disk_log,
            instance_backups,
            instance_backup_action,
            open_instance_folder,
            open_instance_file,
            install_content,
            download_action,
            clear_finished_downloads,
            launcher_settings,
            save_launcher_settings,
            save_launcher_settings_full,
            detect_java_runtimes,
            launcher_storage_summary,
            clear_launcher_cache,
            saved_skins_list,
            saved_skins_save,
            saved_skins_delete,
            profile_skins_and_capes,
            upload_minecraft_skin,
            set_minecraft_cape,
            export_diagnostics,
            open_data_folder,
            create_local_server,
            local_server_action,
            local_server_logs,
            local_server_command,
            solara_profile,
            solara_sign_in,
            solara_sign_out,
            solara_request,
            minecraft_sign_in,
            cancel_minecraft_sign_in,
            launcher_capabilities,
            minecraft_sign_out,
            select_minecraft_account,
            open_external,
            window_action,
            start_dragging,
            check_update,
            install_update,
            app_update_status
        ]);
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .invoke_handler(move |invoke| {
            let webview = invoke.message.webview_ref();
            let trusted = webview
                .url()
                .map(|url| {
                    let packaged = (url.scheme() == "tauri" && url.host_str() == Some("localhost"))
                        || (["http", "https"].contains(&url.scheme())
                            && url.host_str() == Some("tauri.localhost"));
                    let development = cfg!(debug_assertions)
                        && url.scheme() == "http"
                        && url.host_str() == Some("127.0.0.1")
                        && url.port() == Some(1420);
                    packaged || development
                })
                .unwrap_or(false);
            if webview.label() != "main" || !trusted {
                invoke
                    .resolver
                    .reject("Native commands are available only to the local Solara main window");
                true
            } else {
                handler(invoke)
            }
        })
        .build(tauri::generate_context!())
        .expect("Could not initialize Solara desktop");
    crate::app_updates::start(app.handle().clone());
    app.run(move |app, event| match event {
        tauri::RunEvent::WindowEvent {
            label,
            event: WindowEvent::CloseRequested { api, .. },
            ..
        } if label == "main" => {
            api.prevent_close();
            if closing.swap(true, Ordering::SeqCst) {
                return;
            };
            let app = app.clone();
            let closing = closing.clone();
            tauri::async_runtime::spawn(async move {
                match engine::stop_all().await {
                    Ok(()) => {
                        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                        app.exit(0);
                    }
                    Err(_) => {
                        closing.store(false, Ordering::SeqCst);
                        use tauri::Emitter;
                        let _ = app.emit(
                            "shutdown-error",
                            "Local server is still saving. Check its console and stop it before closing Solara.",
                        );
                    }
                }
            });
        }
        _ => {}
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redact_panic_text_removes_tokens_and_newlines() {
        let raw = "panic\nwith Bearer abc.def.ghi and access_token=secret123 password:my_pass\r\ndone";
        let cleaned = redact_panic_text(raw);
        assert!(!cleaned.contains('\n'));
        assert!(!cleaned.contains('\r'));
        assert!(!cleaned.contains("abc.def.ghi"));
        assert!(!cleaned.contains("secret123"));
        assert!(!cleaned.contains("my_pass"));
        assert!(cleaned.contains("Bearer [REDACTED]"));
        assert!(cleaned.contains("access_token=[REDACTED]"));
        assert!(cleaned.contains("password=[REDACTED]"));
    }

    #[test]
    fn boxed_shell_command_futures_stay_small() {
        let install_fut = install_content(None, "proj".into(), "mod".into(), None, None);
        assert!(std::mem::size_of_val(&install_fut) <= 2048);
        drop(install_fut);

        let updates_fut = apply_instance_content_updates("inst-1".into(), Vec::new());
        assert!(std::mem::size_of_val(&updates_fut) <= 2048);
        drop(updates_fut);

        let create_fut = create_instance("n".into(), "1.21.4".into(), "fabric".into(), "0.16.10".into(), None);
        assert!(std::mem::size_of_val(&create_fut) <= 2048);
        drop(create_fut);

        let repair_fut = repair_instance("inst-1".into());
        assert!(std::mem::size_of_val(&repair_fut) <= 2048);
        drop(repair_fut);
    }
}
