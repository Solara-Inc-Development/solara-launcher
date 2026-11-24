use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use chrono::Utc;
use discord_rich_presence::{
    DiscordIpc, DiscordIpcClient,
    activity::{Activity, Assets, Button, Timestamps},
};
use sqlx::SqlitePool;
use tokio::sync::RwLock;

use crate::state::ProcessManager;

pub const SOLARA_DISCORD_APP_ID: &str = "1574631435627659615";
pub const SOLARA_DISCORD_LOGO_URL: &str =
    "https://app.solara-inc.in/logo-512.png";
pub const SOLARA_WEBSITE_URL: &str = "https://app.solara-inc.in";

#[derive(Clone, Debug, PartialEq, Eq)]
struct PresenceSnapshot {
    details: String,
    state: String,
    started_at_ms: i64,
}

pub struct DiscordGuard {
    client: Arc<RwLock<DiscordIpcClient>>,
    connected: Arc<AtomicBool>,
    last_presence: Arc<RwLock<Option<PresenceSnapshot>>>,
}

pub fn format_loader_name(loader: &str) -> &'static str {
    match loader.to_ascii_lowercase().as_str() {
        "fabric" => "Fabric",
        "forge" => "Forge",
        "neoforge" | "neo" => "NeoForge",
        "quilt" => "Quilt",
        "vanilla" => "Vanilla",
        _ => "Minecraft",
    }
}

impl DiscordGuard {
    /// Initialize Solara's Discord IPC client without blocking startup if Discord is closed.
    pub fn init() -> crate::Result<DiscordGuard> {
        let dipc = DiscordIpcClient::new(SOLARA_DISCORD_APP_ID);

        Ok(DiscordGuard {
            client: Arc::new(RwLock::new(dipc)),
            connected: Arc::new(AtomicBool::new(false)),
            last_presence: Arc::new(RwLock::new(None)),
        })
    }

    pub fn is_connected(&self) -> bool {
        self.connected.load(Ordering::Relaxed)
    }

    /// Connect to a running Discord desktop client if not already connected.
    /// Must always run before `set_activity` / `clear_activity` because the IPC
    /// socket panics or errors if never connected.
    pub async fn retry_if_not_ready(&self) -> bool {
        if self.connected.load(Ordering::Relaxed) {
            return true;
        }

        let mut client = self.client.write().await;
        if self.connected.load(Ordering::Relaxed) {
            return true;
        }

        match client.connect() {
            Ok(()) => {
                self.connected.store(true, Ordering::Relaxed);
                tracing::info!(
                    "Connected to Discord Rich Presence (Solara App ID {})",
                    SOLARA_DISCORD_APP_ID
                );
                true
            }
            Err(err) => {
                tracing::debug!(
                    "Discord desktop client not detected yet: {err}"
                );
                false
            }
        }
    }

    /// Set activity with a single state string (legacy compatibility helper).
    pub async fn set_activity(
        &self,
        msg: &str,
        reconnect_if_fail: bool,
        pool: &SqlitePool,
    ) -> crate::Result<()> {
        self.set_rich_activity(
            "In Solara Launcher",
            msg,
            false,
            reconnect_if_fail,
            pool,
        )
        .await
    }

    /// Set rich Discord presence for an active Minecraft instance.
    pub async fn set_instance_activity(
        &self,
        instance_name: &str,
        loader: &str,
        game_version: &str,
        reconnect_if_fail: bool,
        pool: &SqlitePool,
    ) -> crate::Result<()> {
        let details = format!("Playing {instance_name}");
        let loader_label = format_loader_name(loader);
        let state = if game_version.trim().is_empty() {
            loader_label.to_string()
        } else {
            format!("{loader_label} {game_version}")
        };
        self.set_rich_activity(&details, &state, true, reconnect_if_fail, pool)
            .await
    }

    /// Set rich Solara activity (details + state + elapsed timer + logo + website button).
    pub async fn set_rich_activity(
        &self,
        details: &str,
        state_msg: &str,
        reset_timer_on_change: bool,
        reconnect_if_fail: bool,
        pool: &SqlitePool,
    ) -> crate::Result<()> {
        let settings = crate::state::Settings::get(pool).await?;
        if !settings.discord_rpc {
            let _ = self.clear_activity(false).await;
            return Ok(());
        }

        let started_at_ms = {
            let mut guard = self.last_presence.write().await;
            let reuse_ts = guard.as_ref().and_then(|prev| {
                if !reset_timer_on_change
                    || (prev.details == details && prev.state == state_msg)
                {
                    Some(prev.started_at_ms)
                } else {
                    None
                }
            });
            let ts = reuse_ts.unwrap_or_else(|| Utc::now().timestamp_millis());
            *guard = Some(PresenceSnapshot {
                details: details.to_string(),
                state: state_msg.to_string(),
                started_at_ms: ts,
            });
            ts
        };

        let _ = self
            .force_set_rich_activity(
                details,
                state_msg,
                started_at_ms,
                reconnect_if_fail,
            )
            .await;
        Ok(())
    }

