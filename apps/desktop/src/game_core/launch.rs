//! Independently built argument vectors and local game processes. No shell launch.
use super::{
    auth::Session,
    instances::Instances,
    plan::{self, Plan},
};
use anyhow::{bail, Context, Result};
use serde_json::Value;
use std::{
    collections::{BTreeMap, HashMap},
    ffi::OsString,
    path::Path,
    process::Stdio,
    sync::Arc,
};
use tokio::{
    io::{AsyncRead, AsyncReadExt},
    process::{Child, Command},
    sync::Mutex,
};

fn expand(value: &str, vars: &BTreeMap<&str, String>) -> Result<String> {
    if value.len() > 65536 || value.contains('\0') {
        bail!("Invalid Minecraft argument")
    }
    let mut out = String::new();
    let mut rest = value;
    while let Some(at) = rest.find("${") {
        out.push_str(&rest[..at]);
        let suffix = &rest[at + 2..];
        let end = suffix
            .find('}')
            .context("Invalid Minecraft argument placeholder")?;
        out.push_str(
            vars.get(&suffix[..end])
                .context("This Minecraft argument is not supported yet")?,
        );
        rest = &suffix[end + 1..];
    }
    out.push_str(rest);
    Ok(out)
}
fn arguments(
    raw: &Value,
    vars: &BTreeMap<&str, String>,
    os_version: Option<&str>,
) -> Result<Vec<OsString>> {
    let array = raw
        .as_array()
        .context("Minecraft launch arguments missing")?;
    if array.len() > 300 {
        bail!("Minecraft argument count exceeds limit")
    }
    let mut out = Vec::new();
    for arg in array {
        if let Some(text) = arg.as_str() {
            out.push(expand(text, vars)?.into());
            continue;
        }
        if !plan::allowed(arg.get("rules"), os_version)? {
            continue;
        }
        if let Some(text) = arg["value"].as_str() {
            out.push(expand(text, vars)?.into());
        } else {
            for text in arg["value"]
                .as_array()
                .context("Invalid Minecraft launch argument")?
            {
                out.push(
                    expand(
                        text.as_str().context("Invalid Minecraft launch argument")?,
                        vars,
                    )?
                    .into(),
                );
            }
        }
    }
    Ok(out)
}

