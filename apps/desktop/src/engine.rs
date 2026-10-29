//! Solara's independently implemented native engine integration.
use crate::game_core::{
    auth::Accounts,
    download::{Cancellation, FileCache},
    instances::{write_json, Instances},
    jobs::Jobs,
    launch::Games,
    metadata,
};
use anyhow::{bail, Context, Result};
use serde_json::{json, Value};
use solara_local_server::{LocalServers, ServerConfig};
use std::path::PathBuf;
use tokio::sync::{OnceCell, Semaphore};

static ENGINE: OnceCell<Engine> = OnceCell::const_new();
pub struct Engine {
    pub local: LocalServers,
    pub data: PathBuf,
    pub instances: Instances,
    pub accounts: Accounts,
    pub games: Games,
    pub jobs: Jobs,
    cache: FileCache,
    slots: Semaphore,
}
pub async fn get() -> Result<&'static Engine> {
    ENGINE.get().context("Launcher engine is not initialized")
}
pub async fn initialize() -> Result<()> {
    ENGINE
        .get_or_try_init(|| async {
            let data = dirs::data_local_dir()
                .context("Local app-data folder unavailable")?
                .join("in.solara.launcher");
            tokio::fs::create_dir_all(&data).await?;
            let instances = Instances::new(data.join("custom-instances"));
            // A crashed/closed app cannot leave an instance permanently installing.
            for record in instances.list().await? {
                if record.state == "installing" {
                    instances
                        .transition(&record.id, &["installing"], "failed")
                        .await?;
                }
            }
            Ok::<_, anyhow::Error>(Engine {
                local: LocalServers::new(data.join("local-servers")),
                instances,
                accounts: Accounts::new(data.join("custom-minecraft-accounts")),
                games: Games::default(),
                jobs: Jobs::default(),
                cache: FileCache::new(data.join("verified-game-files"))?,
                slots: Semaphore::new(2),
                data,
            })
        })
        .await?;
    Ok(())
}
pub async fn snapshot() -> Result<Value> {
    let engine = get().await?;
    let running = engine.games.running().await?;
    let instances = engine
        .instances
        .list()
        .await?
        .into_iter()
        .map(|i| {
            let mut value = serde_json::to_value(&i).unwrap_or_default();
            value["running"] = json!(running.contains(&i.id));
            value
        })
        .collect::<Vec<_>>();
    let mut downloads = serde_json::to_value(engine.jobs.list()?)?
        .as_array()
        .cloned()
        .unwrap_or_default();
    downloads.extend(
        engine
            .local
            .downloads()
            .await
            .into_iter()
            .map(|j| serde_json::to_value(j).unwrap_or_default()),
    );
    Ok(
        json!({"instances":instances,"minecraftAccounts":engine.accounts.list().await?,"localServers":engine.local.list().await?,"downloads":downloads}),
    )
}
pub async fn versions() -> Result<Value> {
    Ok(serde_json::to_value(metadata::versions().await?)?)
}
pub fn capabilities() -> Value {
    json!({"loaders":["vanilla"],"contentInstall":false,"preview":true})
}

