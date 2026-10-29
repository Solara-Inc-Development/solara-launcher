//! Read-only official metadata smoke check; no account/game download or launch.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use solara_launcher::game_core::{metadata, plan};
    let id = std::env::args().nth(1).unwrap_or_else(|| "1.21.1".into());
    metadata::version_id(&id)?;
    let versions = metadata::versions().await?;
    let entry = versions
        .iter()
        .find(|v| v.id == id)
        .ok_or_else(|| anyhow::anyhow!("Version not in official manifest"))?;
    let document = metadata::version(entry).await?;
    let plan = plan::windows(&document, Some("10.0"))?;
    println!(
        "{}",
        serde_json::json!({"version":plan.version,"javaMajor":plan.java_major,"fileCount":plan.files.len(),"nativeArchives":plan.natives.len(),"officialVersions":versions.len(),"metadataChecksumVerified":true,"gameDownloaded":false})
    );
    Ok(())
}
