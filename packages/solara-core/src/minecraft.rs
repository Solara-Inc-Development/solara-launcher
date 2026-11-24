//! Official Mojang Piston version manifest, rule evaluation, and verified artifact downloader.

use crate::hash::{verify_sha1_file, write_verified_sha1};
use crate::paths::{SolaraPaths, maven_coordinate_to_rel_path, safe_join_rel};
use anyhow::{Context, bail};
use futures::stream::{self, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;

pub const MOJANG_VERSION_MANIFEST_V2_URL: &str =
    "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";
pub const MOJANG_RESOURCES_BASE_URL: &str = "https://resources.download.minecraft.net";
pub const MOJANG_LIBRARIES_BASE_URL: &str = "https://libraries.minecraft.net/";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionManifestV2 {
    pub latest: LatestVersions,
    pub versions: Vec<VersionSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatestVersions {
    pub release: String,
    pub snapshot: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionSummary {
    pub id: String,
    #[serde(rename = "type")]
    pub version_type: String,
    pub url: String,
    pub time: String,
    #[serde(rename = "releaseTime")]
    pub release_time: String,
    pub sha1: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionDetails {
    pub id: String,
    #[serde(rename = "inheritsFrom")]
    pub inherits_from: Option<String>,
    #[serde(rename = "mainClass")]
    pub main_class: String,
    #[serde(rename = "type")]
    pub version_type: Option<String>,
    pub assets: Option<String>,
    #[serde(rename = "assetIndex")]
    pub asset_index: Option<AssetIndexRef>,
    #[serde(default)]
    pub downloads: VersionDownloads,
    #[serde(default)]
    pub libraries: Vec<LibraryEntry>,
    pub arguments: Option<LaunchArguments>,
    #[serde(rename = "minecraftArguments")]
    pub minecraft_arguments: Option<String>,
    #[serde(rename = "javaVersion")]
    pub java_version: Option<JavaVersionRequirement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JavaVersionRequirement {
    pub component: String,
    #[serde(rename = "majorVersion")]
    pub major_version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetIndexRef {
    pub id: String,
    pub sha1: String,
    pub size: u64,
    #[serde(rename = "totalSize")]
    pub total_size: Option<u64>,
    pub url: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VersionDownloads {
    pub client: Option<DownloadArtifact>,
    pub server: Option<DownloadArtifact>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadArtifact {
    pub path: Option<String>,
    pub sha1: Option<String>,
    pub size: Option<u64>,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryEntry {
    pub name: String,
    pub url: Option<String>,
    pub sha1: Option<String>,
    pub downloads: Option<LibraryDownloads>,
    pub natives: Option<HashMap<String, String>>,
    #[serde(default)]
    pub rules: Vec<Rule>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LibraryDownloads {
    pub artifact: Option<DownloadArtifact>,
    #[serde(default)]
    pub classifiers: HashMap<String, DownloadArtifact>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchArguments {
    #[serde(default)]
    pub game: Vec<ArgumentSpec>,
    #[serde(default)]
    pub jvm: Vec<ArgumentSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ArgumentSpec {
    Simple(String),
    Conditional {
        #[serde(default)]
        rules: Vec<Rule>,
        value: ArgumentValue,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ArgumentValue {
    Single(String),
    Multiple(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub action: String,
    pub os: Option<OsRule>,
    pub features: Option<HashMap<String, bool>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsRule {
    pub name: Option<String>,
    pub arch: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetIndexManifest {
    pub objects: BTreeMap<String, AssetObject>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetObject {
    pub hash: String,
    pub size: u64,
}

/// Target runtime environment used when evaluating Mojang library and argument rules.
#[derive(Debug, Clone)]
pub struct RuleContext {
    pub os_name: String,
    pub os_arch: String,
    pub features: HashMap<String, bool>,
}

impl RuleContext {
    pub fn current_host() -> Self {
        let os_name = if cfg!(target_os = "windows") {
            "windows"
        } else if cfg!(target_os = "macos") {
            "osx"
        } else {
            "linux"
        };
        let os_arch = match std::env::consts::ARCH {
            "x86" => "x86",
            "aarch64" => "arm64",
            _ => "x86_64",
        };
        Self {
            os_name: os_name.to_string(),
            os_arch: os_arch.to_string(),
            features: HashMap::new(),
        }
    }
}

/// Evaluates a Mojang rule list. Empty rules always allow.
pub fn rules_allow(rules: &[Rule], ctx: &RuleContext) -> bool {
    if rules.is_empty() {
        return true;
    }

    let mut allowed = false;
    for rule in rules {
        if rule_matches(rule, ctx) {
            allowed = rule.action == "allow";
        }
    }
    allowed
}

fn rule_matches(rule: &Rule, ctx: &RuleContext) -> bool {
    if let Some(os) = &rule.os {
        if let Some(expected_name) = &os.name
            && !expected_name.eq_ignore_ascii_case(&ctx.os_name)
        {
            return false;
        }
        if let Some(expected_arch) = &os.arch
            && !expected_arch.eq_ignore_ascii_case(&ctx.os_arch)
        {
            return false;
        }
    }

    if let Some(features) = &rule.features {
        for (feature, expected) in features {
            let actual = ctx.features.get(feature).copied().unwrap_or(false);
            if actual != *expected {
                return false;
            }
        }
    }

    true
}

/// Resolved downloadable library file (normal JAR or native classifier archive).
#[derive(Debug, Clone)]
pub struct ResolvedLibrary {
    pub coordinate: String,
    pub rel_path: String,
    pub url: String,
    pub sha1: Option<String>,
    pub is_native: bool,
}

/// Resolves all allowed library JARs and native classifier archives for the given host context.
pub fn resolve_libraries(details: &VersionDetails, ctx: &RuleContext) -> anyhow::Result<Vec<ResolvedLibrary>> {
    let mut out = Vec::new();
    for lib in &details.libraries {
        if !rules_allow(&lib.rules, ctx) {
            continue;
        }

        if let Some(downloads) = &lib.downloads {
            if let Some(art) = &downloads.artifact {
                let rel_path = match &art.path {
                    Some(p) if !p.is_empty() => p.clone(),
                    _ => maven_coordinate_to_rel_path(&lib.name)?,
                };
                out.push(ResolvedLibrary {
                    coordinate: lib.name.clone(),
                    rel_path,
                    url: art.url.clone(),
                    sha1: art.sha1.clone(),
                    is_native: false,
                });
            }

            if let Some(natives_map) = &lib.natives
                && let Some(raw_classifier) = natives_map.get(&ctx.os_name)
            {
                let arch_bits = if ctx.os_arch == "x86" { "32" } else { "64" };
                let classifier = raw_classifier.replace("${arch}", arch_bits);
                if let Some(native_art) = downloads.classifiers.get(&classifier) {
                    let rel_path = match &native_art.path {
                        Some(p) if !p.is_empty() => p.clone(),
                        _ => maven_coordinate_to_rel_path(&format!("{}:{classifier}", lib.name))?,
                    };
                    out.push(ResolvedLibrary {
                        coordinate: format!("{}:{classifier}", lib.name),
                        rel_path,
                        url: native_art.url.clone(),
                        sha1: native_art.sha1.clone(),
                        is_native: true,
                    });
                }
            }
        } else {
            // Fabric / Quilt style Maven library entry with `name` + base `url`.
            let rel_path = maven_coordinate_to_rel_path(&lib.name)?;
            let base = lib.url.as_deref().unwrap_or(MOJANG_LIBRARIES_BASE_URL);
            let url = format!("{}/{}", base.trim_end_matches('/'), rel_path);
            out.push(ResolvedLibrary {
                coordinate: lib.name.clone(),
                rel_path,
                url,
                sha1: lib.sha1.clone(),
                is_native: false,
            });
        }
    }
    Ok(out)
}

pub async fn fetch_version_manifest(client: &reqwest::Client) -> anyhow::Result<VersionManifestV2> {
    client
        .get(MOJANG_VERSION_MANIFEST_V2_URL)
        .send()
        .await
        .context("failed to request Mojang version_manifest_v2.json")?
        .error_for_status()?
        .json::<VersionManifestV2>()
        .await
        .context("failed to parse Mojang version_manifest_v2.json")
}

pub async fn fetch_version_details(
    client: &reqwest::Client,
    summary: &VersionSummary,
) -> anyhow::Result<VersionDetails> {
    let bytes = client
        .get(&summary.url)
        .send()
        .await
        .with_context(|| format!("failed to fetch Mojang version JSON for {}", summary.id))?
        .error_for_status()?
        .bytes()
        .await?;

    let actual_sha1 = crate::hash::sha1_hex(&bytes);
    if !actual_sha1.eq_ignore_ascii_case(&summary.sha1) {
        bail!(
            "Mojang version JSON SHA-1 mismatch for {}: expected {}, got {}",
            summary.id,
            summary.sha1,
            actual_sha1
        );
    }

    serde_json::from_slice::<VersionDetails>(&bytes)
        .with_context(|| format!("failed to parse Mojang version JSON for {}", summary.id))
}

/// Downloads a file if missing or hash-mismatched and verifies SHA-1 before atomic persistence.
pub async fn download_verified_sha1(
    client: &reqwest::Client,
    url: &str,
    dest: PathBuf,
    expected_sha1: Option<&str>,
) -> anyhow::Result<()> {
    if let Some(sha1) = expected_sha1
        && verify_sha1_file(&dest, sha1).await.unwrap_or(false)
    {
        return Ok(());
    }
    if expected_sha1.is_none() && tokio::fs::try_exists(&dest).await.unwrap_or(false) {
        return Ok(());
    }

    let bytes = client
        .get(url)
        .send()
        .await
        .with_context(|| format!("download failed for {url}"))?
        .error_for_status()
        .with_context(|| format!("non-success HTTP status for {url}"))?
        .bytes()
        .await?;

    write_verified_sha1(&dest, &bytes, expected_sha1).await
}

/// Ensures client JAR, all resolved libraries, and all Mojang assets exist and pass SHA-1 checks.
pub async fn sync_minecraft_files(
    client: &reqwest::Client,
    paths: &SolaraPaths,
    details: &VersionDetails,
    ctx: &RuleContext,
    max_concurrency: usize,
) -> anyhow::Result<Vec<ResolvedLibrary>> {
    let concurrency = max_concurrency.clamp(1, 32);

    if let Some(client_dl) = &details.downloads.client {
        let jar_path = paths.version_jar(&details.id);
        download_verified_sha1(client, &client_dl.url, jar_path, client_dl.sha1.as_deref()).await?;
    }

    let libraries = resolve_libraries(details, ctx)?;
    let lib_results: Vec<anyhow::Result<()>> = stream::iter(libraries.iter().cloned())
        .map(|lib| {
            let client = client.clone();
            let libraries_dir = paths.libraries_dir();
            async move {
                let dest = safe_join_rel(&libraries_dir, &lib.rel_path)?;
                download_verified_sha1(&client, &lib.url, dest, lib.sha1.as_deref()).await
            }
        })
        .buffer_unordered(concurrency)
        .collect()
        .await;
    for res in lib_results {
        res?;
    }

    if let Some(asset_index) = &details.asset_index {
        let index_path = paths
            .asset_indexes_dir()
            .join(format!("{}.json", asset_index.id));
        download_verified_sha1(
            client,
            &asset_index.url,
            index_path.clone(),
            Some(&asset_index.sha1),
        )
        .await?;

        let raw_index = tokio::fs::read(&index_path).await?;
        let manifest: AssetIndexManifest = serde_json::from_slice(&raw_index)?;
        let asset_results: Vec<anyhow::Result<()>> = stream::iter(manifest.objects.into_values())
            .map(|obj| {
                let client = client.clone();
                let paths = paths.clone();
                async move {
                    let dest = paths.asset_object_path(&obj.hash)?;
                    let url = format!(
                        "{}/{}/{}",
                        MOJANG_RESOURCES_BASE_URL,
                        &obj.hash[..2],
                        obj.hash
                    );
                    download_verified_sha1(&client, &url, dest, Some(&obj.hash)).await
                }
            })
            .buffer_unordered(concurrency)
            .collect()
            .await;
        for res in asset_results {
            res?;
        }
    }

    Ok(libraries)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evaluates_mojang_os_and_feature_rules() {
        let mut features = HashMap::new();
        features.insert("has_custom_resolution".to_string(), true);
        let win_ctx = RuleContext {
            os_name: "windows".to_string(),
            os_arch: "x86_64".to_string(),
            features,
        };

        let osx_only = vec![Rule {
            action: "allow".to_string(),
            os: Some(OsRule {
                name: Some("osx".to_string()),
                arch: None,
                version: None,
            }),
            features: None,
        }];
        assert!(!rules_allow(&osx_only, &win_ctx));

        let allow_except_linux = vec![
            Rule {
                action: "allow".to_string(),
                os: None,
                features: None,
            },
            Rule {
                action: "disallow".to_string(),
                os: Some(OsRule {
                    name: Some("linux".to_string()),
                    arch: None,
                    version: None,
                }),
                features: None,
            },
        ];
        assert!(rules_allow(&allow_except_linux, &win_ctx));

        let custom_res = vec![Rule {
            action: "allow".to_string(),
            os: None,
            features: Some(HashMap::from([(
                "has_custom_resolution".to_string(),
                true,
            )])),
        }];
        assert!(rules_allow(&custom_res, &win_ctx));
    }
}