async fn windows_version() -> Result<Option<String>> {
    #[cfg(windows)]
    {
        let mut command = tokio::process::Command::new("cmd.exe");
        command.args(["/d", "/c", "ver"]).creation_flags(0x08000000);
        let output = tokio::time::timeout(std::time::Duration::from_secs(5), command.output())
            .await
            .context("Windows version check timed out")??;
        if !output.status.success() {
            bail!("Windows version could not be checked")
        }
        let text = String::from_utf8_lossy(&output.stdout);
        let pattern = regex::Regex::new(r"\d+\.\d+\.\d+")?;
        return Ok(Some(
            pattern
                .find(&text)
                .context("Windows version was not recognized")?
                .as_str()
                .into(),
        ));
    }
    #[cfg(not(windows))]
    {
        Ok(None)
    }
}
async fn queue_install(id: String, name: String) -> Result<String> {
    let engine = get().await?;
    let job = engine.jobs.begin(&id, &name)?;
    let job_id = job.id.clone();
    tokio::spawn(async move {
        let permit = tokio::select! {
            _=job.cancel.cancelled()=>{engine.jobs.finish(&job.id,false,true);return},
            permit=engine.slots.acquire()=>permit,
        };
        let Ok(_permit) = permit else {
            engine.jobs.finish(&job.id, false, false);
            return;
        };
        let result = async {
            job.cancel.check()?;
            let os_version = windows_version().await?;
            crate::game_core::install::vanilla(
                &engine.instances,
                &engine.cache,
                &id,
                &job.cancel,
                os_version.as_deref(),
                |p| {
                    engine
                        .jobs
                        .report(&job.id, p.phase, p.verified_bytes, p.total_bytes)
                },
            )
            .await
        }
        .await;
        engine
            .jobs
            .finish(&job.id, result.is_ok(), job.cancel.check().is_err());
    });
    Ok(job_id)
}
pub async fn create_instance(
    name: String,
    game_version: String,
    loader: String,
    loader_version: String,
) -> Result<Value> {
    if loader != "vanilla" || !loader_version.is_empty() {
        bail!("Only vanilla Minecraft is available in this testing build. More loaders are being added.")
    }
    let engine = get().await?;
    metadata::version_id(&game_version)?;
    if !metadata::versions()
        .await?
        .iter()
        .any(|v| v.id == game_version)
    {
        bail!("Choose a version from the official Minecraft list")
    }
    let prefs = preferences().await?;
    let record = engine
        .instances
        .create(
            name,
            game_version,
            loader,
            None,
            prefs.memory,
            prefs.java_path,
        )
        .await?;
    let job = queue_install(record.id.clone(), record.name).await?;
    Ok(json!({"jobId":job,"instanceId":record.id}))
}
pub async fn retry_instance(id: String) -> Result<String> {
    let engine = get().await?;
    let record = engine.instances.get(&id).await?;
    if !matches!(record.state.as_str(), "created" | "failed" | "cancelled") {
        bail!("This instance is not waiting for an installation retry")
    }
    queue_install(record.id, record.name).await
}
pub async fn launch(id: String) -> Result<Value> {
    if !cfg!(windows) {
        bail!("Minecraft play is available on Windows in this first preview")
    }
    let engine = get().await?;
    if engine.instances.get(&id).await?.state != "installed" {
        bail!("Finish installing this instance before playing")
    }
    let session = engine.accounts.session().await?;
    let version = windows_version().await?;
    let pid = engine
        .games
        .start(&engine.instances, &id, session, version.as_deref())
        .await?;
    Ok(json!({"processId":pid}))
}
pub async fn stop(id: String) -> Result<()> {
    get().await?.games.stop(&id).await
}
pub async fn edit(id: String, name: String, memory: u32, java_path: String) -> Result<()> {
    let engine = get().await?;
    if engine.games.running().await?.contains(&id) {
        bail!("Close this game before changing its settings")
    }
    let defaults = preferences().await?;
    engine
        .instances
        .change(&id, |i| {
            i.name = name.trim().into();
            i.memory = memory;
            i.java_path = if java_path.trim().is_empty() {
                defaults.java_path
            } else {
                java_path
            };
        })
        .await?;
    Ok(())
}
pub async fn instance_folder(id: &str) -> Result<PathBuf> {
    get().await?.instances.game_dir(id).await
}
pub async fn logs(id: &str) -> Result<String> {
    get().await?.games.logs(id).await
}
pub async fn minecraft_login(
    cancel: &Cancellation,
    prompt: impl FnMut(crate::game_core::auth::DevicePrompt),
) -> Result<()> {
    get().await?.accounts.login(cancel, prompt).await
}
pub async fn minecraft_logout(id: &str) -> Result<()> {
    get().await?.accounts.remove(id).await
}
pub async fn minecraft_select(id: &str) -> Result<()> {
    get().await?.accounts.select(id).await
}
pub async fn game_running() -> bool {
    match get().await {
        Ok(e) => e
            .games
            .running()
            .await
            .map(|v| !v.is_empty())
            .unwrap_or(false),
        Err(_) => false,
    }
}

pub async fn install_content(
    _: Option<String>,
    _: String,
    _: String,
    _: Option<String>,
) -> Result<()> {
    bail!("Content browsing is available. Installing mods and modpacks is coming in a later testing build.")
}
pub async fn download_action(id: &str, action: &str) -> Result<()> {
    let engine = get().await?;
    let job = engine.jobs.get(id)?;
    match action {
        "cancel" if matches!(job.status.as_str(), "queued" | "running") => job.cancel.cancel(),
        "retry" if matches!(job.status.as_str(), "failed" | "cancelled") => {
            let record = engine.instances.get(&job.instance).await?;
            queue_install(record.id, record.name).await?;
        }
        _ => bail!("This download action is not available"),
    };
    Ok(())
}
#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Preferences {
    pub memory: u32,
    pub java_path: String,
    pub discord: bool,
}
impl Default for Preferences {
    fn default() -> Self {
        Self {
            memory: 4096,
            java_path: "java".into(),
            discord: false,
        }
    }
}
pub async fn preferences() -> Result<Preferences> {
    let path = get().await?.data.join("preferences.json");
    match tokio::fs::metadata(&path).await {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Preferences::default()),
        Ok(meta) => {
            if !meta.is_file() || meta.len() > 65536 {
                bail!("Invalid local settings")
            };
            let value: Preferences = serde_json::from_slice(&tokio::fs::read(path).await?)?;
            validate_preferences(&value)?;
            Ok(value)
        }
        Err(e) => Err(e.into()),
    }
}
fn validate_preferences(value: &Preferences) -> Result<()> {
    if !(512..=65536).contains(&value.memory)
        || value.java_path.trim().is_empty()
        || value.java_path.contains('\0')
    {
        bail!("Choose valid memory and Java settings")
    };
    Ok(())
}
pub async fn save_preferences(memory: u32, java_path: String, discord: bool) -> Result<()> {
    let value = Preferences {
        memory,
        java_path,
        discord,
    };
    validate_preferences(&value)?;
    if discord
        && option_env!("SOLARA_DISCORD_CLIENT_ID")
            .unwrap_or_default()
            .is_empty()
    {
        bail!("Discord activity is not configured in this build")
    }
    write_json(&get().await?.data.join("preferences.json"), &value).await
}
pub async fn create_local(
    name: String,
    version: String,
    java_path: String,
    memory: u32,
    port: u16,
    eula: bool,
) -> Result<String> {
    get()
        .await?
        .local
        .create(ServerConfig {
            id: String::new(),
            name,
            version,
            java_path,
            memory,
            port,
            eula,
        })
        .await
}
pub async fn stop_all() -> Result<()> {
    if let Some(engine) = ENGINE.get() {
        engine.local.stop_all().await?
    };
    Ok(())
}
