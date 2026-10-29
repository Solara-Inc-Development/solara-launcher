use crate::content_activity::Ledger;
use anyhow::{bail, Context, Result};
use serde_json::{json, Value};
use solara_local_server::{LocalServers, ServerConfig};
use std::path::PathBuf;
use theseus::prelude::*;
use tokio::sync::{Mutex, OnceCell};

static ENGINE: OnceCell<Engine> = OnceCell::const_new();
static CONTENT_ACTIVITY: Mutex<Ledger> = Mutex::const_new(Ledger { jobs: Vec::new() });
static CONTENT_EVENTS: OnceCell<()> = OnceCell::const_new();
async fn content_events() {
    CONTENT_EVENTS
        .get_or_init(|| async {
            let mut events = theseus::EventState::get()
                .solara_instance_events
                .subscribe();
            tokio::spawn(async move {
                loop {
                    match events.recv().await {
                        Ok(payload) => match payload.event {
                            theseus::InstancePayloadType::ContentInstallFinished {
                                project_ids,
                            } => CONTENT_ACTIVITY.lock().await.completed(
                                &payload.instance_id,
                                &project_ids,
                                true,
                            ),
                            theseus::InstancePayloadType::ContentInstallFailed {
                                project_ids,
                                ..
                            } => CONTENT_ACTIVITY.lock().await.completed(
                                &payload.instance_id,
                                &project_ids,
                                false,
                            ),
                            _ => {}
                        },
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                            CONTENT_ACTIVITY.lock().await.unconfirmed()
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                    }
                }
            });
        })
        .await;
}
pub struct Engine {
    pub local: LocalServers,
    pub data: PathBuf,
}
pub async fn get() -> Result<&'static Engine> {
    ENGINE.get().context("Launcher engine is not initialized")
}
pub async fn initialize() -> Result<()> {
    ENGINE
        .get_or_try_init(|| async {
            let data = dirs::data_local_dir()
                .context("Local app-data directory unavailable")?
                .join("in.solara.launcher");
            tokio::fs::create_dir_all(&data).await?;
            theseus::EventState::init().await?;
            content_events().await;
            theseus::State::init("in.solara.launcher".into()).await?;
            let mut settings = settings::get().await?;
            settings.telemetry = false;
            settings.personalized_ads = false;
            settings.discord_rpc = false;
            settings::set(settings).await?;
            Ok::<_, anyhow::Error>(Engine {
                local: LocalServers::new(data.join("local-servers")),
                data,
            })
        })
        .await?;
    Ok(())
}
pub async fn snapshot() -> Result<Value> {
    let e = get().await?;
    let running = process::get_all().await?;
    let items=instance::list().await?.into_iter().map(|m|json!({"id":m.instance.id,"name":m.instance.name,"gameVersion":m.applied_content_set.game_version,"loader":m.applied_content_set.loader,"state":m.instance.install_stage,"running":running.iter().any(|p|p.instance_id==m.instance.id),"memory":m.launch_overrides.memory.map(|m|m.maximum),"javaPath":m.launch_overrides.java_path})).collect::<Vec<_>>();
    let accounts = minecraft_auth::users()
        .await?
        .into_iter()
        .map(|c| json!({"id":c.offline_profile.id,"name":c.offline_profile.name,"active":c.active}))
        .collect::<Vec<_>>();
    let mut downloads = vec![];
    for job in install::list_jobs(true).await? {
        let mut actions = vec![];
        if job.can_cancel {
            actions.push("cancel")
        };
        if job.can_pause {
            actions.push(if job.paused { "resume" } else { "pause" })
        };
        if matches!(
            job.status,
            install::InstallJobStatus::Failed | install::InstallJobStatus::Interrupted
        ) {
            actions.push("retry")
        };
        let progress = job.progress.as_ref().and_then(|p| {
            if p.total > 0 {
                Some(p.current as f64 / p.total as f64)
            } else {
                None
            }
        });
        downloads.push(json!({"id":job.job_id,"name":job.instance_id.clone().unwrap_or_else(||"Minecraft installation".into()),"status":if job.paused{"paused"}else{job.status.as_str()},"total":0,"downloaded":0,"progress":progress,"detail":job.error.map(|e|e.message).unwrap_or_else(||format!("{:?}",job.phase)),"actions":actions}));
    }
    downloads.extend(
        CONTENT_ACTIVITY
            .lock()
            .await
            .jobs
            .iter()
            .map(|job| serde_json::to_value(job).unwrap_or_default()),
    );
    downloads.extend(
        e.local
            .downloads()
            .await
            .into_iter()
            .map(|x| serde_json::to_value(x).unwrap_or_default()),
    );
    Ok(
        json!({"instances":items,"minecraftAccounts":accounts,"localServers":e.local.list().await?,"downloads":downloads}),
    )
}
pub async fn versions() -> Result<Value> {
    let v = metadata::get_minecraft_versions().await?;
    Ok(serde_json::to_value(v.versions)?)
}
pub async fn create_instance(
    name: String,
    game_version: String,
    loader: String,
    loader_version: String,
) -> Result<Value> {
    if name.trim().is_empty() || name.chars().count() > 80 {
        bail!("Choose an instance name of 1–80 characters")
    };
    let loader: ModLoader =
        serde_json::from_value(json!(loader)).context("Unsupported mod loader")?;
    let v = metadata::get_minecraft_versions().await?;
    if !v.versions.iter().any(|v| v.id == game_version) {
        bail!("Choose a Minecraft version from the official manifest")
    };
    let job = install::create_instance(
        name,
        game_version,
        loader,
        if loader_version.is_empty() {
            None
        } else {
            Some(loader_version)
        },
        None,
        None,
        InstanceLink::Unmanaged,
    )
    .await?;
    Ok(json!({"jobId":job.job_id}))
}
pub async fn launch(id: String) -> Result<Value> {
    let p = instance::run(&id, instance::QuickPlayType::None).await?;
    Ok(json!({"processId":p.uuid}))
}
pub async fn stop(id: String) -> Result<()> {
    for p in process::get_by_instance_id(&id).await? {
        process::kill(p.uuid).await?
    }
    Ok(())
}
pub async fn edit(id: String, name: String, memory: u32, java_path: String) -> Result<()> {
    if name.trim().is_empty() || name.len() > 160 || !(512..=65536).contains(&memory) {
        bail!("Invalid instance name or memory setting")
    };
    let patch: EditInstance = serde_json::from_value(
        json!({"name":name,"launch_overrides":{"memory":{"maximum":memory},"java_path":if java_path.is_empty(){Value::Null}else{json!(java_path)}}}),
    )?;
    instance::edit(&id, patch).await?;
    Ok(())
}
pub async fn instance_folder(id: &str) -> Result<PathBuf> {
    let path = instance::get_full_path(id).await?;
    Ok(path)
}
pub async fn logs(id: &str) -> Result<String> {
    let c = theseus::logs::get_live_log_buffer(id).await?;
    let text = serde_json::to_value(c)?;
    Ok(text
        .as_str()
        .unwrap_or_default()
        .chars()
        .take(200_000)
        .collect())
}
pub async fn install_content(
    id: Option<String>,
    project: String,
    kind: String,
    version: Option<String>,
) -> Result<()> {
    get().await?;
    url_segment(&project)?;
    if kind == "modpack" {
        let version_id = version
            .as_deref()
            .context("Choose a modpack version before installing")?;
        url_segment(version_id)?;
        let response = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .redirect(reqwest::redirect::Policy::none())
            .build()?
            .get(format!("https://api.modrinth.com/v2/version/{version_id}"))
            .header(
                "User-Agent",
                "SolaraLauncher/0.0.1 (https://launcher.solara-inc.in)",
            )
            .send()
            .await?
            .error_for_status()?;
        let mut response = response;
        let mut bytes = Vec::new();
        while let Some(chunk) = response.chunk().await? {
            if bytes.len() + chunk.len() > 2_000_000 {
                bail!("Modpack metadata is too large")
            }
            bytes.extend_from_slice(&chunk);
        }
        let metadata: Value = serde_json::from_slice(&bytes)?;
        validate_modpack_version(&metadata, &project, version_id)?;
        install::create_modpack_instance(
            pack::install_from::CreatePackLocation::FromVersionId {
                project_id: project,
                version_id: version_id.into(),
                title: metadata["name"].as_str().unwrap_or("Modpack").into(),
                icon_url: None,
            },
            None,
        )
        .await?;
    } else {
        let id = id
            .as_deref()
            .filter(|s| !s.is_empty())
            .context("Choose an instance for this content")?;
        let content_type = match kind.as_str() {
            "mod" => "mod",
            "resourcepack" => "resource_pack",
            "shader" => "shader",
            _ => bail!("Unsupported content type"),
        };
        if let Some(ref version) = version {
            url_segment(version)?;
        }
        let request: instance::InstallProjectWithDependenciesRequest = serde_json::from_value(
            json!({"project_id":project,"version_id":version,"content_type":content_type,"selected":{}}),
        )?;
        let metadata = instance::get(id)
            .await?
            .context("Choose an existing instance for this content")?;
        let job = CONTENT_ACTIVITY
            .lock()
            .await
            .begin(id, &project, &metadata.instance.name)?;
        match instance::install_project_with_dependencies(id, request).await {
            Ok(_) => CONTENT_ACTIVITY.lock().await.running(&job),
            Err(error) => {
                CONTENT_ACTIVITY.lock().await.rejected(&job);
                return Err(error.into());
            }
        }
    }
    Ok(())
}
fn validate_modpack_version(metadata: &Value, project: &str, version: &str) -> Result<()> {
    if metadata["project_id"].as_str() != Some(project) || metadata["id"].as_str() != Some(version)
    {
        bail!("Selected modpack version does not belong to this project")
    }
    if !metadata["files"].as_array().is_some_and(|files| {
        files.iter().any(|f| {
            f["filename"]
                .as_str()
                .is_some_and(|name| name.ends_with(".mrpack"))
        })
    }) {
        bail!("Selected version has no supported modpack archive")
    }
    Ok(())
}

