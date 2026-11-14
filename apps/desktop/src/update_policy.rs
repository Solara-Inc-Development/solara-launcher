use serde_json::Value;

pub const GITHUB_LATEST_MANIFEST_URL: &str =
    "https://github.com/Solara-Inc-Development/solara-launcher/releases/latest/download/latest.json";

#[allow(dead_code)]
pub fn is_trusted_update_asset_url(url: &str) -> bool {
    let Ok(parsed) = reqwest::Url::parse(url) else {
        return false;
    };
    if parsed.scheme() != "https" || !parsed.username().is_empty() || parsed.password().is_some() {
        return false;
    }
    match parsed.host_str() {
        Some("github.com") => parsed
            .path()
            .starts_with("/Solara-Inc-Development/solara-launcher/releases/"),
        Some("objects.githubusercontent.com" | "release-assets.githubusercontent.com") => true,
        Some("app.solara-inc.in") => parsed.path().starts_with("/builds/"),
        _ => false,
    }
}

#[allow(dead_code)]
pub fn validate_latest_manifest(manifest: &Value) -> Result<(), &'static str> {
    let version = manifest["version"]
        .as_str()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .ok_or("missing version")?;
    let mut parts = version.split('.');
    let (Some(maj), Some(min), Some(patch), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return Err("version must be numeric semver");
    };
    if ![maj, min, patch]
        .into_iter()
        .all(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()))
    {
        return Err("version must be numeric semver");
    }

    let pub_date = manifest["pub_date"]
        .as_str()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or("missing pub_date")?;
    if chrono::DateTime::parse_from_rfc3339(pub_date).is_err() {
        return Err("pub_date must be valid RFC3339");
    }

    let win = manifest["platforms"]["windows-x86_64"]
        .as_object()
        .ok_or("missing windows-x86_64 platform")?;
    let signature = win
        .get("signature")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| s.len() > 50)
        .ok_or("missing or invalid signature")?;
    if signature.contains(char::is_whitespace) {
        return Err("signature must be base64 encoded");
    }
    let url = win
        .get("url")
        .and_then(Value::as_str)
        .map(str::trim)
        .ok_or("missing download url")?;
    if !is_trusted_update_asset_url(url) {
        return Err("untrusted download url");
    }
    Ok(())
}

/// Unknown state blocks replacement too: never interrupt local work on a guess.
#[allow(dead_code)]
pub fn busy(snapshot: &Value) -> bool {
    let Some(instances) = snapshot["instances"].as_array() else {
        return true;
    };
    let Some(servers) = snapshot["localServers"].as_array() else {
        return true;
    };
    let Some(downloads) = snapshot["downloads"].as_array() else {
        return true;
    };
    instances
        .iter()
        .any(|i| i["running"].as_bool() != Some(false))
        || servers
            .iter()
            .any(|s| !matches!(s["state"].as_str(), Some("stopped" | "setup incomplete")))
        || downloads.iter().any(|d| {
            !matches!(
                d["status"].as_str(),
                Some(
                    "completed" | "succeeded" | "failed" | "cancelled" | "canceled" | "interrupted"
                )
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn replacement_waits_for_games_servers_downloads_and_unknown_state() {
        let idle = json!({"instances":[{"running":false}],"localServers":[{"state":"stopped"}],"downloads":[{"status":"completed"}]});
        assert!(!busy(&idle));
        assert!(busy(&json!({})));
        for terminal in [
            "completed",
            "succeeded",
            "failed",
            "cancelled",
            "canceled",
            "interrupted",
        ] {
            let mut value = idle.clone();
            value["downloads"][0]["status"] = json!(terminal);
            assert!(!busy(&value));
        }
        for state in ["running", "queued", "paused", "unknown", "resolving"] {
            let mut value = idle.clone();
            value["downloads"][0]["status"] = json!(state);
            assert!(busy(&value));
        }
        let mut value = idle.clone();
        value["instances"][0]["running"] = json!(true);
        assert!(busy(&value));
        let mut value = idle.clone();
        value["localServers"][0]["state"] = json!("running");
        assert!(busy(&value));
        let mut value = idle;
        value["instances"][0]["running"] = Value::Null;
        assert!(busy(&value));
    }

    #[test]
    fn github_cdn_manifest_url_and_schema_validation() {
        assert_eq!(
            GITHUB_LATEST_MANIFEST_URL,
            "https://github.com/Solara-Inc-Development/solara-launcher/releases/latest/download/latest.json"
        );
        let parsed = reqwest::Url::parse(GITHUB_LATEST_MANIFEST_URL).expect("valid url");
        assert_eq!(parsed.scheme(), "https");
        assert_eq!(parsed.host_str(), Some("github.com"));

        let valid = json!({
            "version": "0.0.40",
            "notes": "GitHub Releases CDN updater",
            "pub_date": "2026-11-15T00:00:00.000Z",
            "platforms": {
                "windows-x86_64": {
                    "signature": "dW50cnVzdGVkIGNvbW1lbnQ6IHNpZ25hdHVyZSBmcm9tIHRhdXJpIHNlY3JldCBrZXkK",
                    "url": "https://github.com/Solara-Inc-Development/solara-launcher/releases/download/V-0.0.40/solaralauncher.exe"
                }
            }
        });
        assert!(validate_latest_manifest(&valid).is_ok());

        let mut bad_ver = valid.clone();
        bad_ver["version"] = json!("V-0.0.40");
        assert!(validate_latest_manifest(&bad_ver).is_err());

        let mut bad_url = valid.clone();
        bad_url["platforms"]["windows-x86_64"]["url"] =
            json!("http://github.com/Solara-Inc-Development/solara-launcher/releases/download/V-0.0.40/solaralauncher.exe");
        assert!(validate_latest_manifest(&bad_url).is_err());

        let mut evil_url = valid.clone();
        evil_url["platforms"]["windows-x86_64"]["url"] =
            json!("https://evil.example.com/solaralauncher.exe");
        assert!(validate_latest_manifest(&evil_url).is_err());

        let mut missing_plat = valid;
        missing_plat["platforms"] = json!({});
        assert!(validate_latest_manifest(&missing_plat).is_err());
    }
}
