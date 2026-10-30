//! Solara's independently implemented native engine integration.
use crate::game_core::{
    auth::Accounts,
    download::{Cancellation, FileCache},
    files::Files,
    instances::{write_json, Instances},
    java::Runtimes,
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
    pub java: Runtimes,
    pub files: Files,
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
            cleanup_stale_files(&data).await;
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
                java: Runtimes::new(data.join("java-runtimes")),
                files: Files::default(),
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
pub async fn loader_versions(game: &str, loader: &str) -> Result<Value> {
    match loader {
        "vanilla" => Ok(json!([])),
        "fabric" | "quilt" | "forge" | "neoforge" => Ok(serde_json::to_value(
            crate::game_core::loaders::versions(loader, game).await?,
        )?),
        _ => bail!("This loader is not available yet"),
    }
}
pub fn capabilities() -> Value {
    json!({"loaders":["vanilla","fabric","quilt","forge","neoforge"],"contentInstall":true,"automaticJava":true,"instanceFiles":true,"modManager":true,"instanceBackups":true,"preview":true})
}

async fn windows_version() -> Result<Option<String>> {
    #[cfg(windows)]
    {
        let ver = windows_version::OsVersion::current();
        Ok(Some(format!("{}.{}.{}", ver.major, ver.minor, ver.build)))
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
            crate::game_core::install::instance(
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
            .await?;
            if cfg!(windows) {
                resolve_java(&id, &job.cancel, |phase, done, total| {
                    engine.jobs.report(&job.id, phase.into(), done, total)
                })
                .await?;
            }
            Ok::<_, anyhow::Error>(())
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
    if !matches!(
        loader.as_str(),
        "vanilla" | "fabric" | "quilt" | "forge" | "neoforge"
    ) || (loader == "vanilla" && !loader_version.is_empty())
    {
        bail!("Choose a supported loader. More loaders are being added.")
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
    let resolved = if matches!(loader.as_str(), "fabric" | "quilt" | "forge" | "neoforge") {
        Some(
            crate::game_core::loaders::resolve_version(&loader, &game_version, &loader_version)
                .await?,
        )
    } else {
        None
    };
    let prefs = preferences().await?;
    let record = engine
        .instances
        .create(
            name,
            game_version,
            loader,
            resolved,
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
async fn resolve_java(
    id: &str,
    cancel: &Cancellation,
    report: impl FnMut(&str, u64, u64) + Send,
) -> Result<String> {
    let engine = get().await?;
    let record = engine.instances.get(id).await?;
    if !matches!(record.java_path.as_str(), "auto" | "java") {
        return Ok(record.java_path);
    }
    let path = engine.instances.path(id)?.join("runtime/plan.json");
    let meta = tokio::fs::symlink_metadata(&path).await?;
    if !meta.is_file() || meta.file_type().is_symlink() || meta.len() > 4 * 1024 * 1024 {
        bail!("Invalid installed launch plan")
    }
    let plan: crate::game_core::plan::Plan = serde_json::from_slice(&tokio::fs::read(path).await?)?;
    if plan.version != record.game_version {
        bail!("Installed files do not match this instance")
    }
    // A compatible explicitly selected PATH Java can be kept. Automatic mode
    // installs the metadata-required major in the launcher's own local directory.
    if record.java_path == "java"
        && crate::game_core::launch::java_check("java", plan.java_major)
            .await
            .is_ok()
    {
        return Ok("java".into());
    }
    let java = engine.java.ensure(plan.java_major, cancel, report).await?;
    java.to_str()
        .map(String::from)
        .context("Java path is not valid Unicode")
}
async fn queue_java(id: String) -> Result<String> {
    let engine = get().await?;
    let record = engine.instances.get(&id).await?;
    let job = engine
        .jobs
        .begin_kind(&id, &format!("Java for {}", record.name), "java")?;
    let key = job.id.clone();
    tokio::spawn(async move {
        let result = resolve_java(&id, &job.cancel, |phase, done, total| {
            engine.jobs.report(&job.id, phase.into(), done, total)
        })
        .await;
        engine
            .jobs
            .finish(&job.id, result.is_ok(), job.cancel.check().is_err());
    });
    Ok(key)
}
pub async fn launch(id: String) -> Result<Value> {
    if !cfg!(windows) {
        bail!("Minecraft play is available on Windows in this first preview")
    }
    let engine = get().await?;
    if engine.instances.get(&id).await?.state != "installed" {
        bail!("Finish installing this instance before playing")
    }
    let record = engine.instances.get(&id).await?;
    let java_job = engine
        .jobs
        .begin_kind(&id, &format!("Java for {}", record.name), "java")?;
    let result = resolve_java(&id, &java_job.cancel, |phase, done, total| {
        engine.jobs.report(&java_job.id, phase.into(), done, total)
    })
    .await;
    engine.jobs.finish(
        &java_job.id,
        result.is_ok(),
        java_job.cancel.check().is_err(),
    );
    let java = result?;
    let session = engine.accounts.session().await?;
    let version = windows_version().await?;
    let pid = engine
        .games
        .start(&engine.instances, &id, session, version.as_deref(), &java)
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
pub async fn assert_writable(id: &str) -> Result<()> {
    let engine = get().await?;
    engine.instances.get(id).await?;
    if engine.games.running().await?.iter().any(|key| key == id) {
        bail!("Close this game before changing its files")
    }
    if engine
        .jobs
        .list()?
        .iter()
        .any(|j| j.instance == id && matches!(j.status.as_str(), "queued" | "running"))
    {
        bail!("Wait for this instance's current task to finish")
    }
    Ok(())
}
pub async fn list_files(id: &str, path: &str) -> Result<Value> {
    let e = get().await?;
    Ok(serde_json::to_value(
        e.files.list(&e.instances, id, path).await?,
    )?)
}
pub async fn read_text(id: &str, path: &str) -> Result<Value> {
    let e = get().await?;
    Ok(serde_json::to_value(
        e.files.read(&e.instances, id, path).await?,
    )?)
}
pub async fn read_image(id: &str, relative: &str) -> Result<String> {
    use base64::Engine as _;
    let engine = get().await?;
    let path = crate::game_core::files::resolve(&engine.instances, id, relative, false).await?;
    let meta = tokio::fs::metadata(&path).await?;
    if !meta.is_file() || meta.len() > 10 * 1024 * 1024 {
        bail!("Choose an image smaller than 10 MiB");
    }
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();
    let mime = match ext.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        _ => bail!("Unsupported image format"),
    };
    let bytes = tokio::fs::read(&path).await?;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
    Ok(format!("data:{mime};base64,{b64}"))
}
pub async fn list_trash(id: &str) -> Result<Value> {
    let e = get().await?;
    Ok(serde_json::to_value(
        e.files.trash_list(&e.instances, id).await?,
    )?)
}
pub async fn list_mods(id: &str) -> Result<Value> {
    let e = get().await?;
    Ok(serde_json::to_value(
        crate::game_core::mods::list(&e.instances, &e.files, id).await?,
    )?)
}
pub async fn toggle_mod(id: &str, path: &str, enabled: bool) -> Result<()> {
    assert_writable(id).await?;
    let e = get().await?;
    crate::game_core::mods::toggle(&e.instances, &e.files, id, path, enabled).await
}
pub async fn import_mod(
    id: &str,
    file_name: Option<&str>,
    bytes_base64: Option<&str>,
    source_path: Option<&str>,
) -> Result<Value> {
    assert_writable(id).await?;
    let e = get().await?;
    let imported = if let Some(path) = source_path.map(str::trim).filter(|s| !s.is_empty()) {
        crate::game_core::mods::import_path(&e.instances, id, std::path::Path::new(path)).await?
    } else {
        let name = file_name
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .context("Choose a .jar mod file to import")?;
        let encoded = bytes_base64
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .context("Selected .jar mod file is empty")?;
        use base64::Engine as _;
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(encoded)
            .context("Could not read selected .jar mod file")?;
        crate::game_core::mods::import_bytes(&e.instances, id, name, &bytes).await?
    };
    Ok(serde_json::to_value(imported)?)
}
pub async fn list_backups(id: &str) -> Result<Value> {
    let e = get().await?;
    Ok(serde_json::to_value(
        crate::game_core::backups::list(&e.instances, id).await?,
    )?)
}
pub async fn backup_action(
    id: &str,
    action: &str,
    backup_id: Option<&str>,
    name: Option<&str>,
) -> Result<Value> {
    assert_writable(id).await?;
    let e = get().await?;
    match action {
        "create" => Ok(serde_json::to_value(
            crate::game_core::backups::create(&e.instances, id, name).await?,
        )?),
        "restore" => {
            let bid = backup_id
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .context("Choose a backup to restore")?;
            Ok(serde_json::to_value(
                crate::game_core::backups::restore(&e.instances, id, bid).await?,
            )?)
        }
        "delete" => {
            let bid = backup_id
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .context("Choose a backup to delete")?;
            crate::game_core::backups::delete(&e.instances, id, bid).await?;
            Ok(json!({"ok": true}))
        }
        _ => bail!("This backup action is not available"),
    }
}
pub async fn file_action(
    id: &str,
    action: &str,
    path: &str,
    target: Option<&str>,
    content: Option<&str>,
    expected: Option<&str>,
) -> Result<Value> {
    assert_writable(id).await?;
    let e = get().await?;
    match action {
        "mkdir" => e.files.mkdir(&e.instances, id, path).await?,
        "rename" => {
            e.files
                .rename(
                    &e.instances,
                    id,
                    path,
                    target.context("Choose a new file name")?,
                )
                .await?
        }
        "trash" => return Ok(json!({"trashId":e.files.trash(&e.instances,id,path).await?})),
        "restore" => e.files.restore(&e.instances, id, path).await?,
        "save" => {
            return Ok(
                json!({"sha256":e.files.write(&e.instances,id,path,content.context("File text missing")?,expected).await?}),
            )
        }
        _ => bail!("This file action is not available"),
    };
    Ok(json!({"ok":true}))
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
    instance_id: Option<String>,
    project_id: String,
    content_type: String,
    version_id: Option<String>,
) -> Result<()> {
    let engine = get().await?;
    if content_type == "modpack" {
        if instance_id.is_some() {
            bail!("Modpacks create a new instance")
        }
        let version = version_id
            .as_deref()
            .filter(|v| !v.is_empty())
            .context("Choose a modpack version")?;
        let prepared = crate::game_core::content::prepare_modpack(&project_id, version).await?;
        let prefs = preferences().await?;
        let record = engine
            .instances
            .create(
                prepared.name,
                prepared.game_version,
                prepared.loader,
                prepared.loader_version,
                prefs.memory,
                prefs.java_path,
            )
            .await?;
        let dir = engine.instances.path(&record.id)?;
        let temp = dir.join(format!(".modpack-{}.tmp", uuid::Uuid::new_v4()));
        if let Err(e) = async {
            use tokio::io::AsyncWriteExt;
            let mut f = tokio::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temp)
                .await?;
            f.write_all(&prepared.mrpack_bytes).await?;
            f.sync_all().await?;
            drop(f);
            tokio::fs::rename(&temp, dir.join("modpack.mrpack")).await?;
            Ok::<_, anyhow::Error>(())
        }
        .await
        {
            let _ = tokio::fs::remove_file(&temp).await;
            let _ = tokio::fs::remove_dir_all(&dir).await;
            return Err(e);
        }
        queue_install(record.id, record.name).await?;
        return Ok(());
    }
    let id = instance_id
        .filter(|s| !s.is_empty())
        .context("Choose an instance")?;
    assert_writable(&id).await?;
    let instance = engine.instances.get(&id).await?;
    let (label, planned) = crate::game_core::content::plan_project_install(
        &instance,
        &project_id,
        &content_type,
        version_id.as_deref(),
    )
    .await?;
    queue_content_install(id, format!("{label} for {}", instance.name), planned).await?;
    Ok(())
}
async fn queue_content_install(
    id: String,
    job_name: String,
    planned: Vec<crate::game_core::content::VerifiedFile>,
) -> Result<String> {
    let engine = get().await?;
    let job = engine.jobs.begin_content(&id, &job_name, planned.clone())?;
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
        let result = crate::game_core::content::install_files(
            &engine.instances,
            &engine.cache,
            &id,
            &planned,
            &job.cancel,
            |p| {
                engine
                    .jobs
                    .report(&job.id, p.phase, p.verified_bytes, p.total_bytes)
            },
        )
        .await;
        engine
            .jobs
            .finish(&job.id, result.is_ok(), job.cancel.check().is_err());
    });
    Ok(job_id)
}
pub async fn download_action(id: &str, action: &str) -> Result<()> {
    let engine = get().await?;
    let job = engine.jobs.get(id)?;
    match action {
        "cancel" if matches!(job.status.as_str(), "queued" | "running") => job.cancel.cancel(),
        "retry" if matches!(job.status.as_str(), "failed" | "cancelled") => {
            let record = engine.instances.get(&job.instance).await?;
            match job.kind.as_str() {
                "java" => {
                    queue_java(record.id).await?;
                }
                "install" => {
                    queue_install(record.id, record.name).await?;
                }
                "content" => {
                    let files = job
                        .content_files
                        .context("This content download cannot be retried")?;
                    assert_writable(&record.id).await?;
                    queue_content_install(record.id, job.name, files).await?;
                }
                _ => bail!("This download action is not available"),
            }
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
            java_path: "auto".into(),
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
pub fn discord_client_id() -> Option<String> {
    if let Some(val) = option_env!("SOLARA_DISCORD_CLIENT_ID") {
        if !val.trim().is_empty() {
            return Some(val.trim().to_string());
        }
    }
    if let Ok(val) = std::env::var("SOLARA_DISCORD_CLIENT_ID") {
        if !val.trim().is_empty() {
            return Some(val.trim().to_string());
        }
    }
    // Default Discord application client ID for Solara Rich Presence
    Some("1557970379111276665".to_string())
}
pub async fn save_preferences(memory: u32, java_path: String, discord: bool) -> Result<()> {
    let value = Preferences {
        memory,
        java_path,
        discord,
    };
    validate_preferences(&value)?;
    if discord && discord_client_id().is_none() {
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

/// Clean up stale temporary files, download caches, and legacy remnants
/// while strictly preserving user data (instances, saves, configs, mods, accounts, and servers).
pub async fn cleanup_stale_files(data: &std::path::Path) {
    // 1. Stale temporary directories and legacy engine remnants in launcher root
    for dir_name in &["app-update", "temp", "tmp", "theseus", "legacy"] {
        let path = data.join(dir_name);
        if path.exists() {
            let _ = tokio::fs::remove_dir_all(&path).await;
        }
    }

    // 2. Game file verification temporary directory
    let verified_tmp = data.join("verified-game-files").join(".tmp");
    if verified_tmp.exists() {
        let _ = tokio::fs::remove_dir_all(&verified_tmp).await;
    }

    // 3. Stale temporary files in launcher root data folder
    if let Ok(mut rd) = tokio::fs::read_dir(data).await {
        while let Ok(Some(entry)) = rd.next_entry().await {
            let file_name = entry.file_name().to_string_lossy().to_string();
            let is_stale_temp = file_name.ends_with(".tmp")
                || file_name.ends_with(".pending")
                || (file_name.starts_with(".solara-") && file_name.contains(".tmp"))
                || (file_name.starts_with(".modpack-") && file_name.ends_with(".tmp"));
            if is_stale_temp {
                if let Ok(meta) = entry.metadata().await {
                    if meta.is_file() {
                        let _ = tokio::fs::remove_file(entry.path()).await;
                    }
                }
            }
        }
    }

    // 4. Stale staging directories and temporary files inside custom-instances
    // User saves (game/saves), configs, mods (*.jar), and identity remain strictly untouched.
    let instances_dir = data.join("custom-instances");
    if let Ok(mut rd) = tokio::fs::read_dir(&instances_dir).await {
        while let Ok(Some(entry)) = rd.next_entry().await {
            if let Ok(file_type) = entry.file_type().await {
                if file_type.is_dir() {
                    let inst_dir = entry.path();
                    // Interrupted backup/restore staging directories
                    let backups_dir = inst_dir.join("backups");
                    if let Ok(mut b_rd) = tokio::fs::read_dir(&backups_dir).await {
                        while let Ok(Some(b_entry)) = b_rd.next_entry().await {
                            let name = b_entry.file_name().to_string_lossy().to_string();
                            if name.starts_with(".create-stage-")
                                || name.starts_with(".restore-stage-")
                            {
                                let _ = tokio::fs::remove_dir_all(b_entry.path()).await;
                            } else if name.ends_with(".tmp") || name.starts_with(".solara-backup-")
                            {
                                let _ = tokio::fs::remove_file(b_entry.path()).await;
                            }
                        }
                    }
                    // Clean temporary files in instance root and game folders
                    for sub in &["", "game", "game/mods"] {
                        let target_sub = if sub.is_empty() {
                            inst_dir.clone()
                        } else {
                            inst_dir.join(sub)
                        };
                        if let Ok(mut sub_rd) = tokio::fs::read_dir(&target_sub).await {
                            while let Ok(Some(f_entry)) = sub_rd.next_entry().await {
                                let name = f_entry.file_name().to_string_lossy().to_string();
                                if name.ends_with(".tmp")
                                    || (name.starts_with(".solara-") && name.contains(".tmp"))
                                {
                                    if let Ok(m) = f_entry.metadata().await {
                                        if m.is_file() {
                                            let _ = tokio::fs::remove_file(f_entry.path()).await;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn cleanup_stale_files_removes_unwanted_and_preserves_user_data() -> Result<()> {
        let dir =
            std::env::temp_dir().join(format!("solara-cleanup-test-{}", uuid::Uuid::new_v4()));
        tokio::fs::create_dir_all(&dir).await?;
        let data = dir.as_path();

        // 1. Setup stale/unwanted files and directories
        let app_update = data.join("app-update");
        tokio::fs::create_dir_all(&app_update).await?;
        tokio::fs::write(app_update.join("update.zip"), b"old update bytes").await?;

        let theseus = data.join("theseus");
        tokio::fs::create_dir_all(&theseus).await?;
        tokio::fs::write(theseus.join("legacy.dat"), b"old engine").await?;

        let verified_tmp = data.join("verified-game-files").join(".tmp");
        tokio::fs::create_dir_all(&verified_tmp).await?;
        tokio::fs::write(verified_tmp.join("chunk.tmp"), b"temp asset").await?;

        let root_tmp = data.join("stale.tmp");
        tokio::fs::write(&root_tmp, b"temp").await?;
        let root_solara_tmp = data.join(".solara-content-123.tmp");
        tokio::fs::write(&root_solara_tmp, b"temp").await?;

        // 2. Setup custom-instances with both valid data and stale staging
        let inst_dir = data.join("custom-instances").join("test-instance");
        let saves_dir = inst_dir.join("game").join("saves").join("World1");
        tokio::fs::create_dir_all(&saves_dir).await?;
        let level_dat = saves_dir.join("level.dat");
        tokio::fs::write(&level_dat, b"player save world").await?;

        let mods_dir = inst_dir.join("game").join("mods");
        tokio::fs::create_dir_all(&mods_dir).await?;
        let sodium_jar = mods_dir.join("sodium-mc1.20.jar");
        tokio::fs::write(&sodium_jar, b"real mod jar").await?;
        let stale_mod_tmp = mods_dir.join(".solara-mod-temp.tmp");
        tokio::fs::write(&stale_mod_tmp, b"partial mod download").await?;

        let instance_json = inst_dir.join("instance.json");
        tokio::fs::write(&instance_json, b"{\"id\":\"test-instance\"}").await?;

        let backups_dir = inst_dir.join("backups");
        let stage_dir = backups_dir.join(".create-stage-999");
        tokio::fs::create_dir_all(&stage_dir).await?;
        tokio::fs::write(stage_dir.join("partial.zip"), b"partial backup").await?;

        let valid_backup = backups_dir.join("backup-1.zip");
        tokio::fs::write(&valid_backup, b"completed backup").await?;

        // 3. Setup accounts, local servers, and preferences (must be preserved)
        let accounts_dir = data.join("custom-minecraft-accounts");
        tokio::fs::create_dir_all(&accounts_dir).await?;
        let account_file = accounts_dir.join("accounts.json");
        tokio::fs::write(&account_file, b"{\"accounts\":[]}").await?;

        let servers_dir = data.join("local-servers").join("survival-server");
        tokio::fs::create_dir_all(&servers_dir).await?;
        let server_props = servers_dir.join("server.properties");
        tokio::fs::write(&server_props, b"motd=Solara Server").await?;

        let preferences = data.join("preferences.json");
        tokio::fs::write(&preferences, b"{\"theme\":\"dark\"}").await?;

        // Run cleanup
        cleanup_stale_files(data).await;

        // Verify unwanted/stale files were deleted
        assert!(!app_update.exists(), "app-update dir must be deleted");
        assert!(!theseus.exists(), "theseus dir must be deleted");
        assert!(
            !verified_tmp.exists(),
            "verified-game-files/.tmp must be deleted"
        );
        assert!(!root_tmp.exists(), "root .tmp file must be deleted");
        assert!(
            !root_solara_tmp.exists(),
            "root .solara-*.tmp must be deleted"
        );
        assert!(
            !stage_dir.exists(),
            "interrupted backup stage must be deleted"
        );
        assert!(
            !stale_mod_tmp.exists(),
            "stale mod download temp must be deleted"
        );

        // Verify all user data is strictly preserved
        assert!(level_dat.exists(), "saves/level.dat must be preserved");
        assert!(sodium_jar.exists(), "mods jar must be preserved");
        assert!(instance_json.exists(), "instance.json must be preserved");
        assert!(
            valid_backup.exists(),
            "completed backup zip must be preserved"
        );
        assert!(account_file.exists(), "account logins must be preserved");
        assert!(
            server_props.exists(),
            "server configurations must be preserved"
        );
        assert!(preferences.exists(), "preferences must be preserved");

        let _ = tokio::fs::remove_dir_all(&dir).await;
        Ok(())
    }
}
