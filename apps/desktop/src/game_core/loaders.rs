//! Solara-authored loader integration using the loaders' official metadata.
use super::{
    metadata::{self, Artifact},
    plan::{self, File, Plan},
};
use anyhow::{bail, Context, Result};
use serde::Serialize;
use serde_json::{json, Value};

#[derive(Serialize)]
pub struct LoaderVersion {
    pub id: String,
    pub stable: bool,
}
pub fn loader_id(value: &str) -> Result<()> {
    if value.is_empty()
        || value.len() > 100
        || !value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-+".contains(&b))
        || matches!(value, "." | "..")
    {
        bail!("Invalid loader version")
    }
    Ok(())
}
pub async fn fabric_versions(game: &str) -> Result<Vec<LoaderVersion>> {
    metadata::version_id(game)?;
    let mut url = reqwest::Url::parse("https://meta.fabricmc.net/v2/versions/loader/")?;
    url.path_segments_mut()
        .map_err(|_| anyhow::anyhow!("Invalid Fabric metadata address"))?
        .pop_if_empty()
        .push(game);
    let raw: Value = serde_json::from_slice(&metadata::fetch_metadata(url.as_str()).await?)?;
    let rows = raw.as_array().context("Invalid Fabric loader list")?;
    if rows.len() > 2000 {
        bail!("Fabric loader list exceeds limit")
    }
    rows.iter()
        .map(|r| {
            let id = r["loader"]["version"]
                .as_str()
                .context("Invalid Fabric version")?;
            loader_id(id)?;
            Ok(LoaderVersion {
                id: id.into(),
                stable: r["loader"]["stable"].as_bool() == Some(true),
            })
        })
        .collect()
}
pub async fn fabric_version(game: &str, requested: &str) -> Result<String> {
    let list = fabric_versions(game).await?;
    if !requested.is_empty() {
        return list
            .into_iter()
            .find(|v| v.id == requested)
            .map(|v| v.id)
            .context("Choose a Fabric version available for this Minecraft version");
    }
    list.into_iter()
        .find(|v| v.stable)
        .map(|v| v.id)
        .context("No stable Fabric loader supports this Minecraft version")
}
pub async fn fabric_profile(game: &str, loader: &str) -> Result<Value> {
    metadata::version_id(game)?;
    loader_id(loader)?;
    let mut url = reqwest::Url::parse("https://meta.fabricmc.net/v2/versions/loader/")?;
    url.path_segments_mut()
        .map_err(|_| anyhow::anyhow!("Invalid Fabric metadata address"))?
        .pop_if_empty()
        .extend([game, loader, "profile", "json"]);
    let mut profile: Value =
        serde_json::from_slice(&metadata::fetch_metadata(url.as_str()).await?)?;
    if profile["inheritsFrom"].as_str() != Some(game) {
        bail!("Fabric profile does not match this Minecraft version")
    }
    let libraries = profile["libraries"]
        .as_array_mut()
        .context("Fabric libraries missing")?;
    if libraries.len() > 300 {
        bail!("Fabric library count exceeds limit")
    }
    let http = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(30))
        .user_agent("SolaraLauncher/independent-core")
        .build()?;
    for lib in libraries {
        let path = maven_path(
            lib["name"]
                .as_str()
                .context("Fabric library coordinate missing")?,
        )?;
        if lib["url"].as_str().unwrap_or("https://maven.fabricmc.net/")
            != "https://maven.fabricmc.net/"
        {
            bail!("Untrusted Fabric library repository")
        }
        let artifact_url = format!("https://maven.fabricmc.net/{path}");
        // Fabric's own loader/intermediary entries omit hashes and sizes.
        // Resolve their official Maven sidecar and HEAD; never invent integrity data.
        if lib.get("sha1").is_none() {
            let mut response = http
                .get(format!("{artifact_url}.sha1"))
                .send()
                .await?
                .error_for_status()?;
            let mut bytes = Vec::new();
            while let Some(chunk) = response.chunk().await? {
                if bytes.len() + chunk.len() > 256 {
                    bail!("Fabric checksum response exceeds limit")
                }
                bytes.extend_from_slice(&chunk);
            }
            let hash = std::str::from_utf8(&bytes)?.trim();
            if hash.len() != 40 || !hash.bytes().all(|b| b.is_ascii_hexdigit()) {
                bail!("Invalid official Fabric checksum")
            }
            lib["sha1"] = json!(hash.to_ascii_lowercase());
        }
        if lib.get("size").is_none() {
            let response = http.head(&artifact_url).send().await?.error_for_status()?;
            let size = response
                .headers()
                .get(reqwest::header::CONTENT_LENGTH)
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<u64>().ok())
                .filter(|n| *n > 0 && *n <= 1024 * 1024 * 1024)
                .context("Official Fabric library size is unavailable")?;
            lib["size"] = json!(size);
        }
    }
    Ok(profile)
}
pub fn maven_path(coordinate: &str) -> Result<String> {
    if coordinate.len() > 400 {
        bail!("Maven coordinate exceeds limit")
    }
    let pieces: Vec<_> = coordinate.split(':').collect();
    if !(3..=4).contains(&pieces.len())
        || pieces.iter().any(|p| {
            p.is_empty()
                || !p
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"._-+".contains(&b))
        })
    {
        bail!("Unsupported Maven library coordinate")
    }
    if pieces[0].split('.').any(|p| p.is_empty() || p == "..") {
        bail!("Invalid Maven group")
    }
    let classifier = pieces.get(3).map(|s| format!("-{s}")).unwrap_or_default();
    let path = format!(
        "{}/{}/{}/{}-{}{classifier}.jar",
        pieces[0].replace('.', "/"),
        pieces[1],
        pieces[2],
        pieces[1],
        pieces[2]
    );
    metadata::relative_path(&path)?;
    Ok(path)
}
fn library_key(path: &str) -> Option<String> {
    let path = path.strip_prefix("libraries/")?;
    let p: Vec<_> = path.split('/').collect();
    if p.len() < 4 {
        return None;
    }
    Some(p[..p.len() - 2].join("/"))
}
pub fn apply_fabric(plan: &mut Plan, profile: &Value, os_version: Option<&str>) -> Result<()> {
    if profile["inheritsFrom"].as_str() != Some(&plan.version) {
        bail!("Fabric profile does not match this Minecraft version")
    }
    let main = profile["mainClass"]
        .as_str()
        .context("Fabric main class missing")?;
    if !main.starts_with("net.fabricmc.")
        || main.len() > 200
        || !main
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._$".contains(&b))
    {
        bail!("Invalid Fabric main class")
    }
    let libraries = profile["libraries"]
        .as_array()
        .context("Fabric libraries missing")?;
    if libraries.len() > 300 {
        bail!("Fabric library count exceeds limit")
    }
    let mut added = Vec::new();
    for lib in libraries {
        if !plan::allowed(lib.get("rules"), os_version)? {
            continue;
        }
        let relative = maven_path(
            lib["name"]
                .as_str()
                .context("Fabric library coordinate missing")?,
        )?;
        let base = lib["url"].as_str().unwrap_or("https://maven.fabricmc.net/");
        if base != "https://maven.fabricmc.net/" {
            bail!("Untrusted Fabric library repository")
        }
        let artifact = Artifact {
            url: format!("{base}{relative}"),
            sha1: lib["sha1"]
                .as_str()
                .context("Fabric library checksum missing")?
                .into(),
            size: lib["size"]
                .as_u64()
                .context("Fabric library size missing")?,
            path: Some(relative.clone()),
        };
        artifact.validate()?;
        let destination = format!("libraries/{relative}");
        let key = library_key(&destination);
        plan.files.retain(|f| library_key(&f.destination) != key);
        added.push(File {
            destination,
            artifact,
        });
    }
    plan.files.extend(added);
    if plan.arguments.is_null() {
        let game: Vec<_> = plan
            .legacy_arguments
            .as_deref()
            .context("Minecraft launch arguments missing")?
            .split_whitespace()
            .map(String::from)
            .collect();
        plan.arguments = json!({"game":game,"jvm":["-Djava.library.path=${natives_directory}","-cp","${classpath}"]});
    }
    for kind in ["jvm", "game"] {
        if let Some(extra) = profile["arguments"][kind].as_array() {
            if extra.len() > 100 {
                bail!("Fabric argument count exceeds limit")
            }
            plan.arguments[kind]
                .as_array_mut()
                .context("Invalid Minecraft arguments")?
                .extend(extra.iter().cloned());
        }
    }
    if let Some(major) = profile["javaVersion"]["majorVersion"].as_u64() {
        if !(8..=64).contains(&major) {
            bail!("Unsupported Fabric Java requirement")
        }
        plan.java_major = plan.java_major.max(major as u32)
    }
    plan.main_class = main.into();
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn maven_paths_cannot_escape_library_directory() {
        loader_id("0.10.6+build.214").unwrap();
        for value in [
            "../loader",
            "loader?query",
            "loader#fragment",
            "loader version",
        ] {
            assert!(loader_id(value).is_err());
        }
        assert_eq!(
            maven_path("net.fabricmc:fabric-loader:0.20.1").unwrap(),
            "net/fabricmc/fabric-loader/0.20.1/fabric-loader-0.20.1.jar"
        );
        for value in [
            "evil:../../path:1",
            "net..evil:mod:1",
            "net.test:CON:1",
            "net.test:mod:1@exe",
        ] {
            assert!(maven_path(value).is_err(), "{value}")
        }
    }
    #[test]
    fn fabric_profiles_bind_game_and_reject_untrusted_artifacts() {
        let mut p = Plan {
            version: "1.21.1".into(),
            kind: "release".into(),
            java_major: 21,
            main_class: "net.minecraft.client.main.Main".into(),
            files: vec![],
            natives: vec![],
            asset_index: Artifact {
                url: "https://piston-meta.mojang.com/fixture".into(),
                sha1: "0".repeat(40),
                size: 1,
                path: None,
            },
            asset_index_id: "fixture".into(),
            arguments: json!({"game":[],"jvm":[]}),
            legacy_arguments: None,
            logging_argument: None,
        };
        let profile = json!({"inheritsFrom":"1.21.1","mainClass":"net.fabricmc.loader.impl.launch.knot.KnotClient","libraries":[{"name":"net.fabricmc:fabric-loader:0.20.1","url":"https://evil.invalid/","sha1":"0".repeat(40),"size":1}],"arguments":{"game":[],"jvm":[]}});
        assert!(apply_fabric(&mut p, &profile, None).is_err());
        let mut valid = profile.clone();
        valid["libraries"][0]["url"] = json!("https://maven.fabricmc.net/");
        apply_fabric(&mut p, &valid, None).unwrap();
        assert_eq!(p.files.len(), 1);
        valid["inheritsFrom"] = json!("1.20.1");
        assert!(apply_fabric(&mut p, &valid, None).is_err());
    }
}