    /// Low-level activity setter used directly when Discord IPC is ready.
    pub async fn force_set_activity(
        &self,
        msg: &str,
        reconnect_if_fail: bool,
    ) -> crate::Result<()> {
        self.force_set_rich_activity(
            "In Solara Launcher",
            msg,
            Utc::now().timestamp_millis(),
            reconnect_if_fail,
        )
        .await
    }

    async fn force_set_rich_activity(
        &self,
        details: &str,
        state_msg: &str,
        started_at_ms: i64,
        reconnect_if_fail: bool,
    ) -> crate::Result<()> {
        if !self.retry_if_not_ready().await {
            return Ok(());
        }

        let activity = Activity::new()
            .details(details)
            .state(state_msg)
            .timestamps(Timestamps::new().start(started_at_ms))
            .assets(
                Assets::new()
                    .large_image(SOLARA_DISCORD_LOGO_URL)
                    .large_text("Solara Launcher")
                    .large_url(SOLARA_WEBSITE_URL),
            )
            .buttons(vec![Button::new(
                "Get Solara Launcher",
                SOLARA_WEBSITE_URL,
            )]);

        let mut client = self.client.write().await;
        let res = client.set_activity(activity.clone());

        if let Err(first_err) = res {
            if reconnect_if_fail {
                if client.reconnect().is_ok() {
                    self.connected.store(true, Ordering::Relaxed);
                    if let Err(retry_err) = client.set_activity(activity) {
                        self.connected.store(false, Ordering::Relaxed);
                        tracing::debug!(
                            "Discord Rich Presence retry failed: {retry_err}"
                        );
                    }
                } else {
                    self.connected.store(false, Ordering::Relaxed);
                    tracing::debug!(
                        "Discord Rich Presence reconnect failed after error: {first_err}"
                    );
                }
            } else {
                self.connected.store(false, Ordering::Relaxed);
                tracing::debug!(
                    "Discord Rich Presence set_activity failed: {first_err}"
                );
            }
        }

        Ok(())
    }

    /// Clear the activity entirely when Discord RPC is turned off.
    pub async fn clear_activity(
        &self,
        reconnect_if_fail: bool,
    ) -> crate::Result<()> {
        if !self.connected.load(Ordering::Relaxed) {
            return Ok(());
        }

        let mut client = self.client.write().await;
        let res = client.clear_activity();

        if res.is_err() {
            if reconnect_if_fail && client.reconnect().is_ok() {
                if client.clear_activity().is_err() {
                    self.connected.store(false, Ordering::Relaxed);
                }
            } else {
                self.connected.store(false, Ordering::Relaxed);
            }
        }
        Ok(())
    }

    /// Clear to default Solara presence (active running instance if any, else idle launcher).
    pub async fn clear_to_default(
        &self,
        reconnect_if_fail: bool,
        pool: &SqlitePool,
        process_manager: &ProcessManager,
    ) -> crate::Result<()> {
        let settings = crate::state::Settings::get(pool).await?;
        if !settings.discord_rpc {
            return self.clear_activity(false).await;
        }

        let running_instances = process_manager.get_all();
        if let Some(existing_child) = running_instances.first() {
            if let Ok(Some(inst)) =
                crate::state::get_instance(&existing_child.instance_id, pool)
                    .await
            {
                self.set_instance_activity(
                    &inst.instance.name,
                    inst.applied_content_set.loader.as_str(),
                    &inst.applied_content_set.game_version,
                    reconnect_if_fail,
                    pool,
                )
                .await?;
            } else {
                self.set_rich_activity(
                    &format!("Playing {}", existing_child.instance_name),
                    "Minecraft Java Edition",
                    true,
                    reconnect_if_fail,
                    pool,
                )
                .await?;
            }
        } else {
            self.set_rich_activity(
                "In Solara Launcher",
                "Ready to play",
                false,
                reconnect_if_fail,
                pool,
            )
            .await?;
        }
        Ok(())
    }

    /// Periodic background sync so Discord is automatically detected even if opened
    /// after Solara Launcher or restarted mid-session.
    pub async fn sync_background(
        &self,
        pool: &SqlitePool,
        process_manager: &ProcessManager,
    ) {
        let Ok(settings) = crate::state::Settings::get(pool).await else {
            return;
        };

        if !settings.discord_rpc {
            if self.is_connected() {
                let _ = self.clear_activity(false).await;
            }
            return;
        }

        if !self.is_connected() {
            let _ = self.clear_to_default(false, pool, process_manager).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solara_discord_app_id_and_assets_are_configured() {
        assert_eq!(SOLARA_DISCORD_APP_ID, "1574631435627659615");
        assert!(SOLARA_DISCORD_LOGO_URL.starts_with("https://app.solara-inc.in/"));
        assert_eq!(SOLARA_WEBSITE_URL, "https://app.solara-inc.in");
    }

    #[test]
    fn loader_names_format_cleanly_for_discord_presence() {
        assert_eq!(format_loader_name("fabric"), "Fabric");
        assert_eq!(format_loader_name("forge"), "Forge");
        assert_eq!(format_loader_name("neoforge"), "NeoForge");
        assert_eq!(format_loader_name("neo"), "NeoForge");
        assert_eq!(format_loader_name("quilt"), "Quilt");
        assert_eq!(format_loader_name("vanilla"), "Vanilla");
    }
}
