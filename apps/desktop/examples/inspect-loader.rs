//! Read-only official Fabric/Quilt and Modrinth content planning; never executes game code.
use solara_launcher::game_core::{
    content,
    download::{Cancellation, FileCache},
    instances::Instances,
    loaders, metadata, plan,
};
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<_> = std::env::args().collect();
    let game = args
        .get(1)
        .filter(|a| !a.starts_with("--"))
        .map(String::as_str)
        .unwrap_or("1.21.1");
    let loader_kind = args
        .windows(2)
        .find(|w| w[0] == "--loader")
        .map(|w| w[1].as_str())
        .unwrap_or("fabric");
    let entries = metadata::versions().await?;
    let entry = entries
        .iter()
        .find(|v| v.id == game)
        .ok_or_else(|| anyhow::anyhow!("Official Minecraft version missing"))?;
    let document = metadata::version(entry).await?;
    let mut plan = plan::windows(&document, Some("10.0.22631"))?;
    let loader = loaders::resolve_version(loader_kind, game, "").await?;
    let profile = loaders::loader_profile(loader_kind, game, &loader).await?;
    loaders::apply_loader(&mut plan, loader_kind, &profile, Some("10.0.22631"))?;
    let mut verified = 0;
    let all = args.iter().any(|a| a == "--verify-loader-libraries");
    if all || args.iter().any(|a| a == "--verify-one-library") {
        let files = plan.files.iter().filter(|f| {
            f.artifact.url.starts_with("https://maven.fabricmc.net/")
                || f.artifact
                    .url
                    .starts_with("https://maven.quiltmc.org/repository/release/")
        });
        let cache = FileCache::new(std::env::temp_dir().join("solara-loader-smoke-cache"))?;
        for file in files.take(if all { usize::MAX } else { 1 }) {
            cache
                .obtain(&file.artifact, &Cancellation::default(), |_, _| {})
                .await?;
            verified += 1;
        }
    }
    let mut modrinth_summary = serde_json::Value::Null;
    if args.iter().any(|a| a == "--verify-modrinth-smoke") {
        let smoke_root =
            std::env::temp_dir().join(format!("solara-modrinth-smoke-{}", uuid::Uuid::new_v4()));
        let store = Instances::new(smoke_root.join("inst"));
        let cache = FileCache::new(smoke_root.join("cache"))?;
        let inst = store
            .create(
                "Smoke".into(),
                game.into(),
                loader_kind.into(),
                Some(loader.clone()),
                4096,
                "auto".into(),
            )
            .await?;
        let (mod_label, mod_files) =
            content::plan_project_install(&inst, "modmenu", "mod", None).await?;
        let root_mod = mod_files
            .last()
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Mod plan produced no files"))?;
        content::install_files(
            &store,
            &cache,
            &inst.id,
            &[root_mod.clone()],
            &Cancellation::default(),
            |_| {},
        )
        .await?;
        let installed_size = tokio::fs::metadata(
            store
                .game_dir(&inst.id)
                .await?
                .join(&root_mod.destination),
        )
        .await?
        .len();
        let _ = tokio::fs::remove_dir_all(&smoke_root).await;
        let pack = content::prepare_modpack("1KVo5zza", "N276l2ON").await?;
        modrinth_summary = serde_json::json!({
            "modLabel": mod_label,
            "modPlannedFilesWithDeps": mod_files.len(),
            "modVerifiedInstalledFile": root_mod.destination,
            "modVerifiedInstalledBytes": installed_size,
            "modpackName": pack.name,
            "modpackGameVersion": pack.game_version,
            "modpackLoader": pack.loader,
            "modpackLoaderVersion": pack.loader_version,
            "modpackArchiveBytes": pack.mrpack_bytes.len(),
            "modpackPlannedFiles": pack.files.len()
        });
    }
    println!(
        "{}",
        serde_json::json!({"game":game,"loader":loader_kind,"loaderVersion":loader,"mainClass":plan.main_class,"plannedFiles":plan.files.len(),"officialLoaderLibrariesVerified":verified,"modrinthSmoke":modrinth_summary,"gameLaunched":false})
    );
    Ok(())
}