pub fn build_args(
    plan: &Plan,
    runtime: &Path,
    game: &Path,
    session: &Session,
    memory: u32,
    os_version: Option<&str>,
) -> Result<Vec<OsString>> {
    if !(512..=65536).contains(&memory) {
        bail!("Invalid Minecraft memory setting")
    }
    let text = |p: &Path| {
        p.to_str()
            .map(String::from)
            .context("The game path is not valid Unicode")
    };
    let mut classpath = Vec::new();
    for file in &plan.files {
        if file.destination.starts_with("libraries/") && file.destination.ends_with(".jar") {
            classpath.push(runtime.join(&file.destination));
        }
    }
    classpath.push(runtime.join("client.jar"));
    let classpath = std::env::join_paths(classpath)?;
    let mut vars = BTreeMap::new();
    vars.insert("auth_player_name", session.profile.name.clone());
    vars.insert(
        "auth_uuid",
        uuid::Uuid::parse_str(&session.profile.id)?
            .simple()
            .to_string(),
    );
    vars.insert("auth_access_token", session.access_token.clone());
    vars.insert(
        "auth_session",
        format!("token:{}:{}", session.access_token, session.profile.id),
    );
    vars.insert("version_name", plan.version.clone());
    vars.insert("version_type", plan.kind.clone());
    vars.insert("game_directory", text(game)?);
    vars.insert("assets_root", text(&runtime.join("assets"))?);
    vars.insert("assets_index_name", plan.asset_index_id.clone());
    vars.insert("natives_directory", text(&runtime.join("natives"))?);
    vars.insert("library_directory", text(&runtime.join("libraries"))?);
    vars.insert(
        "classpath",
        classpath
            .to_str()
            .context("Classpath is not valid Unicode")?
            .into(),
    );
    vars.insert(
        "classpath_separator",
        if cfg!(windows) { ";" } else { ":" }.into(),
    );
    vars.insert("launcher_name", "Solara Launcher".into());
    vars.insert("launcher_version", env!("CARGO_PKG_VERSION").into());
    vars.insert("user_type", "msa".into());
    vars.insert("user_properties", "{}".into());
    vars.insert("auth_xuid", session.xuid.clone().unwrap_or_default());
    vars.insert(
        "clientid",
        option_env!("SOLARA_MICROSOFT_CLIENT_ID")
            .unwrap_or_default()
            .into(),
    );
    let mut out = vec![OsString::from(format!("-Xmx{memory}M"))];
    if !plan.arguments.is_null() {
        out.extend(arguments(&plan.arguments["jvm"], &vars, os_version)?);
    } else {
        out.push(format!("-Djava.library.path={}", text(&runtime.join("natives"))?).into());
        out.push("-cp".into());
        out.push(classpath);
    }
    if let Some(logging) = &plan.logging_argument {
        let mut logging_vars = vars.clone();
        logging_vars.insert("path", text(&runtime.join("logging.xml"))?);
        out.push(expand(logging, &logging_vars)?.into());
    }
    out.push(plan.main_class.clone().into());
    if !plan.arguments.is_null() {
        out.extend(arguments(&plan.arguments["game"], &vars, os_version)?);
    } else {
        for value in plan
            .legacy_arguments
            .as_deref()
            .context("Legacy Minecraft arguments missing")?
            .split_whitespace()
        {
            out.push(expand(value, &vars)?.into());
        }
    }
    Ok(out)
}

