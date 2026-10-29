//! Read-only official Fabric planning; optionally verify one library, never run it.
use solara_launcher::game_core::{
    download::{Cancellation, FileCache},
    loaders, metadata, plan,
};
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<_> = std::env::args().collect();
    let game = args.get(1).map(String::as_str).unwrap_or("1.21.1");
    let entries = metadata::versions().await?;
    let entry = entries
        .iter()
        .find(|v| v.id == game)
        .ok_or_else(|| anyhow::anyhow!("Official Minecraft version missing"))?;
    let document = metadata::version(entry).await?;
    let mut plan = plan::windows(&document, Some("10.0.22631"))?;
    let loader = loaders::fabric_version(game, "").await?;
    let profile = loaders::fabric_profile(game, &loader).await?;
    loaders::apply_fabric(&mut plan, &profile, Some("10.0.22631"))?;
    let mut verified = 0;
    let all = args.iter().any(|a| a == "--verify-loader-libraries");
    if all || args.iter().any(|a| a == "--verify-one-library") {
        let files = plan
            .files
            .iter()
            .filter(|f| f.artifact.url.starts_with("https://maven.fabricmc.net/"));
        let cache = FileCache::new(std::env::temp_dir().join("solara-fabric-smoke-cache"))?;
        for file in files.take(if all { usize::MAX } else { 1 }) {
            cache
                .obtain(&file.artifact, &Cancellation::default(), |_, _| {})
                .await?;
            verified += 1;
        }
    }
    println!(
        "{}",
        serde_json::json!({"game":game,"loader":"fabric","loaderVersion":loader,"mainClass":plan.main_class,"plannedFiles":plan.files.len(),"officialLoaderLibrariesVerified":verified,"gameLaunched":false})
    );
    Ok(())
}
