use serde_json::Value;

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
}