fn url_segment(s: &str) -> Result<&str> {
    if s.is_empty()
        || s.len() > 100
        || !s
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    {
        bail!("Invalid provider project identifier")
    };
    Ok(s)
}
pub async fn download_action(id: &str, action: &str) -> Result<()> {
    let id = uuid::Uuid::parse_str(id)?;
    match action {
        "pause" => {
            install::pause_job(id).await?;
        }
        "resume" => {
            install::resume_job(id).await?;
        }
        "cancel" => {
            install::cancel_job(id).await?;
        }
        "retry" => {
            install::retry_job(id).await?;
        }
        _ => bail!("Unsupported download action"),
    };
    Ok(())
}
#[derive(Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Preferences {
    pub memory: u32,
    pub java_path: String,
    pub discord: bool,
}
pub async fn preferences() -> Result<Preferences> {
    let path = get().await?.data.join("preferences.json");
    if path.exists() {
        return Ok(serde_json::from_slice(&tokio::fs::read(path).await?)?);
    };
    Ok(Preferences {
        memory: settings::get().await?.memory.maximum,
        java_path: "java".into(),
        discord: false,
    })
}
pub async fn save_preferences(memory: u32, java_path: String, discord: bool) -> Result<()> {
    if !(512..=65536).contains(&memory) || java_path.is_empty() {
        bail!("Choose valid memory and Java settings")
    };
    if discord
        && option_env!("SOLARA_DISCORD_CLIENT_ID")
            .unwrap_or_default()
            .is_empty()
    {
        bail!("Discord activity requires the Solara Discord application ID at build time")
    };
    let mut settings = settings::get().await?;
    settings.memory.maximum = memory;
    settings.telemetry = false;
    settings.personalized_ads = false;
    settings.discord_rpc = false;
    settings::set(settings).await?;
    let p = Preferences {
        memory,
        java_path,
        discord,
    };
    tokio::fs::write(
        get().await?.data.join("preferences.json"),
        serde_json::to_vec_pretty(&p)?,
    )
    .await?;
    Ok(())
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
    if let Some(e) = ENGINE.get() {
        e.local.stop_all().await?
    };
    Ok(())
}