fn java_major(text: &str) -> Result<u32> {
    // Ignore unrelated numbers in JAVA_TOOL_OPTIONS warnings and vendor banners.
    let capture = regex::Regex::new(r#"(?m)^(?:openjdk|java) (?:version )?"(\d+)(?:\.(\d+))?"#)?
        .captures(text)
        .context("Java version could not be identified")?;
    let first: u32 = capture[1].parse()?;
    if first == 1 {
        Ok(capture
            .get(2)
            .context("Java version missing")?
            .as_str()
            .parse()?)
    } else {
        Ok(first)
    }
}

pub async fn java_check(path: &str, required: u32) -> Result<()> {
    if path.trim().is_empty() || path.contains('\0') {
        bail!("Choose a Java path in Settings")
    }
    let mut command = Command::new(path);
    command
        .arg("-version")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    #[cfg(windows)]
    command.creation_flags(0x08000000);
    let mut child = command.spawn().map_err(|_| {
        anyhow::anyhow!("Java was not found. Choose your Java executable in Settings.")
    })?;
    let stdout = child.stdout.take().context("Java output unavailable")?;
    let stderr = child
        .stderr
        .take()
        .context("Java error output unavailable")?;
    let work = async {
        let read = |stream: Box<dyn AsyncRead + Unpin + Send>| async move {
            let mut out = Vec::new();
            stream.take(8192).read_to_end(&mut out).await?;
            Ok::<_, std::io::Error>(out)
        };
        let (status, out, err) =
            tokio::try_join!(child.wait(), read(Box::new(stdout)), read(Box::new(stderr)))?;
        if !status.success() {
            bail!("The selected Java executable could not start")
        }
        let text = format!(
            "{}\n{}",
            String::from_utf8_lossy(&out),
            String::from_utf8_lossy(&err)
        );
        let major = java_major(&text)?;
        if major < required {
            bail!("This Minecraft version needs Java {required} or newer. Update the Java path in Settings.")
        }
        Ok(())
    };
    tokio::time::timeout(std::time::Duration::from_secs(10), work)
        .await
        .context("Java version check timed out")?
}

struct Game {
    child: Child,
    log: Arc<Mutex<String>>,
}
pub struct Games {
    items: Mutex<HashMap<String, Game>>,
}
impl Default for Games {
    fn default() -> Self {
        Self {
            items: Mutex::new(HashMap::new()),
        }
    }
}
impl Games {
    pub async fn running(&self) -> Result<Vec<String>> {
        let mut items = self.items.lock().await;
        let mut running = Vec::new();
        for (id, game) in items.iter_mut() {
            if game.child.try_wait()?.is_none() {
                running.push(id.clone());
            }
        }
        Ok(running)
    }
    pub async fn start(
        &self,
        store: &Instances,
        id: &str,
        session: Session,
        os_version: Option<&str>,
        java_path: &str,
    ) -> Result<u32> {
        let instance = store.get(id).await?;
        if instance.state != "installed" {
            bail!("Finish installing this instance before playing")
        }
        let mut items = self.items.lock().await;
        if let Some(prior) = items.get_mut(id) {
            if prior.child.try_wait()?.is_none() {
                bail!("This Minecraft instance is already running")
            }
        }
        let runtime = store.path(id)?.join("runtime");
        let game_dir = store.game_dir(id).await?;
        let plan: Plan =
            serde_json::from_slice(&tokio::fs::read(runtime.join("plan.json")).await?)?;
        if plan.version != instance.game_version {
            bail!("Installed files do not match this instance")
        }
        java_check(java_path, plan.java_major).await?;
        // Check immutable launch files before passing a verified session to Java.
        for file in &plan.files {
            super::metadata::relative_path(&file.destination)?;
            if !super::download::verified_file(&runtime.join(&file.destination), &file.artifact)
                .await?
            {
                bail!("A Minecraft file is missing or changed. Reinstall this instance before playing.")
            }
        }
        let args = build_args(
            &plan,
            &runtime,
            &game_dir,
            &session,
            instance.memory,
            os_version,
        )?;
        let mut command = Command::new(java_path);
        command
            .args(args)
            .current_dir(game_dir)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(false);
        #[cfg(windows)]
        command.creation_flags(0x08000000);
        let mut child = command.spawn().map_err(|_| {
            anyhow::anyhow!("Minecraft could not start with the selected Java executable")
        })?;
        let pid = child.id().context("Game process identifier unavailable")?;
        let log = Arc::new(Mutex::new(String::new()));
        for stream in [
            child
                .stdout
                .take()
                .map(|s| Box::new(s) as Box<dyn AsyncRead + Unpin + Send>),
            child
                .stderr
                .take()
                .map(|s| Box::new(s) as Box<dyn AsyncRead + Unpin + Send>),
        ]
        .into_iter()
        .flatten()
        {
            let log = log.clone();
            let secrets = vec![session.access_token.clone()];
            tokio::spawn(capture_log(stream, log, secrets));
        }
        items.insert(id.into(), Game { child, log });
        Ok(pid)
    }
    pub async fn logs(&self, id: &str) -> Result<String> {
        let items = self.items.lock().await;
        let game = items
            .get(id)
            .context("No game log is available for this instance")?;
        let value = game.log.lock().await.clone();
        Ok(value)
    }
    pub async fn stop(&self, id: &str) -> Result<()> {
        let mut items = self.items.lock().await;
        let game = items.get_mut(id).context("This instance is not running")?;
        if game.child.try_wait()?.is_some() {
            return Ok(());
        }
        let pid = game.child.id().context("Game process unavailable")?;
        #[cfg(windows)]
        let mut command = {
            let mut c = Command::new("taskkill");
            c.args(["/PID", &pid.to_string()])
                .creation_flags(0x08000000);
            c
        };
        #[cfg(not(windows))]
        let mut command = {
            let mut c = Command::new("kill");
            c.args(["-TERM", &pid.to_string()]);
            c
        };
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(10),
            command
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status(),
        )
        .await;
        if !matches!(result,Ok(Ok(status)) if status.success()) {
            bail!("Close the Minecraft window to save and stop your game")
        }
        match tokio::time::timeout(std::time::Duration::from_secs(15), game.child.wait()).await {
            Ok(Ok(_)) => {}
            Ok(Err(_)) => bail!("Minecraft process state could not be checked"),
            Err(_) => bail!("Minecraft is still closing. Save and close it from the game window."),
        }
        Ok(())
    }
}

