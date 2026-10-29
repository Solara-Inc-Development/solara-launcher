//! Opt-in static download/extraction smoke; never executes the Windows runtime.
use solara_launcher::game_core::{download::Cancellation, java::Runtimes};
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<_> = std::env::args().collect();
    anyhow::ensure!(args.get(1).map(String::as_str)==Some("--allow-runtime-download"),"Pass --allow-runtime-download <major> <temporary-output-folder>. No Java executable will be run.");
    let major: u32 = args
        .get(2)
        .ok_or_else(|| anyhow::anyhow!("Java major required"))?
        .parse()?;
    let root = std::path::PathBuf::from(
        args.get(3)
            .ok_or_else(|| anyhow::anyhow!("Temporary output folder required"))?,
    );
    let runtimes = Runtimes::new(root);
    let mut previous = (String::new(), 0u64);
    let path = runtimes
        .ensure(major, &Cancellation::default(), |phase, done, total| {
            let bucket = if total > 0 {
                done.saturating_mul(4) / total
            } else {
                0
            };
            if previous.0 != phase || previous.1 != bucket {
                println!("{phase}: {done}/{total}");
                previous = (phase.into(), bucket)
            }
        })
        .await?;
    anyhow::ensure!(tokio::fs::metadata(&path).await?.is_file());
    println!(
        "{}",
        serde_json::json!({"major":major,"managedExecutablePresent":true,"runtimeExecuted":false})
    );
    Ok(())
}
