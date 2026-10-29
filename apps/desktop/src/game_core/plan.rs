//! Windows vanilla install plans from the official version schema.
use super::metadata::{relative_path, Artifact};
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct File {
    pub destination: String,
    pub artifact: Artifact,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Plan {
    pub version: String,
    pub kind: String,
    pub java_major: u32,
    pub main_class: String,
    pub files: Vec<File>,
    pub natives: Vec<Artifact>,
    pub asset_index: Artifact,
    pub asset_index_id: String,
    pub arguments: Value,
    pub legacy_arguments: Option<String>,
    pub logging_argument: Option<String>,
}

pub fn allowed(rules: Option<&Value>, os_version: Option<&str>) -> Result<bool> {
    let Some(rules) = rules else { return Ok(true) };
    let rules = rules
        .as_array()
        .context("Invalid Minecraft platform rules")?;
    let mut result = false;
    for rule in rules {
        let action = rule["action"]
            .as_str()
            .context("Missing Minecraft platform action")?;
        if !matches!(action, "allow" | "disallow") {
            bail!("Unsupported Minecraft platform action")
        }
        let mut applies = true;
        if let Some(os) = rule.get("os") {
            if let Some(name) = os.get("name") {
                applies &= name.as_str() == Some("windows");
            }
            if let Some(arch) = os.get("arch") {
                applies &= matches!(arch.as_str(), Some("x86_64" | "amd64"));
            }
            if let Some(version) = os.get("version") {
                if applies {
                    let actual = os_version
                        .context("This Minecraft version needs a Windows version check")?;
                    let pattern = version.as_str().context("Invalid Windows version rule")?;
                    if pattern.len() > 200 {
                        bail!("Windows version rule too large")
                    }
                    applies &= regex::Regex::new(pattern)?.is_match(actual);
                }
            }
        }
        if let Some(features) = rule.get("features") {
            for (_, expected) in features.as_object().context("Invalid Minecraft features")? {
                // No demo, custom-size or quick-play launch is silently enabled.
                applies &= expected.as_bool() == Some(false);
            }
        }
        if applies {
            result = action == "allow";
        }
    }
    Ok(result)
}

pub fn windows(version: &Value, os_version: Option<&str>) -> Result<Plan> {
    let id = version["id"]
        .as_str()
        .context("Minecraft version missing")?;
    super::metadata::version_id(id)?;
    let mut files = BTreeMap::<String, Artifact>::new();
    let client: Artifact = serde_json::from_value(version["downloads"]["client"].clone())?;
    client.validate()?;
    files.insert("client.jar".into(), client);
    let mut natives = Vec::new();
    let libraries = version["libraries"]
        .as_array()
        .context("Minecraft libraries missing")?;
    if libraries.len() > 2000 {
        bail!("Minecraft library limit exceeded")
    }
    for library in libraries {
        if !allowed(library.get("rules"), os_version)? {
            continue;
        }
        if let Some(raw) = library["downloads"].get("artifact") {
            let artifact: Artifact = serde_json::from_value(raw.clone())?;
            artifact.validate()?;
            let path = artifact
                .path
                .as_deref()
                .context("Minecraft library path missing")?;
            let target = format!("libraries/{path}");
            relative_path(&target)?;
            if let Some(prior) = files.insert(target, artifact.clone()) {
                if prior.sha1 != artifact.sha1 {
                    bail!("Conflicting Minecraft library path")
                }
            }
        }
        if let Some(classifier) = library["natives"]["windows"].as_str() {
            let classifier = classifier.replace("${arch}", "64");
            let native: Artifact =
                serde_json::from_value(library["downloads"]["classifiers"][&classifier].clone())?;
            native.validate()?;
            natives.push(native);
        }
    }
    let index: Artifact = serde_json::from_value(version["assetIndex"].clone())?;
    index.validate()?;
    let index_id = version["assetIndex"]["id"]
        .as_str()
        .context("Minecraft asset index missing")?;
    super::metadata::version_id(index_id)?;
    let logging = version["logging"]["client"].as_object();
    let mut logging_argument = None;
    if let Some(logging) = logging {
        let file: Artifact = serde_json::from_value(
            logging
                .get("file")
                .context("Minecraft log configuration missing")?
                .clone(),
        )?;
        file.validate()?;
        files.insert("logging.xml".into(), file);
        let arg = logging
            .get("argument")
            .and_then(Value::as_str)
            .context("Invalid Minecraft logging argument")?;
        if arg.len() > 1024 || !arg.starts_with("-Dlog4j.configurationFile=") {
            bail!("Unsupported Minecraft logging argument")
        }
        logging_argument = Some(arg.into());
    }
    let main_class = version["mainClass"]
        .as_str()
        .context("Minecraft main class missing")?;
    if main_class.is_empty()
        || main_class.len() > 200
        || !main_class
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._$".contains(&b))
    {
        bail!("Invalid Minecraft main class")
    }
    let java_major = version["javaVersion"]["majorVersion"].as_u64().unwrap_or(8);
    if !(8..=64).contains(&java_major) {
        bail!("Unsupported Java requirement")
    }
    Ok(Plan {
        version: id.into(),
        kind: version["type"]
            .as_str()
            .context("Minecraft version type missing")?
            .into(),
        java_major: java_major as u32,
        main_class: main_class.into(),
        files: files
            .into_iter()
            .map(|(destination, artifact)| File {
                destination,
                artifact,
            })
            .collect(),
        natives,
        asset_index: index,
        asset_index_id: index_id.into(),
        arguments: version["arguments"].clone(),
        legacy_arguments: version["minecraftArguments"].as_str().map(String::from),
        logging_argument,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn platform_and_feature_rules_do_not_enable_demo_or_other_operating_systems() {
        assert!(allowed(None, None).unwrap());
        assert!(!allowed(Some(&json!([{"action":"allow","os":{"name":"osx"}}])), None).unwrap());
        assert!(!allowed(
            Some(&json!([{"action":"allow","features":{"is_demo_user":true}}])),
            None
        )
        .unwrap());
        assert!(allowed(
            Some(&json!([{"action":"allow"},{"action":"disallow","os":{"name":"linux"}}])),
            None
        )
        .unwrap());
        assert!(!allowed(
            Some(&json!([{"action":"allow"},{"action":"disallow","os":{"name":"windows"}}])),
            None
        )
        .unwrap());
        assert!(allowed(
            Some(&json!([{"action":"allow","os":{"version":"^10\\."}}])),
            None
        )
        .is_err());
        assert!(allowed(
            Some(&json!([{"action":"allow","os":{"version":"^10\\."}}])),
            Some("10.0")
        )
        .unwrap());
    }
}