async fn capture_log(
    mut stream: Box<dyn AsyncRead + Unpin + Send>,
    log: Arc<Mutex<String>>,
    secrets: Vec<String>,
) {
    let mut buffer = [0u8; 4096];
    let mut line = Vec::new();
    let mut discard = false;
    while let Ok(read) = stream.read(&mut buffer).await {
        if read == 0 {
            if !discard && !line.is_empty() {
                append_log(&log, &line, &secrets).await;
            }
            break;
        }
        for byte in &buffer[..read] {
            if *byte == b'\n' {
                if discard {
                    append_log(&log, b"[Long game log line omitted]", &secrets).await;
                } else {
                    append_log(&log, &line, &secrets).await;
                }
                line.clear();
                discard = false;
            } else if !discard {
                line.push(*byte);
                if line.len() > 65536 {
                    line.clear();
                    discard = true;
                }
            }
        }
    }
}
async fn append_log(log: &Mutex<String>, line: &[u8], secrets: &[String]) {
    let mut text = String::from_utf8_lossy(line).to_string();
    for secret in secrets {
        if !secret.is_empty() {
            text = text.replace(secret, "[redacted]");
        }
    }
    let mut buffer = log.lock().await;
    buffer.push_str(&text);
    buffer.push('\n');
    if buffer.len() > 200000 {
        let mut at = buffer.len() - 200000;
        while !buffer.is_char_boundary(at) {
            at += 1;
        }
        buffer.drain(..at);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn placeholders_keep_paths_as_single_arguments_and_reject_unknowns() {
        let vars = BTreeMap::from([("game_directory", String::from("C:\\Users\\Test Name\\game"))]);
        let raw = serde_json::json!(["--gameDir","${game_directory}",{"rules":[{"action":"allow","features":{"is_demo_user":true}}],"value":"--demo"}]);
        let args = arguments(&raw, &vars, None).unwrap();
        assert_eq!(args.len(), 2);
        assert_eq!(args[1], OsString::from("C:\\Users\\Test Name\\game"));
        assert!(expand("${unsupported}", &vars).is_err());
    }
    #[test]
    fn java_version_ignores_unrelated_warning_numbers() {
        assert_eq!(
            java_major(
                "Picked up JAVA_TOOL_OPTIONS: -Xmx4096M\nopenjdk version \"21.0.8\" 2025-07-15"
            )
            .unwrap(),
            21
        );
        assert_eq!(java_major("java version \"1.8.0_451\"").unwrap(), 8);
        assert!(java_major("warning: 4096 version unavailable").is_err());
    }
    #[tokio::test]
    async fn split_stream_credentials_are_redacted_before_entering_logs() {
        let (mut writer, reader) = tokio::io::duplex(128);
        let log = Arc::new(Mutex::new(String::new()));
        let output = log.clone();
        let task = tokio::spawn(capture_log(
            Box::new(reader),
            output,
            vec!["SECRET_FIXTURE_TOKEN".into()],
        ));
        use tokio::io::AsyncWriteExt;
        writer.write_all(b"token=SECRET_FIX").await.unwrap();
        writer.write_all(b"TURE_TOKEN\n").await.unwrap();
        drop(writer);
        task.await.unwrap();
        let text = log.lock().await;
        assert!(!text.contains("SECRET_FIXTURE_TOKEN"));
        assert!(text.contains("[redacted]"));
    }
}
