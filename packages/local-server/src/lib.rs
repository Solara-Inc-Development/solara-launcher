use anyhow::{bail, Context, Result};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, VecDeque},
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::{Child, ChildStdin, Command},
    sync::Mutex,
};

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerConfig {
    pub id: String,
    pub name: String,
    pub version: String,
    pub java_path: String,
    pub memory: u32,
    pub port: u16,
    pub eula: bool,
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerView {
    #[serde(flatten)]
    pub config: ServerConfig,
    pub state: String,
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Download {
    pub id: String,
    pub name: String,
    pub status: String,
    pub downloaded: u64,
    pub total: u64,
    pub detail: String,
    pub actions: Vec<String>,
}
struct Running {
    child: Child,
    stdin: ChildStdin,
}
#[derive(Clone)]
pub struct LocalServers {
    root: PathBuf,
    running: Arc<Mutex<HashMap<String, Running>>>,
    logs: Arc<Mutex<HashMap<String, VecDeque<String>>>>,
    downloads: Arc<Mutex<HashMap<String, Download>>>,
}
impl LocalServers {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            running: Default::default(),
            logs: Default::default(),
            downloads: Default::default(),
        }
    }
    fn dir(&self, id: &str) -> Result<PathBuf> {
        uuid::Uuid::parse_str(id).context("Invalid local server identifier")?;
        let p = self.root.join(id);
        if p.exists() && std::fs::symlink_metadata(&p)?.file_type().is_symlink() {
            bail!("Server directories cannot be symbolic links")
        };
        Ok(p)
    }
    async fn config(&self, id: &str) -> Result<ServerConfig> {
        let p = self.dir(id)?.join("solara-server.json");
        let c: ServerConfig = serde_json::from_slice(&tokio::fs::read(p).await?)?;
        if c.id != id {
            bail!("Server identity mismatch")
        };
        Ok(c)
    }
    pub async fn list(&self) -> Result<Vec<ServerView>> {
        let mut out = vec![];
        if !self.root.exists() {
            return Ok(out);
        };
        let mut dirs = tokio::fs::read_dir(&self.root).await?;
        let mut running = self.running.lock().await;
        while let Some(d) = dirs.next_entry().await? {
            let id = d.file_name().to_string_lossy().to_string();
            if let Ok(c) = self.config(&id).await {
                let alive = if let Some(p) = running.get_mut(&id) {
                    p.child.try_wait()?.is_none()
                } else {
                    false
                };
                if !alive {
                    running.remove(&id);
                };
                out.push(ServerView {
                    config: c,
                    state: if alive {
                        "running"
                    } else if d.path().join("server.jar").exists() {
                        "stopped"
                    } else {
                        "setup incomplete"
                    }
                    .into(),
                });
            }
        }
        Ok(out)
    }
    pub async fn downloads(&self) -> Vec<Download> {
        self.downloads.lock().await.values().cloned().collect()
    }
    pub fn validate(config: &ServerConfig) -> Result<()> {
        if config.name.trim().is_empty() || config.name.chars().count() > 80 {
            bail!("Choose a name of 1–80 characters")
        };
        if !(512..=65536).contains(&config.memory) {
            bail!("Memory must be 512–65536 MiB")
        };
        if config.port < 1024 {
            bail!("Use a port from 1024 to 65535")
        };
        if !config.eula {
            bail!("Explicit Minecraft EULA acceptance is required")
        };
        if config.java_path.trim().is_empty() || config.java_path.contains('\0') {
            bail!("Choose a Java executable")
        };
        Ok(())
    }
    pub async fn create(&self, mut config: ServerConfig) -> Result<String> {
        Self::validate(&config)?;
        config.id = uuid::Uuid::new_v4().to_string();
        let dir = self.dir(&config.id)?;
        tokio::fs::create_dir_all(&self.root).await?;
        tokio::fs::create_dir(&dir).await?;
        tokio::fs::write(
            dir.join("solara-server.json"),
            serde_json::to_vec_pretty(&config)?,
        )
        .await?;
        tokio::fs::write(
            dir.join("eula.txt"),
            format!(
                "# Accepted explicitly in Solara at {}\neula=true\n",
                chrono::Utc::now().to_rfc3339()
            ),
        )
        .await?;
        tokio::fs::write(
            dir.join("server.properties"),
            format!(
                "server-port={}\nonline-mode=true\nenable-query=false\nenable-rcon=false\n",
                config.port
            ),
        )
        .await?;
        let id = config.id.clone();
        let owned = self.clone();
        tokio::spawn(async move {
            let download = Download {
                id: config.id.clone(),
                name: format!("{} · server files", config.name),
                status: "running".into(),
                downloaded: 0,
                total: 0,
                detail: "Reading official Minecraft metadata".into(),
                actions: vec![],
            };
            owned
                .downloads
                .lock()
                .await
                .insert(config.id.clone(), download);
            let result = owned.install(&config, &dir).await;
            if let Some(d) = owned.downloads.lock().await.get_mut(&config.id) {
                d.status = if result.is_ok() {
                    "completed"
                } else {
                    "failed"
                }
                .into();
                d.detail = result
                    .err()
                    .map(|e| e.to_string())
                    .unwrap_or_else(|| "Official server JAR verified".into());
            }
        });
        Ok(id)
    }
    async fn install(&self, c: &ServerConfig, dir: &Path) -> Result<()> {
        let http = reqwest::Client::builder()
            .user_agent("SolaraLauncher/0.0.1 (https://launcher.solara-inc.in)")
            .timeout(std::time::Duration::from_secs(45))
            .redirect(reqwest::redirect::Policy::none())
            .build()?;
        let manifest: serde_json::Value = http
            .get("https://piston-meta.mojang.com/mc/game/version_manifest_v2.json")
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        let v = manifest["versions"]
            .as_array()
            .context("Invalid official version manifest")?
            .iter()
            .find(|v| v["id"] == c.version)
            .context("Minecraft version not in official manifest")?;
        let meta_url = v["url"].as_str().context("Missing version metadata")?;
        trusted(
            meta_url,
            &["piston-meta.mojang.com", "launchermeta.mojang.com"],
        )?;
        let meta_bytes = http
            .get(meta_url)
            .send()
            .await?
            .error_for_status()?
            .bytes()
            .await?;
        if meta_bytes.len() > 8_000_000 {
            bail!("Version metadata too large")
        };
        if let Some(hash) = v["sha1"].as_str() {
            verify_sha1(&meta_bytes, hash)?
        };
        let meta: serde_json::Value = serde_json::from_slice(&meta_bytes)?;
        let d = &meta["downloads"]["server"];
        let url = d["url"]
            .as_str()
            .context("This version has no official Java server download")?;
        trusted(url, &["piston-data.mojang.com", "launcher.mojang.com"])?;
        let expected = d["sha1"]
            .as_str()
            .context("Missing official server checksum")?;
        let total = d["size"].as_u64().context("Missing server download size")?;
        if total > 512 * 1024 * 1024 {
            bail!("Server artifact exceeds size limit")
        };
        let mut r = http
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .bytes_stream();
        let temp = dir.join("server.jar.part");
        let mut f = tokio::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)
            .await?;
        let mut hash = sha1_smol::Sha1::new();
        let mut count = 0;
        let result: Result<()> = async {
            while let Some(chunk) = r.next().await {
                let bytes = chunk?;
                count += bytes.len() as u64;
                if count > total {
                    bail!("Server download exceeded expected size")
                };
                hash.update(&bytes);
                f.write_all(&bytes).await?;
                if let Some(progress) = self.downloads.lock().await.get_mut(&c.id) {
                    progress.downloaded = count;
                    progress.total = total;
                    progress.detail = "Downloading from Mojang".into();
                }
            }
            f.flush().await?;
            if count != total || hash.digest().to_string() != expected {
                bail!("Official server checksum/size verification failed")
            };
            Ok(())
        }
        .await;
        drop(f);
        if let Err(e) = result {
            let _ = tokio::fs::remove_file(temp).await;
            return Err(e);
        };
        tokio::fs::rename(temp, dir.join("server.jar")).await?;
        Ok(())
    }
    pub async fn start(&self, id: &str) -> Result<()> {
        let config = self.config(id).await?;
        Self::validate(&config)?;
        let dir = self.dir(id)?;
        if !dir.join("server.jar").is_file() {
            bail!("Server setup incomplete: verified server.jar missing")
        };
        let eula = tokio::fs::read_to_string(dir.join("eula.txt")).await?;
        if !eula.lines().any(|s| s.trim() == "eula=true") {
            bail!("Minecraft EULA has not been accepted")
        };
        let mut running = self.running.lock().await;
        if let Some(p) = running.get_mut(id) {
            if p.child.try_wait()?.is_none() {
                bail!("Server is already running")
            }
        };
        let port_check =
            tokio::net::TcpListener::bind((std::net::Ipv4Addr::UNSPECIFIED, config.port))
                .await
                .context("The configured port is already occupied or unavailable")?;
        drop(port_check);
        let java = Command::new(&config.java_path)
            .arg("-version")
            .output()
            .await
            .context("Java not found: choose an installed Java executable")?;
        if !java.status.success() {
            bail!("Java executable check failed")
        };
        let mut child = Command::new(&config.java_path)
            .arg(format!("-Xmx{}M", config.memory))
            .args(["-jar", "server.jar", "nogui"])
            .current_dir(&dir)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .context("Local Java server could not start")?;
        let stdin = child
            .stdin
            .take()
            .context("Local console input unavailable")?;
        for stream in [
            child
                .stdout
                .take()
                .map(|x| Box::pin(x) as std::pin::Pin<Box<dyn tokio::io::AsyncRead + Send>>),
            child
                .stderr
                .take()
                .map(|x| Box::pin(x) as std::pin::Pin<Box<dyn tokio::io::AsyncRead + Send>>),
        ]
        .into_iter()
        .flatten()
        {
            let log = self.logs.clone();
            let key = id.to_owned();
            tokio::spawn(async move {
                let mut reader = BufReader::new(stream);
                let mut buffer = Vec::new();
                loop {
                    buffer.clear();
                    match reader.fill_buf().await {
                        Ok(b) if b.is_empty() => break,
                        Ok(b) => {
                            let take = b.len().min(4096);
                            buffer.extend_from_slice(&b[..take]);
                            reader.consume(take);
                        }
                        Err(_) => break,
                    };
                    let line = String::from_utf8_lossy(&buffer).into_owned();
                    let mut guard = log.lock().await;
                    let q = guard.entry(key.clone()).or_default();
                    q.push_back(line);
                    while q.len() > 256 {
                        q.pop_front();
                    }
                }
            });
        }
        running.insert(id.to_owned(), Running { child, stdin });
        Ok(())
    }
    pub async fn stop(&self, id: &str) -> Result<()> {
        let mut running = self.running.lock().await;
        let p = running.get_mut(id).context("Local server is not running")?;
        if p.child.try_wait()?.is_some() {
            running.remove(id);
            return Ok(());
        };
        p.stdin.write_all(b"stop\n").await?;
        p.stdin.flush().await?;
        match tokio::time::timeout(std::time::Duration::from_secs(30), p.child.wait()).await {
            Ok(exit) => {
                exit?;
                running.remove(id);
                Ok(())
            }
            Err(_) => {
                bail!("Server has not stopped after 30 seconds; check local console before closing")
            }
        }
    }
    pub async fn action(&self, id: &str, action: &str) -> Result<()> {
        match action {
            "start" => self.start(id).await,
            "stop" => self.stop(id).await,
            "restart" => {
                self.stop(id).await?;
                self.start(id).await
            }
            _ => bail!("Unsupported local server action"),
        }
    }
    pub async fn command(&self, id: &str, command: &str) -> Result<()> {
        if command.is_empty() || command.len() > 500 || command.contains(['\r', '\n', '\0']) {
            bail!("Use a single console command up to 500 characters")
        };
        let mut running = self.running.lock().await;
        let p = running.get_mut(id).context("Local server is not running")?;
        p.stdin.write_all(format!("{command}\n").as_bytes()).await?;
        p.stdin.flush().await?;
        Ok(())
    }
    pub async fn logs(&self, id: &str) -> Result<String> {
        self.config(id).await?;
        Ok(self
            .logs
            .lock()
            .await
            .get(id)
            .map(|q| q.iter().cloned().collect::<Vec<_>>().join(""))
            .unwrap_or_default())
    }
    pub async fn stop_all(&self) -> Result<()> {
        let ids = self
            .running
            .lock()
            .await
            .keys()
            .cloned()
            .collect::<Vec<_>>();
        for id in ids {
            self.stop(&id).await?
        }
        Ok(())
    }
}
fn trusted(raw: &str, hosts: &[&str]) -> Result<()> {
    let u = reqwest::Url::parse(raw)?;
    if u.scheme() != "https"
        || !u.username().is_empty()
        || u.password().is_some()
        || !hosts.contains(&u.host_str().unwrap_or_default())
    {
        bail!("Unexpected official download host")
    };
    Ok(())
}
fn verify_sha1(bytes: &[u8], expected: &str) -> Result<()> {
    if sha1_smol::Sha1::from(bytes).digest().to_string() != expected {
        bail!("Official metadata checksum mismatch")
    };
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    fn config() -> ServerConfig {
        ServerConfig {
            id: String::new(),
            name: "Test".into(),
            version: "1.21.1".into(),
            java_path: "java".into(),
            memory: 2048,
            port: 25565,
            eula: true,
        }
    }
    #[test]
    fn explicit_eula_and_bounds() {
        assert!(LocalServers::validate(&config()).is_ok());
        let mut c = config();
        c.eula = false;
        assert!(LocalServers::validate(&c).is_err());
        c = config();
        c.port = 80;
        assert!(LocalServers::validate(&c).is_err());
        c = config();
        c.memory = 10;
        assert!(LocalServers::validate(&c).is_err());
    }
    #[test]
    fn official_source_and_integrity() {
        assert!(trusted(
            "https://piston-data.mojang.com/v1/x",
            &["piston-data.mojang.com"]
        )
        .is_ok());
        assert!(trusted(
            "https://piston-data.mojang.com.evil.example/a",
            &["piston-data.mojang.com"]
        )
        .is_err());
        assert!(trusted(
            "http://piston-data.mojang.com/a",
            &["piston-data.mojang.com"]
        )
        .is_err());
        assert!(verify_sha1(b"test", "a94a8fe5ccb19ba61c4c0873d391e987982fbbd3").is_ok());
        assert!(verify_sha1(b"bad", "a94a8fe5ccb19ba61c4c0873d391e987982fbbd3").is_err());
    }
    #[tokio::test]
    async fn path_and_missing_process_errors() {
        let service =
            LocalServers::new(std::env::temp_dir().join(uuid::Uuid::new_v4().to_string()));
        assert!(service.dir("../../etc").is_err());
        assert!(service.stop("missing").await.is_err());
        assert!(service.command("missing", "a\nb").await.is_err());
        assert!(service.list().await.unwrap().is_empty());
    }
}
