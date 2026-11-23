#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
#![recursion_limit = "256"]

use anyhow::Context;
use native_dialog::{DialogBuilder, MessageLevel};
use std::env;
use std::sync::{
    Once,
    atomic::{AtomicBool, Ordering},
};
use tauri::{Listener, Manager};
use tauri_plugin_fs::FsExt;
use theseus::prelude::*;

mod api;
pub mod app_updates;
pub mod update_policy;

#[cfg(target_os = "macos")]
mod macos;

pub mod updater_impl;
pub mod updater_impl_noop;

pub use updater_impl::*;

static PANIC_HOOK_INIT: Once = Once::new();

pub fn redact_panic_text(raw: &str) -> String {
    let mut cleaned = raw.replace(['\r', '\n'], " ");
    for secret_key in [
        "access_token",
        "refresh_token",
        "minecraft_token",
        "password",
        "secret",
        "token",
        "authorization",
        "Bearer ",
        "bearer ",
    ] {
        while let Some(pos) = cleaned.find(secret_key) {
            let after = pos + secret_key.len();
            let rest = &cleaned[after..];
            let trimmed = rest.trim_start_matches([' ', ':', '=']);
            let skip_ws = rest.len() - trimmed.len();
            let token_len = trimmed
                .find([' ', ',', ';', '"', '\''])
                .unwrap_or(trimmed.len());
            if token_len > 0 && &trimmed[..token_len] != "[REDACTED]" {
                let end = after + skip_ws + token_len;
                cleaned.replace_range(pos..end, &format!("{secret_key}=[REDACTED]"));
            } else {
                break;
            }
        }
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

// Should be called in launcher initialization
#[tracing::instrument(skip_all)]
#[tauri::command]
async fn initialize_state(
    app: tauri::AppHandle,
    events: tauri::ipc::Channel<tauri::ipc::InvokeResponseBody>,
) -> api::Result<()> {
    Box::pin(async move {
        tracing::info!("Initializing app event state...");
        theseus::EventState::init(app.clone(), events).await?;

        tracing::info!("Initializing app state...");
        State::init(app.config().identifier.clone()).await?;

        let state = State::get().await?;
        app.asset_protocol_scope()
            .allow_directory(state.directories.caches_dir(), true)?;
        app.asset_protocol_scope()
            .allow_directory(state.directories.caches_dir().join("icons"), true)?;
        app.asset_protocol_scope()
            .allow_directory(state.directories.icon_dir(), true)?;
        app.fs_scope()
            .allow_directory(state.directories.instances_dir(), true)?;

        app_updates::READY.store(true, Ordering::SeqCst);
        Ok(())
    })
    .await
}

// Should be called once Vue has mounted the app
#[tracing::instrument(skip_all)]
#[tauri::command]
fn show_window(app: tauri::AppHandle) {
    let win = app.get_window("main").unwrap();
    if let Err(e) = win.show() {
        DialogBuilder::message()
            .set_level(MessageLevel::Error)
            .set_title("Initialization error")
            .set_text(format!(
                "Cannot display application window due to an error:\n{e}"
            ))
            .alert()
            .show()
            .unwrap();
        panic!("cannot display application window")
    } else {
        let _ = win.set_focus();
    }
}

#[tauri::command]
fn is_dev() -> bool {
    cfg!(debug_assertions)
}

#[tauri::command]
fn are_updates_enabled() -> bool {
    env::var("SOLARA_EXTERNAL_UPDATE_PROVIDER").is_err()
        && app_updates::configured_pubkey().is_some()
}

// Window decorations remain locked off so the custom Solara title bar is always used.
#[tauri::command]
async fn toggle_decorations(_b: bool, window: tauri::Window) -> api::Result<()> {
    window
        .set_decorations(false)
        .context("failed to lock window decorations off")?;
    Ok(())
}

#[tauri::command]
fn restart_app(app: tauri::AppHandle) {
    app.restart();
}

#[derive(Default)]
struct RestartAfterPendingUpdate(AtomicBool);

#[tauri::command]
async fn set_restart_after_pending_update(
    should_restart: bool,
    restart: tauri::State<'_, RestartAfterPendingUpdate>,
) -> api::Result<()> {
    restart.0.store(should_restart, Ordering::Relaxed);
    Ok(())
}

pub const BUNDLED_LICENSE: &str = include_str!("../../../LICENSE");
pub const BUNDLED_THIRD_PARTY_NOTICES: &str = include_str!("../../../THIRD_PARTY_NOTICES.md");

fn main() {
    #[cfg(target_os = "windows")]
    if app_updates::try_handle_legacy_update_handoff() {
        return;
    }
    app_updates::wait_if_post_update_relaunch();

    install_panic_hook();
    theseus::install_crypto_provider();

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("failed to create app async runtime");
    tauri::async_runtime::set(runtime.handle().clone());

    #[cfg(feature = "export-app-events")]
    theseus::export_app_event_bindings(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../app-frontend/src/generated/app-events"),
    )
    .expect("failed to export app event TypeScript bindings");

    let tauri_context = tauri::generate_context!();

    let _log_guard = theseus::start_logger(&tauri_context.config().identifier);

    tracing::info!("Initialized tracing subscriber. Loading Solara Launcher!");

    let mut builder = tauri::Builder::default();

    #[cfg(target_os = "macos")]
    {
        builder = builder
            .menu(macos::menu::create)
            .on_menu_event(macos::menu::handle_event);
    }

    builder = builder
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            if let Some(payload) = args.get(1) {
                tracing::info!("Handling command-line deep link");
                let payload = payload.clone();
                tauri::async_runtime::spawn(api::utils::handle_command(
                    payload,
                ));
            }

            if let Some(win) = app.get_window("main") {
                let _ = win.set_focus();
            }
        }))
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init());

    #[cfg(target_os = "windows")]
    {
        use windows::Win32::UI::WindowsAndMessaging::{
            SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
            SWP_NOZORDER, SetWindowPos,
        };

        builder = builder.plugin(
            tauri::plugin::Builder::<_, ()>::new("window-frame")
                .on_window_ready(|window| {
                    if window.label() != "main" {
                        return;
                    }

                    let hwnd = match window.hwnd() {
                        Ok(hwnd) => hwnd,
                        Err(e) => {
                            tracing::warn!("Failed to get window handle: {e}");
                            return;
                        }
                    };

                    if let Err(e) = unsafe {
                        SetWindowPos(
                            hwnd,
                            None,
                            0,
                            0,
                            0,
                            0,
                            SWP_FRAMECHANGED
                                | SWP_NOACTIVATE
                                | SWP_NOMOVE
                                | SWP_NOSIZE
                                | SWP_NOZORDER,
                        )
                    } {
                        tracing::warn!("Failed to refresh window frame: {e}");
                    }
                })
                .build(),
        );
    }

    let window_state_flags = tauri_plugin_window_state::StateFlags::POSITION
        | tauri_plugin_window_state::StateFlags::SIZE
        | tauri_plugin_window_state::StateFlags::MAXIMIZED;
    #[cfg(target_os = "windows")]
    let window_state_flags =
        window_state_flags | tauri_plugin_window_state::StateFlags::DECORATIONS;

    builder = builder
        .plugin(
            tauri_plugin_window_state::Builder::default()
                .with_filename("app-window-state.json")
                .with_denylist(&["signin"])
                .with_state_flags(window_state_flags)
                .build(),
        )
        .setup(|app| {
            app_updates::start(app.handle().clone());

            #[cfg(target_os = "macos")]
            {
                let payload = macos::deep_link::get_or_init_payload(app);

                let mtx_copy = payload.payload;
                app.listen("deep-link://new-url", move |url| {
                    let mtx_copy_copy = mtx_copy.clone();
                    let request = url.payload().to_owned();

                    let actual_request =
                        serde_json::from_str::<Vec<String>>(&request)
                            .ok()
                            .map(|mut x| x.remove(0))
                            .unwrap_or(request);

                    tauri::async_runtime::spawn(async move {
                        tracing::info!("Handling macOS deep link");

                        let mut payload = mtx_copy_copy.lock().await;
                        if payload.is_none() {
                            *payload = Some(actual_request.clone());
                        }

                        let _ =
                            api::utils::handle_command(actual_request).await;
                    });
                });
            };

            #[cfg(not(target_os = "macos"))]
            app.listen("deep-link://new-url", |url| {
                let payload = url.payload().to_owned();
                tracing::info!("Handling deep link");
                tauri::async_runtime::spawn(api::utils::handle_command(
                    payload,
                ));
            });

            #[cfg(not(target_os = "linux"))]
            if let Some(window) = app.get_window("main")
                && let Err(e) = window.set_shadow(true)
            {
                tracing::warn!("Failed to set window shadow: {e}");
            }

            Ok(())
        });

    builder = builder
        .plugin(api::auth::init())
        .plugin(api::onboarding_checklist::init())
        .plugin(api::import::init())
        .plugin(api::install::init())
        .plugin(api::instance::init())
        .plugin(api::logs::init())
        .plugin(api::jre::init())
        .plugin(api::metadata::init())
        .plugin(api::minecraft_skins::init())
        .plugin(api::process::init())
        .plugin(api::reports::init())
        .plugin(api::settings::init())
        .plugin(api::shortcuts::init())
        .plugin(api::tags::init())
        .plugin(api::users::init())
        .plugin(api::utils::init())
        .plugin(api::cache::init())
        .plugin(api::curseforge::init())
        .plugin(api::files::init())
        .plugin(api::worlds::init())
        .manage(PendingUpdateData)
        .manage(RestartAfterPendingUpdate::default())
        .invoke_handler(tauri::generate_handler![
            initialize_state,
            is_dev,
            are_updates_enabled,
            check_app_update,
            install_app_update,
            get_app_update_status,
            get_update_size,
            enqueue_update_for_installation,
            remove_enqueued_update,
            set_restart_after_pending_update,
            toggle_decorations,
            show_window,
            restart_app,
        ]);

    tracing::info!("Initializing Solara Launcher...");
    let app = builder.build(tauri_context);

    match app {
        Ok(app) => {
            app.run(|app, event| {
                let _ = app;

                if matches!(&event, tauri::RunEvent::ExitRequested { .. })
                    && let Err(error) = tauri::async_runtime::block_on(
                        theseus::minecraft_skins::flush_pending_skin_change(),
                    )
                {
                    tracing::warn!(
                        "Failed to flush pending Minecraft skin change before exit: {error}"
                    );
                }

                // Note: No silent RunEvent::Exit update installation. Updates only download and apply
                // when the user explicitly confirms via "Update Now" or "Restart to Update".

                #[cfg(target_os = "macos")]
                if let tauri::RunEvent::Opened { urls } = event {
                    tracing::info!("Handling webview open {urls:?}");

                    let file = urls
                        .into_iter()
                        .find_map(|url| url.to_file_path().ok());

                    if let Some(file) = file {
                        let payload =
                            macos::deep_link::get_or_init_payload(app);

                        let mtx_copy = payload.payload;
                        let request = file.to_string_lossy().to_string();
                        tauri::async_runtime::spawn(async move {
                            let mut payload = mtx_copy.lock().await;
                            if payload.is_none() {
                                *payload = Some(request.clone());
                            }

                            let _ = api::utils::handle_command(request).await;
                        });
                    }
                }
            });
        }
        Err(e) => {
            tracing::error!("Error while running tauri application: {:?}", e);

            #[cfg(target_os = "windows")]
            {
                if format!("{e:?}").contains(
                    "Runtime(CreateWebview(WebView2Error(WindowsError",
                ) {
                    DialogBuilder::message()
                        .set_level(MessageLevel::Error)
                        .set_title("Initialization error")
                        .set_text("Your Microsoft Edge WebView2 installation is corrupt.\n\nMicrosoft Edge WebView2 is required to run Solara Launcher.\n\nPlease repair or reinstall Microsoft Edge WebView2 Runtime.")
                        .alert()
                        .show()
                        .unwrap();

                    panic!("webview2 initialization failed")
                }
            }

            DialogBuilder::message()
                .set_level(MessageLevel::Error)
                .set_title("Initialization error")
                .set_text(format!(
                    "Cannot initialize application due to an error:\n{e:?}"
                ))
                .alert()
                .show()
                .unwrap();

            panic!("{1}: {:?}", e, "error while running tauri application")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rustls_process_provider_is_resolvable() {
        let _ = rustls::ClientConfig::builder();
    }

    #[test]
    fn panic_redaction_strips_bearer_and_oauth_tokens() {
        let raw = "failed with access_token=eyJhbGciOiJIUzI1NiJ9 and refresh_token:secret12345";
        let redacted = redact_panic_text(raw);
        assert!(!redacted.contains("eyJhbGciOiJIUzI1NiJ9"));
        assert!(!redacted.contains("secret12345"));
        assert!(redacted.contains("[REDACTED]"));
    }
}
