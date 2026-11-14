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
    path::{Path, PathBuf},
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
    build_args_with_options(plan, runtime, game, session, memory, os_version, None, None, false)
}

pub fn split_command_args(raw: &str) -> Result<Vec<String>> {
    if raw.contains('\0') {
        bail!("Command arguments cannot contain null bytes");
    }
    if raw
        .chars()
        .any(|c| c.is_control() && c != '\n' && c != '\r' && c != '\t')
    {
        bail!("Command arguments contain unsupported control characters");
    }
    let mut out = Vec::new();
    let mut current = String::new();
    let mut in_token = false;
    let mut quote: Option<char> = None;
    for ch in raw.chars() {
        match quote {
            Some(q) if ch == q => {
                quote = None;
                in_token = true;
            }
            Some(_) => {
                current.push(ch);
                in_token = true;
            }
            None if ch == '"' || ch == '\'' => {
                quote = Some(ch);
                in_token = true;
            }
            None if ch.is_whitespace() => {
                if in_token {
                    out.push(std::mem::take(&mut current));
                    in_token = false;
                }
            }
            None => {
                current.push(ch);
                in_token = true;
            }
        }
    }
    if quote.is_some() {
        bail!("Unclosed quote in command arguments");
    }
    if in_token {
        out.push(current);
    }
    Ok(out)
}

fn split_jvm_args(raw: &str) -> Result<Vec<OsString>> {
    Ok(split_command_args(raw)?
        .into_iter()
        .map(OsString::from)
        .collect())
}

