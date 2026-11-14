pub mod engine;
#[cfg(feature = "desktop-shell")]
pub mod shell;

#[cfg(feature = "desktop-shell")]
mod app_updates;
mod update_policy;

pub mod game_core;
