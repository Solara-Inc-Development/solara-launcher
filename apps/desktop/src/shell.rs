use crate::{engine, identity};
use serde_json::{json, Value};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tauri::WindowEvent;
use tauri_plugin_opener::OpenerExt;
use tauri_plugin_updater::UpdaterExt;
fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
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
async fn create_instance(
    name: String,
    game_version: String,
    loader: String,
    loader_version: String,
) -> Result<Value, String> {
    let _operation = crate::app_updates::OPERATIONS.read().await;
    engine::create_instance(name, game_version, loader, loader_version)
        .await
        .map_err(err)
}
#[tauri::command]
async fn retry_instance(id: String) -> Result<String, String> {
    let _operation = crate::app_updates::OPERATIONS.read().await;
    engine::retry_instance(id).await.map_err(err)
}
#[tauri::command]
async fn launch_instance(id: String) -> Result<Value, String> {
    let _operation = crate::app_updates::OPERATIONS.read().await;
    engine::launch(id).await.map_err(err)
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
    let mut p =
        crate::game_core::files::resolve(&e.instances, &id, path.as_deref().unwrap_or(""), false)
            .await
            .map_err(err)?;
    if !p.is_dir() {
        p = p.parent().ok_or("File folder unavailable")?.to_path_buf();
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
) -> Result<(), String> {
    let _operation = crate::app_updates::OPERATIONS.read().await;
    engine::install_content(instance_id, project_id, content_type, version_id)
        .await
        .map_err(err)
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
async fn launcher_settings() -> Result<engine::Preferences, String> {
    engine::preferences().await.map_err(err)
}
#[tauri::command]
async fn save_launcher_settings(
    memory: u32,
    java_path: String,
    discord: bool,
) -> Result<(), String> {
    let _operation = crate::app_updates::OPERATIONS.read().await;
    engine::save_preferences(memory, java_path, discord)
        .await
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
        "close" => window.close().map_err(err),
        _ => Err("Unsupported window action".into()),
    }
}
fn updater(app: &tauri::AppHandle) -> Result<tauri_plugin_updater::Updater, String> {
    let key = option_env!("SOLARA_UPDATER_PUBLIC_KEY")
        .unwrap_or(include_str!("../updater.pub"))
        .trim();
    if key.is_empty() {
        return Err(
            "Verified updates are not configured: trusted public signing key missing".into(),
        );
    };
    app.updater_builder()
        .pubkey(key)
        .endpoints(vec![
            "https://api.solara-inc.in/v1/launcher/update/windows/x86_64"
                .parse()
                .map_err(err)?,
        ])
        .map_err(err)?
        .build()
        .map_err(err)
}
#[tauri::command]
async fn check_update(app: tauri::AppHandle) -> Result<Value, String> {
    let result = updater(&app)?.check().await.map_err(err)?;
    Ok(match result {
        Some(u) => json!({"available":true,"version":u.version}),
        None => json!({"available":false}),
    })
}
#[tauri::command]
fn app_update_status() -> Value {
    crate::app_updates::snapshot()
}
pub fn run() {
    let closing = Arc::new(AtomicBool::new(false));
    let presence_stop = Arc::new(AtomicBool::new(false));
    let thread_stop = presence_stop.clone();
    let handler: Box<dyn Fn(tauri::ipc::Invoke<tauri::Wry>) -> bool + Send + Sync> =
        Box::new(tauri::generate_handler![
            initialize_launcher,
            launcher_snapshot,
            minecraft_versions,
            create_instance,
            launch_instance,
            retry_instance,
            stop_instance,
            edit_instance,
            instance_logs,
            instance_files,
            instance_text,
            instance_trash,
            instance_mods,
            instance_file_action,
            toggle_instance_mod,
            open_instance_folder,
            install_content,
            download_action,
            launcher_settings,
            save_launcher_settings,
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
            check_update,
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
    let handle = app.handle().clone();
    tauri::async_runtime::spawn(async move {
        use discord_rich_presence::{activity::Activity, DiscordIpc, DiscordIpcClient};
        let id = option_env!("SOLARA_DISCORD_CLIENT_ID").unwrap_or_default();
        if id.is_empty() {
            return;
        };
        let mut client = DiscordIpcClient::new(id);
        let mut connected = false;
        loop {
            if thread_stop.load(Ordering::Relaxed) {
                if connected {
                    let _ = client.clear_activity();
                    let _ = client.close();
                }
                break;
            };
            let enabled = engine::preferences()
                .await
                .map(|p| p.discord)
                .unwrap_or(false);
            if enabled {
                if !connected {
                    connected = client.connect().is_ok()
                };
                if connected {
                    let running = engine::game_running().await;
                    if client
                        .set_activity(Activity::new().state(if running {
                            "Minecraft running"
                        } else {
                            "Solara open"
                        }))
                        .is_err()
                    {
                        connected = false;
                        let _ = client.close();
                    }
                }
            } else if connected {
                let _ = client.clear_activity();
                let _ = client.close();
                connected = false
            };
            tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        }
    });
    app.run(move|app,event|match event{tauri::RunEvent::WindowEvent{label,event:WindowEvent::CloseRequested{api,..},..}if label=="main"=>{api.prevent_close();if closing.swap(true,Ordering::SeqCst){return};let app=app.clone();let closing=closing.clone();let stop=presence_stop.clone();tauri::async_runtime::spawn(async move{match engine::stop_all().await{Ok(())=>{stop.store(true,Ordering::Relaxed);tokio::time::sleep(std::time::Duration::from_millis(100)).await;app.exit(0)},Err(_)=>{closing.store(false,Ordering::SeqCst);use tauri::Emitter;let _=app.emit("shutdown-error","Local server is still saving. Check its console and stop it before closing Solara.");}}});},_=>{}});
    let _ = handle;
}