#[cfg(test)]
mod content_tests {
    use super::*;
    #[test]
    fn chosen_pack_version_must_match_project_and_have_archive() {
        let valid = json!({"id":"version123","project_id":"project123","files":[{"filename":"world.mrpack"}]});
        assert!(validate_modpack_version(&valid, "project123", "version123").is_ok());
        assert!(validate_modpack_version(&valid, "other", "version123").is_err());
        assert!(validate_modpack_version(&valid, "project123", "other").is_err());
        assert!(validate_modpack_version(
            &json!({"id":"version123","project_id":"project123","files":[{"filename":"mod.jar"}]}),
            "project123",
            "version123"
        )
        .is_err());
    }
    #[test]
    fn provider_identifiers_cannot_be_paths_or_query_parameters() {
        assert!(url_segment("project-123_A").is_ok());
        for id in ["", "../other", "x?token=test", "x/y", "x#fragment"] {
            assert!(url_segment(id).is_err());
        }
    }
}

#[cfg(test)]
mod activity_event_tests {
    use super::*;
    #[tokio::test]
    async fn inherited_headless_events_update_local_activity_without_raw_error_text() {
        theseus::EventState::init().await.unwrap();
        content_events().await;
        for success in [true, false] {
            let project = if success { "qa-complete" } else { "qa-failed" };
            let id = CONTENT_ACTIVITY
                .lock()
                .await
                .begin("qa-instance", project, "QA fixture world")
                .unwrap();
            let event = if success {
                theseus::InstancePayloadType::ContentInstallFinished {
                    project_ids: vec![project.into()],
                }
            } else {
                theseus::InstancePayloadType::ContentInstallFailed {
                    project_ids: vec![project.into()],
                    message: "QA_RAW_ERROR_NOT_FOR_UI".into(),
                }
            };
            theseus::emit_instance("qa-instance", event).await.unwrap();
            tokio::time::timeout(std::time::Duration::from_secs(2), async {
                loop {
                    let done = CONTENT_ACTIVITY.lock().await.jobs.iter().any(|job| {
                        job.id == id
                            && job.status
                                == if success {
                                    crate::content_activity::Status::Completed
                                } else {
                                    crate::content_activity::Status::Failed
                                }
                    });
                    if done {
                        break;
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                }
            })
            .await
            .unwrap();
        }
        let json = serde_json::to_string(&CONTENT_ACTIVITY.lock().await.jobs).unwrap();
        assert!(!json.contains("QA_RAW_ERROR_NOT_FOR_UI"));
        assert!(!json.contains("qa-instance"));
    }
}
