//! Independent Fabric Meta and Quilt Meta loader profile fetching and merging.

use crate::minecraft::{LaunchArguments, VersionDetails};
use anyhow::{Context, bail};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub const FABRIC_META_BASE_URL: &str = "https://meta.fabricmc.net/v2";
pub const QUILT_META_BASE_URL: &str = "https://meta.quiltmc.org/v3";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SolaraLoader {
    Vanilla,
    Fabric,
    Quilt,
}

impl SolaraLoader {
    pub fn profile_json_url(self, game_version: &str, loader_version: &str) -> anyhow::Result<String> {
        let game = game_version.trim();
        let loader = loader_version.trim();
        if game.is_empty() || loader.is_empty() {
            bail!("game_version and loader_version must not be empty");
        }
        match self {
            SolaraLoader::Vanilla => bail!("vanilla does not use an external loader overlay"),
            SolaraLoader::Fabric => Ok(format!(
                "{FABRIC_META_BASE_URL}/versions/loader/{game}/{loader}/profile/json"
            )),
            SolaraLoader::Quilt => Ok(format!(
                "{QUILT_META_BASE_URL}/versions/loader/{game}/{loader}/profile/json"
            )),
        }
    }
}

pub async fn fetch_loader_profile(
    client: &reqwest::Client,
    loader: SolaraLoader,
    game_version: &str,
    loader_version: &str,
) -> anyhow::Result<VersionDetails> {
    let url = loader.profile_json_url(game_version, loader_version)?;
    client
        .get(&url)
        .send()
        .await
        .with_context(|| format!("failed to fetch loader profile from {url}"))?
        .error_for_status()?
        .json::<VersionDetails>()
        .await
        .with_context(|| format!("failed to parse loader profile JSON from {url}"))
}

/// Merges a Fabric or Quilt loader overlay profile onto a base Mojang Vanilla `VersionDetails`.
///
/// Loader libraries take precedence over duplicate Maven `group:artifact` coordinates in vanilla
/// (matching official Fabric/Quilt launcher semantics), while preserving vanilla downloads and assets.
pub fn merge_loader_profile(vanilla: &VersionDetails, overlay: &VersionDetails) -> VersionDetails {
    let mut seen_artifacts = HashSet::new();
    let mut merged_libraries = Vec::with_capacity(overlay.libraries.len() + vanilla.libraries.len());

    for lib in &overlay.libraries {
        if let Some(key) = group_artifact_key(&lib.name) {
            seen_artifacts.insert(key);
        }
        merged_libraries.push(lib.clone());
    }

    for lib in &vanilla.libraries {
        if let Some(key) = group_artifact_key(&lib.name)
            && seen_artifacts.contains(&key)
        {
            continue;
        }
        merged_libraries.push(lib.clone());
    }

    let merged_arguments = match (&vanilla.arguments, &overlay.arguments) {
        (Some(v), Some(o)) => Some(LaunchArguments {
            game: [v.game.clone(), o.game.clone()].concat(),
            jvm: [v.jvm.clone(), o.jvm.clone()].concat(),
        }),
        (Some(v), None) => Some(v.clone()),
        (None, Some(o)) => Some(o.clone()),
        (None, None) => None,
    };

    VersionDetails {
        id: overlay.id.clone(),
        inherits_from: Some(vanilla.id.clone()),
        main_class: overlay.main_class.clone(),
        version_type: overlay
            .version_type
            .clone()
            .or_else(|| vanilla.version_type.clone()),
        assets: overlay.assets.clone().or_else(|| vanilla.assets.clone()),
        asset_index: overlay
            .asset_index
            .clone()
            .or_else(|| vanilla.asset_index.clone()),
        downloads: vanilla.downloads.clone(),
        libraries: merged_libraries,
        arguments: merged_arguments,
        minecraft_arguments: overlay
            .minecraft_arguments
            .clone()
            .or_else(|| vanilla.minecraft_arguments.clone()),
        java_version: overlay
            .java_version
            .clone()
            .or_else(|| vanilla.java_version.clone()),
    }
}

fn group_artifact_key(coordinate: &str) -> Option<String> {
    let mut parts = coordinate.split(':');
    let group = parts.next()?;
    let artifact = parts.next()?;
    Some(format!("{group}:{artifact}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::minecraft::{LibraryEntry, VersionDownloads};

    #[test]
    fn merges_fabric_overlay_with_library_dedup() {
        let vanilla = VersionDetails {
            id: "1.21.4".to_string(),
            inherits_from: None,
            main_class: "net.minecraft.client.main.Main".to_string(),
            version_type: Some("release".to_string()),
            assets: Some("19".to_string()),
            asset_index: None,
            downloads: VersionDownloads::default(),
            libraries: vec![
                LibraryEntry {
                    name: "org.ow2.asm:asm:9.6".to_string(),
                    url: None,
                    sha1: None,
                    downloads: None,
                    natives: None,
                    rules: vec![],
                },
                LibraryEntry {
                    name: "com.mojang:logging:1.2.7".to_string(),
                    url: None,
                    sha1: None,
                    downloads: None,
                    natives: None,
                    rules: vec![],
                },
            ],
            arguments: None,
            minecraft_arguments: None,
            java_version: None,
        };

        let fabric = VersionDetails {
            id: "fabric-loader-0.16.9-1.21.4".to_string(),
            inherits_from: Some("1.21.4".to_string()),
            main_class: "net.fabricmc.loader.impl.launch.knot.KnotClient".to_string(),
            version_type: Some("release".to_string()),
            assets: None,
            asset_index: None,
            downloads: VersionDownloads::default(),
            libraries: vec![
                LibraryEntry {
                    name: "org.ow2.asm:asm:9.7.1".to_string(),
                    url: Some("https://maven.fabricmc.net/".to_string()),
                    sha1: None,
                    downloads: None,
                    natives: None,
                    rules: vec![],
                },
                LibraryEntry {
                    name: "net.fabricmc:fabric-loader:0.16.9".to_string(),
                    url: Some("https://maven.fabricmc.net/".to_string()),
                    sha1: None,
                    downloads: None,
                    natives: None,
                    rules: vec![],
                },
            ],
            arguments: None,
            minecraft_arguments: None,
            java_version: None,
        };

        let merged = merge_loader_profile(&vanilla, &fabric);
        assert_eq!(
            merged.main_class,
            "net.fabricmc.loader.impl.launch.knot.KnotClient"
        );
        assert_eq!(merged.assets.as_deref(), Some("19"));
        let names: Vec<&str> = merged.libraries.iter().map(|l| l.name.as_str()).collect();
        assert_eq!(
            names,
            vec![
                "org.ow2.asm:asm:9.7.1",
                "net.fabricmc:fabric-loader:0.16.9",
                "com.mojang:logging:1.2.7"
            ]
        );
    }

    #[test]
    fn builds_official_fabric_and_quilt_urls() {
        assert_eq!(
            SolaraLoader::Fabric
                .profile_json_url("1.21.4", "0.16.9")
                .unwrap(),
            "https://meta.fabricmc.net/v2/versions/loader/1.21.4/0.16.9/profile/json"
        );
        assert_eq!(
            SolaraLoader::Quilt
                .profile_json_url("1.20.4", "0.26.4")
                .unwrap(),
            "https://meta.quiltmc.org/v3/versions/loader/1.20.4/0.26.4/profile/json"
        );
    }
}
