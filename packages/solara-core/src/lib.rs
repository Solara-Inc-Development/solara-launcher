//! Independent Solara Launcher core crate.
//!
//! Implements Solara's own Minecraft Java launcher primitives directly from
//! official Mojang Piston, Adoptium Temurin, Fabric Meta, and Quilt Meta
//! specifications without depending on inherited launcher engines.

pub mod hash;
pub mod jre;
pub mod launch;
pub mod loaders;
pub mod minecraft;
pub mod paths;

/// Default User-Agent sent to official Mojang, Adoptium, Fabric, and Quilt endpoints.
pub const SOLARA_CORE_USER_AGENT: &str = concat!(
    "Solara-Inc-Development/solara-core/",
    env!("CARGO_PKG_VERSION"),
    " (https://app.solara-inc.in)"
);

/// Builds a hardened HTTP client for official launcher metadata and verified artifact downloads.
pub fn build_http_client() -> anyhow::Result<reqwest::Client> {
    reqwest::Client::builder()
        .user_agent(SOLARA_CORE_USER_AGENT)
        .connect_timeout(std::time::Duration::from_secs(15))
        .timeout(std::time::Duration::from_secs(180))
        .build()
        .map_err(Into::into)
}
