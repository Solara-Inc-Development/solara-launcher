//! Solara-authored loader integration using the loaders' official metadata.
use super::{
    metadata::{self, Artifact},
    plan::{self, File, Plan},
};
use anyhow::{bail, Context, Result};
use serde::Serialize;
use serde_json::{json, Value};

#[derive(Serialize, Debug, Clone)]
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
fn version_sort_key(value: &str) -> (u32, u32, u32, u8, u32) {
    let (base, pre) = value.split_once('-').unwrap_or((value, ""));
    let base = base.split('+').next().unwrap_or(base);
    let mut parts = base.split('.').map(|s| s.parse::<u32>().unwrap_or(0));
    let major = parts.next().unwrap_or(0);
    let minor = parts.next().unwrap_or(0);
    let patch = parts.next().unwrap_or(0);
    let pre_num = pre
        .split(|c: char| !c.is_ascii_digit())
        .find_map(|s| (!s.is_empty()).then(|| s.parse::<u32>().unwrap_or(0)))
        .unwrap_or(0);
    (major, minor, patch, u8::from(pre.is_empty()), pre_num)
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
pub async fn quilt_versions(game: &str) -> Result<Vec<LoaderVersion>> {
    metadata::version_id(game)?;
    let mut url = reqwest::Url::parse("https://meta.quiltmc.org/v3/versions/loader/")?;
    url.path_segments_mut()
        .map_err(|_| anyhow::anyhow!("Invalid Quilt metadata address"))?
        .pop_if_empty()
        .push(game);
    let raw: Value = serde_json::from_slice(&metadata::fetch_metadata(url.as_str()).await?)?;
    let rows = raw.as_array().context("Invalid Quilt loader list")?;
    if rows.len() > 2000 {
        bail!("Quilt loader list exceeds limit")
    }
    let mut list: Vec<LoaderVersion> = rows
        .iter()
        .map(|r| {
            let id = r["loader"]["version"]
                .as_str()
                .context("Invalid Quilt version")?;
            loader_id(id)?;
            Ok(LoaderVersion {
                id: id.into(),
                stable: !id.contains('-'),
            })
        })
        .collect::<Result<_>>()?;
    list.sort_by(|a, b| version_sort_key(&b.id).cmp(&version_sort_key(&a.id)));
    Ok(list)
}
pub async fn forge_versions(game: &str) -> Result<Vec<LoaderVersion>> {
    metadata::version_id(game)?;
    let url = "https://files.minecraftforge.net/net/minecraftforge/forge/promotions_slim.json";
    let raw: Value = serde_json::from_slice(&metadata::fetch_metadata(url).await?)?;
    let promos = raw["promos"]
        .as_object()
        .context("Invalid Forge promotions list")?;
    let rec_key = format!("{game}-recommended");
    let lat_key = format!("{game}-latest");
    let mut list = Vec::new();
    let mut seen = std::collections::HashSet::new();

    if let Some(v) = promos.get(&rec_key).and_then(Value::as_str) {
        if loader_id(v).is_ok() && seen.insert(v.to_string()) {
            list.push(LoaderVersion {
                id: v.into(),
                stable: true,
            });
        }
    }
    if let Some(v) = promos.get(&lat_key).and_then(Value::as_str) {
        if loader_id(v).is_ok() && seen.insert(v.to_string()) {
            let has_rec = !list.is_empty();
            list.push(LoaderVersion {
                id: v.into(),
                stable: !has_rec,
            });
        }
    }
    Ok(list)
}
fn neoforge_version_prefix(game: &str) -> Option<String> {
    let parts: Vec<&str> = game.split('.').collect();
    if parts.len() >= 2 && parts[0] == "1" {
        let minor = parts[1].parse::<u32>().ok()?;
        if minor >= 20 {
            let patch = if parts.len() >= 3 {
                parts[2].parse::<u32>().ok()?
            } else {
                0
            };
            return Some(format!("{minor}.{patch}."));
        }
    }
    None
}
pub async fn neoforge_versions(game: &str) -> Result<Vec<LoaderVersion>> {
    metadata::version_id(game)?;
    let prefix = match neoforge_version_prefix(game) {
        Some(p) => p,
        None => return Ok(Vec::new()),
    };
    let url = "https://maven.neoforged.net/releases/net/neoforged/neoforge/maven-metadata.xml";
    let bytes = metadata::fetch_metadata(url).await?;
    let text = std::str::from_utf8(&bytes)?;
    let mut versions = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("<version>") {
            if let Some(v) = rest.strip_suffix("</version>") {
                if v.starts_with(&prefix) && loader_id(v).is_ok() {
                    versions.push(v.to_string());
                }
            }
        }
    }
    versions.sort_by(|a, b| version_sort_key(b).cmp(&version_sort_key(a)));
    if versions.len() > 50 {
        versions.truncate(50);
    }
    let list = versions
        .into_iter()
        .map(|v| {
            let is_pre = v.contains('-') || v.contains("alpha") || v.contains("beta");
            LoaderVersion {
                id: v,
                stable: !is_pre,
            }
        })
        .collect();
    Ok(list)
}
pub async fn versions(loader: &str, game: &str) -> Result<Vec<LoaderVersion>> {
    match loader {
        "fabric" => fabric_versions(game).await,
        "quilt" => quilt_versions(game).await,
        "forge" => forge_versions(game).await,
        "neoforge" => neoforge_versions(game).await,
        "vanilla" => Ok(vec![]),
        _ => bail!("This loader is not available yet"),
    }
}
pub async fn fabric_version(game: &str, requested: &str) -> Result<String> {
    resolve_version("fabric", game, requested).await
}
pub async fn quilt_version(game: &str, requested: &str) -> Result<String> {
    resolve_version("quilt", game, requested).await
}
pub async fn resolve_version(loader: &str, game: &str, requested: &str) -> Result<String> {
    let label = match loader {
        "fabric" => "Fabric",
        "quilt" => "Quilt",
        "forge" => "Forge",
        "neoforge" => "NeoForge",
        _ => bail!("Unsupported loader"),
    };
    let list = versions(loader, game).await?;
    if !requested.is_empty() {
        return list
            .into_iter()
            .find(|v| v.id == requested)
            .map(|v| v.id)
            .with_context(|| {
                format!("Choose a {label} version available for this Minecraft version")
            });
    }
    list.into_iter()
        .find(|v| v.stable)
        .map(|v| v.id)
        .with_context(|| format!("No stable {label} loader supports this Minecraft version"))
}
fn allowed_maven_base(loader: &str, url: &str) -> bool {
    match loader {
        "fabric" => url == "https://maven.fabricmc.net/",
        "quilt" => matches!(
            url,
            "https://maven.fabricmc.net/" | "https://maven.quiltmc.org/repository/release/"
        ),
        "forge" => matches!(
            url,
            "https://maven.minecraftforge.net/" | "https://libraries.minecraft.net/"
        ),
        "neoforge" => matches!(
            url,
            "https://maven.neoforged.net/releases/"
                | "https://maven.neoforged.net/"
                | "https://libraries.minecraft.net/"
        ),
        _ => false,
    }
}
fn is_allowed_artifact_url(loader: &str, url: &str) -> bool {
    match loader {
        "fabric" => url.starts_with("https://maven.fabricmc.net/"),
        "quilt" => {
            url.starts_with("https://maven.fabricmc.net/")
                || url.starts_with("https://maven.quiltmc.org/repository/release/")
        }
        "forge" => {
            url.starts_with("https://maven.minecraftforge.net/")
                || url.starts_with("https://libraries.minecraft.net/")
        }
        "neoforge" => {
            url.starts_with("https://maven.neoforged.net/")
                || url.starts_with("https://libraries.minecraft.net/")
        }
        _ => false,
    }
}
pub async fn fabric_profile(game: &str, loader_version: &str) -> Result<Value> {
    loader_profile("fabric", game, loader_version).await
}
pub async fn quilt_profile(game: &str, loader_version: &str) -> Result<Value> {
    loader_profile("quilt", game, loader_version).await
}
pub async fn loader_profile(loader: &str, game: &str, loader_version: &str) -> Result<Value> {
    metadata::version_id(game)?;
    loader_id(loader_version)?;

    if loader == "forge" {
        let installer_url = format!(
            "https://maven.minecraftforge.net/net/minecraftforge/forge/{game}-{loader_version}/forge-{game}-{loader_version}-installer.jar"
        );
        let bytes = metadata::fetch_metadata(&installer_url).await?;
        let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes))
            .context("Forge installer is not a valid zip")?;
        let file = zip
            .by_name("version.json")
            .context("Forge installer missing version.json")?;
        let mut profile: Value = serde_json::from_reader(file)?;
        normalize_profile_libraries(&mut profile, "forge", "https://maven.minecraftforge.net/")?;
        return Ok(profile);
    }
    if loader == "neoforge" {
        let installer_url = format!(
            "https://maven.neoforged.net/releases/net/neoforged/neoforge/{loader_version}/neoforge-{loader_version}-installer.jar"
        );
        let bytes = metadata::fetch_metadata(&installer_url).await?;
        let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes))
            .context("NeoForge installer is not a valid zip")?;
        let file = zip
            .by_name("version.json")
            .context("NeoForge installer missing version.json")?;
        let mut profile: Value = serde_json::from_reader(file)?;
        normalize_profile_libraries(
            &mut profile,
            "neoforge",
            "https://maven.neoforged.net/releases/",
        )?;
        return Ok(profile);
    }

    let (endpoint, default_maven, label) = match loader {
        "fabric" => (
            "https://meta.fabricmc.net/v2/versions/loader/",
            "https://maven.fabricmc.net/",
            "Fabric",
        ),
        "quilt" => (
            "https://meta.quiltmc.org/v3/versions/loader/",
            "https://maven.quiltmc.org/repository/release/",
            "Quilt",
        ),
        _ => bail!("Unsupported loader"),
    };
    let mut url = reqwest::Url::parse(endpoint)?;
    url.path_segments_mut()
        .map_err(|_| anyhow::anyhow!("Invalid {label} metadata address"))?
        .pop_if_empty()
        .extend([game, loader_version, "profile", "json"]);
    let mut profile: Value =
        serde_json::from_slice(&metadata::fetch_metadata(url.as_str()).await?)?;
    if profile["inheritsFrom"].as_str() != Some(game) {
        bail!("{label} profile does not match this Minecraft version")
    }
    let libraries = profile["libraries"]
        .as_array_mut()
        .with_context(|| format!("{label} libraries missing"))?;
    if libraries.len() > 300 {
        bail!("{label} library count exceeds limit")
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
                .with_context(|| format!("{label} library coordinate missing"))?,
        )?;
        let base = lib["url"].as_str().unwrap_or(default_maven);
        if !allowed_maven_base(loader, base) {
            bail!("Untrusted {label} library repository")
        }
        let artifact_url = format!("{base}{path}");
        if lib.get("sha1").is_none() {
            let mut response = http
                .get(format!("{artifact_url}.sha1"))
                .send()
                .await?
                .error_for_status()?;
            let mut bytes = Vec::new();
            while let Some(chunk) = response.chunk().await? {
                if bytes.len() + chunk.len() > 256 {
                    bail!("{label} checksum response exceeds limit")
                }
                bytes.extend_from_slice(&chunk);
            }
            let hash = std::str::from_utf8(&bytes)?.trim();
            if hash.len() != 40 || !hash.bytes().all(|b| b.is_ascii_hexdigit()) {
                bail!("Invalid official {label} checksum")
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
                .with_context(|| format!("Official {label} library size is unavailable"))?;
            lib["size"] = json!(size);
        }
    }
    Ok(profile)
}
fn normalize_profile_libraries(
    profile: &mut Value,
    loader: &str,
    default_maven: &str,
) -> Result<()> {
    let libraries = profile["libraries"]
        .as_array_mut()
        .context("Libraries array missing in profile")?;
    for lib in libraries {
        let (extracted_sha1, extracted_size, extracted_url) =
            if let Some(art) = lib.get("downloads").and_then(|d| d.get("artifact")) {
                let sha1 = art.get("sha1").and_then(Value::as_str).map(String::from);
                let size = art.get("size").and_then(Value::as_u64);
                let url = art.get("url").and_then(Value::as_str).map(String::from);
                (sha1, size, url)
            } else {
                (None, None, None)
            };
        if lib.get("sha1").is_none() {
            if let Some(sha1) = extracted_sha1 {
                lib["sha1"] = json!(sha1);
            }
        }
        if lib.get("size").is_none() {
            if let Some(size) = extracted_size {
                lib["size"] = json!(size);
            }
        }
        if lib.get("url").is_none() {
            if let Some(url) = extracted_url {
                lib["url"] = json!(url);
            }
        }
        let url = lib.get("url").and_then(Value::as_str).unwrap_or(default_maven);
        if !is_allowed_artifact_url(loader, url) && !allowed_maven_base(loader, url) {
            bail!("Untrusted {loader} library repository: {url}");
        }
    }
    Ok(())
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
    apply_loader(plan, "fabric", profile, os_version)
}
pub fn apply_quilt(plan: &mut Plan, profile: &Value, os_version: Option<&str>) -> Result<()> {
    apply_loader(plan, "quilt", profile, os_version)
}
pub fn apply_forge(plan: &mut Plan, profile: &Value, os_version: Option<&str>) -> Result<()> {
    apply_loader(plan, "forge", profile, os_version)
}
pub fn apply_neoforge(plan: &mut Plan, profile: &Value, os_version: Option<&str>) -> Result<()> {
    apply_loader(plan, "neoforge", profile, os_version)
}
pub fn apply_loader(
    plan: &mut Plan,
    loader: &str,
    profile: &Value,
    os_version: Option<&str>,
) -> Result<()> {
    let (expected_prefixes, default_maven, label): (&[&str], &str, &str) = match loader {
        "fabric" => (&["net.fabricmc."], "https://maven.fabricmc.net/", "Fabric"),
        "quilt" => (
            &["org.quiltmc."],
            "https://maven.quiltmc.org/repository/release/",
            "Quilt",
        ),
        "forge" => (
            &["cpw.mods.", "net.minecraftforge."],
            "https://maven.minecraftforge.net/",
            "Forge",
        ),
        "neoforge" => (
            &["cpw.mods.", "net.neoforged."],
            "https://maven.neoforged.net/releases/",
            "NeoForge",
        ),
        _ => bail!("Unsupported loader"),
    };
    if profile["inheritsFrom"].as_str() != Some(&plan.version) {
        bail!("{label} profile does not match this Minecraft version")
    }
    let main = profile["mainClass"]
        .as_str()
        .with_context(|| format!("{label} main class missing"))?;
    if !expected_prefixes.iter().any(|prefix| main.starts_with(prefix))
        || main.len() > 200
        || !main
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._$".contains(&b))
    {
        bail!("Invalid {label} main class")
    }
    let libraries = profile["libraries"]
        .as_array()
        .with_context(|| format!("{label} libraries missing"))?;
    if libraries.len() > 300 {
        bail!("{label} library count exceeds limit")
    }
    let mut added = Vec::new();
    for lib in libraries {
        if !plan::allowed(lib.get("rules"), os_version)? {
            continue;
        }
        let (relative, artifact_url, sha1, size) = if let Some(art) =
            lib.get("downloads").and_then(|d| d.get("artifact"))
        {
            let path = art["path"].as_str().context("Library path missing")?;
            metadata::relative_path(path)?;
            let url = art["url"].as_str().context("Library url missing")?;
            let sha1 = art["sha1"]
                .as_str()
                .context("Library sha1 missing")?
                .to_string();
            let size = art["size"].as_u64().context("Library size missing")?;
            (path.to_string(), url.to_string(), sha1, size)
        } else {
            let rel = maven_path(
                lib["name"]
                    .as_str()
                    .with_context(|| format!("{label} library coordinate missing"))?,
            )?;
            let base = lib["url"].as_str().unwrap_or(default_maven);
            let sha1 = lib["sha1"]
                .as_str()
                .with_context(|| format!("{label} library checksum missing"))?
                .to_string();
            let size = lib["size"]
                .as_u64()
                .with_context(|| format!("{label} library size missing"))?;
            (rel.clone(), format!("{base}{rel}"), sha1, size)
        };
        if !is_allowed_artifact_url(loader, &artifact_url) && !allowed_maven_base(loader, &artifact_url) {
            bail!("Untrusted {label} library repository")
        }
        let artifact = Artifact {
            url: artifact_url,
            sha1,
            size,
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
                bail!("{label} argument count exceeds limit")
            }
            plan.arguments[kind]
                .as_array_mut()
                .context("Invalid Minecraft arguments")?
                .extend(extra.iter().cloned());
        }
    }
    if let Some(major) = profile["javaVersion"]["majorVersion"].as_u64() {
        if !(8..=64).contains(&major) {
            bail!("Unsupported {label} Java requirement")
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
    fn fabric_and_quilt_profiles_bind_game_and_reject_untrusted_artifacts() {
        let base_plan = || Plan {
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
        let mut p = base_plan();
        let profile = json!({"inheritsFrom":"1.21.1","mainClass":"net.fabricmc.loader.impl.launch.knot.KnotClient","libraries":[{"name":"net.fabricmc:fabric-loader:0.20.1","url":"https://evil.invalid/","sha1":"0".repeat(40),"size":1}],"arguments":{"game":[],"jvm":[]}});
        assert!(apply_fabric(&mut p, &profile, None).is_err());
        let mut valid = profile.clone();
        valid["libraries"][0]["url"] = json!("https://maven.fabricmc.net/");
        apply_fabric(&mut p, &valid, None).unwrap();
        assert_eq!(p.files.len(), 1);
        valid["inheritsFrom"] = json!("1.20.1");
        assert!(apply_fabric(&mut p, &valid, None).is_err());

        let mut q = base_plan();
        let quilt_profile = json!({"inheritsFrom":"1.21.1","mainClass":"org.quiltmc.loader.impl.launch.knot.KnotClient","libraries":[{"name":"org.quiltmc:quilt-loader:0.30.1","url":"https://maven.quiltmc.org/repository/release/","sha1":"0".repeat(40),"size":1}],"arguments":{"game":[],"jvm":[]}});
        apply_quilt(&mut q, &quilt_profile, None).unwrap();
        assert_eq!(q.files.len(), 1);
        assert_eq!(
            q.main_class,
            "org.quiltmc.loader.impl.launch.knot.KnotClient"
        );
        assert!(version_sort_key("0.30.1") > version_sort_key("0.30.1-beta.4"));
    }
    #[test]
    fn forge_and_neoforge_profiles_bind_game_and_verify_artifacts() {
        let base_plan = || Plan {
            version: "1.20.1".into(),
            kind: "release".into(),
            java_major: 17,
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
        let mut p = base_plan();
        let forge_profile = json!({
            "inheritsFrom": "1.20.1",
            "mainClass": "cpw.mods.bootstraplauncher.BootstrapLauncher",
            "libraries": [
                {
                    "name": "cpw.mods:securejarhandler:2.1.10",
                    "downloads": {
                        "artifact": {
                            "path": "cpw/mods/securejarhandler/2.1.10/securejarhandler-2.1.10.jar",
                            "url": "https://maven.minecraftforge.net/cpw/mods/securejarhandler/2.1.10/securejarhandler-2.1.10.jar",
                            "sha1": "51e6a22c6c716beb11e244bf5b8be480f51dd6b5",
                            "size": 88749
                        }
                    }
                }
            ],
            "arguments": {"game": [], "jvm": []}
        });
        apply_forge(&mut p, &forge_profile, None).unwrap();
        assert_eq!(p.files.len(), 1);
        assert_eq!(p.main_class, "cpw.mods.bootstraplauncher.BootstrapLauncher");

        let mut n = base_plan();
        n.version = "1.21.1".into();
        let neoforge_profile = json!({
            "inheritsFrom": "1.21.1",
            "mainClass": "net.neoforged.neoforge.Main",
            "libraries": [
                {
                    "name": "net.neoforged.fancymodloader:loader:4.0.45",
                    "downloads": {
                        "artifact": {
                            "path": "net/neoforged/fancymodloader/loader/4.0.45/loader-4.0.45.jar",
                            "url": "https://maven.neoforged.net/releases/net/neoforged/fancymodloader/loader/4.0.45/loader-4.0.45.jar",
                            "sha1": "4a0968c2ec6234d58e0a9ed97acafc08cc8f33b5",
                            "size": 505696
                        }
                    }
                }
            ],
            "arguments": {"game": [], "jvm": []}
        });
        apply_neoforge(&mut n, &neoforge_profile, None).unwrap();
        assert_eq!(n.files.len(), 1);
        assert_eq!(n.main_class, "net.neoforged.neoforge.Main");
    }

    #[test]
    fn parses_zip_installer_and_normalizes_libraries() {
        let profile_json = json!({
            "id": "1.20.1-forge-47.4.28",
            "inheritsFrom": "1.20.1",
            "mainClass": "cpw.mods.bootstraplauncher.BootstrapLauncher",
            "libraries": [
                {
                    "name": "cpw.mods:securejarhandler:2.1.10",
                    "downloads": {
                        "artifact": {
                            "path": "cpw/mods/securejarhandler/2.1.10/securejarhandler-2.1.10.jar",
                            "url": "https://maven.minecraftforge.net/cpw/mods/securejarhandler/2.1.10/securejarhandler-2.1.10.jar",
                            "sha1": "51e6a22c6c716beb11e244bf5b8be480f51dd6b5",
                            "size": 88749
                        }
                    }
                }
            ]
        });
        let mut cursor = std::io::Cursor::new(Vec::new());
        {
            let mut zip = zip::ZipWriter::new(&mut cursor);
            let options = zip::write::SimpleFileOptions::default();
            zip.start_file("version.json", options).unwrap();
            use std::io::Write;
            zip.write_all(profile_json.to_string().as_bytes()).unwrap();
            zip.finish().unwrap();
        }
        let bytes = cursor.into_inner();
        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();
        let file = archive.by_name("version.json").unwrap();
        let mut extracted: Value = serde_json::from_reader(file).unwrap();
        normalize_profile_libraries(&mut extracted, "forge", "https://maven.minecraftforge.net/").unwrap();
        assert_eq!(extracted["libraries"][0]["sha1"], "51e6a22c6c716beb11e244bf5b8be480f51dd6b5");
    }
}