pub async fn run_hook(command_line: &str, working_dir: &Path) -> Result<()> {
    let trimmed = command_line.trim();
    if trimmed.is_empty() {
        return Ok(());
    }
    let parts = split_command_args(trimmed)?;
    let Some(program) = parts.first() else {
        return Ok(());
    };
    if program.is_empty() {
        bail!("Hook command cannot start with an empty executable name");
    }
    let mut command = Command::new(program);
    command
        .args(&parts[1..])
        .current_dir(working_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    #[cfg(windows)]
    command.creation_flags(0x08000000);
    let child = command
        .spawn()
        .with_context(|| format!("Hook command \"{program}\" could not start"))?;
    let output = tokio::time::timeout(
        std::time::Duration::from_secs(60),
        child.wait_with_output(),
    )
    .await
    .map_err(|_| anyhow::anyhow!("Hook command \"{program}\" timed out after 60 seconds"))?
    .with_context(|| format!("Hook command \"{program}\" failed to run"))?;
    if !output.status.success() {
        let stderr_text = String::from_utf8_lossy(&output.stderr);
        let detail = stderr_text.trim();
        let code = output
            .status
            .code()
            .map(|c| format!("exit code {c}"))
            .unwrap_or_else(|| "terminated by signal".to_string());
        if detail.is_empty() {
            bail!("Hook command \"{program}\" exited with {code}");
        } else {
            let truncated: String = detail.chars().take(240).collect();
            bail!("Hook command \"{program}\" exited with {code}: {truncated}");
        }
    }
    Ok(())
}

pub fn build_args_with_options(
    plan: &Plan,
    runtime: &Path,
    game: &Path,
    session: &Session,
    memory: u32,
    os_version: Option<&str>,
    extra_jvm_args: Option<&str>,
    resolution: Option<[u32; 2]>,
    fullscreen: bool,
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
    if let Some(extra) = extra_jvm_args.map(str::trim).filter(|s| !s.is_empty()) {
        out.extend(split_jvm_args(extra)?);
    }
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
    if let Some([w, h]) = resolution {
        out.push("--width".into());
        out.push(w.to_string().into());
        out.push("--height".into());
        out.push(h.to_string().into());
    }
    if fullscreen {
        out.push("--fullscreen".into());
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

pub struct FinishedGame {
    pub instance_id: String,
    pub elapsed_seconds: u64,
    pub game_dir: PathBuf,
    pub post_exit: Option<String>,
}

struct Game {
    child: Child,
    // Child::id() becomes None after try_wait reaps it. Keep identity stable.
    pid: u32,
    log: Arc<Mutex<String>>,
    started_at: std::time::Instant,
    exit_recorded: bool,
    game_dir: PathBuf,
    post_exit: Option<String>,
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
    pub async fn collect_finished(&self) -> Result<Vec<FinishedGame>> {
        let mut items = self.items.lock().await;
        let mut finished = Vec::new();
        for (id, game) in items.iter_mut() {
            if game.child.try_wait()?.is_some() && !game.exit_recorded {
                game.exit_recorded = true;
                finished.push(FinishedGame {
                    instance_id: id.clone(),
                    elapsed_seconds: game.started_at.elapsed().as_secs(),
                    game_dir: game.game_dir.clone(),
                    post_exit: game.post_exit.take(),
                });
            }
        }
        Ok(finished)
    }
    pub async fn poll_until_exited(&self, id: &str, pid: u32) -> Option<std::process::ExitStatus> {
        loop {
            {
                let mut items = self.items.lock().await;
                match items.get_mut(id) {
                    Some(game) if game.pid == pid => {
                        match game.child.try_wait() {
                            Ok(Some(status)) => return Some(status),
                            Ok(None) => {},
                            // A transient inspection error is not evidence of exit.
                            Err(_) => {},
                        }
                    }
                    _ => return None,
                }
            }
            tokio::time::sleep(std::time::Duration::from_millis(800)).await;
        }
    }
    #[allow(clippy::too_many_arguments)]
    pub async fn start(
        &self,
        store: &Instances,
        id: &str,
        session: Session,
        os_version: Option<&str>,
        java_path: &str,
        extra_jvm_args: Option<&str>,
        resolution: Option<[u32; 2]>,
        fullscreen: bool,
        post_exit: Option<String>,
    ) -> Result<u32> {
        let instance = store.get(id).await?;
        if instance.state != "installed" {
            bail!("Finish installing this instance before playing")
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
        let args = build_args_with_options(
            &plan,
            &runtime,
            &game_dir,
            &session,
            instance.memory,
            os_version,
            extra_jvm_args,
            resolution,
            fullscreen,
        )?;
        // Only spawning/registration needs the global process lock. File
        // verification and Java checks must not freeze other games or polling.
        let mut items = self.items.lock().await;
        if let Some(prior) = items.get_mut(id) {
            if prior.child.try_wait()?.is_none() {
                bail!("This Minecraft instance is already running")
            }
        }
        let mut command = Command::new(java_path);
        command
            .args(args)
            .current_dir(&game_dir)
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
        items.insert(
            id.into(),
            Game {
                child,
                pid,
                log,
                started_at: std::time::Instant::now(),
                exit_recorded: false,
                game_dir,
                post_exit: post_exit
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty()),
            },
        );
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
        let pid = {
            let mut items = self.items.lock().await;
            let game = items.get_mut(id).context("This instance is not running")?;
            if game.child.try_wait()?.is_some() {
                return Ok(());
            }
            game.pid
        };
        #[cfg(windows)]
        if !close_process_windows(pid) {
            // Never silently hard-kill a game that may be writing a world.
            bail!("Minecraft has no window to close yet. Wait for it to open, then stop it from the game window.");
        }
        #[cfg(not(windows))]
        {
            let result = tokio::time::timeout(
                std::time::Duration::from_secs(10),
                Command::new("kill").args(["-TERM", &pid.to_string()])
                    .stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).status(),
            ).await;
            if !matches!(result, Ok(Ok(status)) if status.success()) {
                bail!("Close the Minecraft window to save and stop your game");
            }
        }
        // Poll without keeping the global mutex across the shutdown wait. A
        // replaced session is never stopped or mistaken for the previous one.
        tokio::time::timeout(std::time::Duration::from_secs(15), async {
            loop {
                {
                    let mut items = self.items.lock().await;
                    match items.get_mut(id) {
                        Some(game) if game.pid == pid => {
                            if game.child.try_wait()?.is_some() { return Ok(()); }
                        }
                        _ => return Ok(()),
                    }
                }
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
        }).await.context("Minecraft is still closing. Save and close it from the game window.")?
    }
}

#[cfg(windows)]
fn close_process_windows(pid: u32) -> bool {
    use windows_sys::Win32::Foundation::{BOOL, HWND, LPARAM};
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetWindowThreadProcessId, PostMessageW, WM_CLOSE,
    };

    struct EnumData {
        target_pid: u32,
        found: bool,
    }

    unsafe extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let data = &mut *(lparam as *mut EnumData);
        let mut process_id = 0u32;
        GetWindowThreadProcessId(hwnd, &mut process_id);
        if process_id == data.target_pid {
            PostMessageW(hwnd, WM_CLOSE, 0, 0);
            data.found = true;
        }
        1
    }

    let mut data = EnumData {
        target_pid: pid,
        found: false,
    };
    unsafe {
        EnumWindows(Some(enum_proc), &mut data as *mut _ as LPARAM);
    }
    data.found
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
    #[cfg(unix)]
    async fn register_test_process(games: &Games, id: &str, script: &str) -> u32 {
        let mut child = Command::new("sh").args(["-c", script])
            .stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::null())
            .kill_on_drop(true).spawn().unwrap();
        let pid = child.id().unwrap();
        // Wait until the process has installed its signal handlers.
        use tokio::io::{AsyncBufReadExt, BufReader};
        let mut ready = String::new();
        BufReader::new(child.stdout.take().unwrap()).read_line(&mut ready).await.unwrap();
        games.items.lock().await.insert(id.into(), Game {
            child, pid, log: Arc::new(Mutex::new(String::new())),
            started_at: std::time::Instant::now(), exit_recorded: false,
            game_dir: std::env::temp_dir(), post_exit: Some("test-only-hook".into()),
        });
        pid
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn exit_is_detected_after_snapshot_reaps_child_and_collected_once() {
        let games = Games::default();
        let pid = register_test_process(&games, "fixture", "echo ready; exit 7").await;
        tokio::time::timeout(std::time::Duration::from_secs(3), async {
            while !games.running().await.unwrap().is_empty() { tokio::task::yield_now().await; }
        }).await.unwrap();
        let finished = games.collect_finished().await.unwrap();
        assert_eq!(finished.len(), 1);
        assert_eq!(finished[0].post_exit.as_deref(), Some("test-only-hook"));
        assert!(games.collect_finished().await.unwrap().is_empty());
        assert_eq!(games.items.lock().await["fixture"].child.id(), None);
        let status = games.poll_until_exited("fixture", pid).await.unwrap();
        assert_eq!(status.code(), Some(7));
        assert!(!status.success());
        assert!(games.poll_until_exited("fixture", pid.wrapping_add(1)).await.is_none());
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn stopping_a_process_does_not_block_running_snapshot_or_logs() {
        let games = Arc::new(Games::default());
        register_test_process(&games, "fixture", "trap '' TERM; echo ready; sleep 1").await;
        let stop_games = games.clone();
        let stopping = tokio::spawn(async move { stop_games.stop("fixture").await });
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        assert!(!stopping.is_finished(), "fixture must still be shutting down");
        tokio::time::timeout(std::time::Duration::from_millis(200), async {
            assert_eq!(games.running().await.unwrap(), vec!["fixture"]);
            assert_eq!(games.logs("fixture").await.unwrap(), "");
        }).await.expect("Stop must release the global process lock while waiting");
        stopping.await.unwrap().unwrap();
        assert!(games.running().await.unwrap().is_empty());
        assert_eq!(games.collect_finished().await.unwrap().len(), 1);
    }
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

    #[test]
    fn split_command_args_handles_quotes_and_rejects_unclosed() {
        let parsed =
            split_command_args(r#"python3 "/tmp/my script.py" --flag='a b' "" plain"#).unwrap();
        assert_eq!(
            parsed,
            vec!["python3", "/tmp/my script.py", "--flag=a b", "", "plain"]
        );
        let win_parsed =
            split_command_args(r#""C:\Program Files\My Tool\run.exe" --arg "hello world""#).unwrap();
        assert_eq!(
            win_parsed,
            vec![r"C:\Program Files\My Tool\run.exe", "--arg", "hello world"]
        );
        assert!(split_command_args("echo \"unclosed").is_err());
        assert!(split_command_args("echo\0bad").is_err());
    }

    #[tokio::test]
    async fn run_hook_executes_without_shell_and_reports_failures() {
        let dir = std::env::temp_dir().join(format!("solara-hook-test-{}", uuid::Uuid::new_v4()));
        tokio::fs::create_dir_all(&dir).await.unwrap();

        #[cfg(not(windows))]
        {
            // Confirm literal argument delivery (no shell expansion of `&&` or `$()`)
            let marker = dir.join("should-not-exist");
            let cmd = format!("true && touch {}", marker.display());
            run_hook(&cmd, &dir).await.unwrap();
            assert!(!marker.exists(), "hook must not pass through a shell");

            // Non-zero exit fails clearly
            let err = run_hook("false", &dir).await.unwrap_err().to_string();
            assert!(err.contains("exited with"));
        }

        // Empty hook is a no-op; empty quoted program fails clearly
        run_hook("   ", &dir).await.unwrap();
        assert!(run_hook(r#""""#, &dir).await.is_err());

        // Missing binary fails with clear context
        let missing = run_hook("definitely-missing-solara-hook-binary-xyz", &dir)
            .await
            .unwrap_err()
            .to_string();
        assert!(missing.contains("could not start"));

        let _ = tokio::fs::remove_dir_all(&dir).await;
    }
}
