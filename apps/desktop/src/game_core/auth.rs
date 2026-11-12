//! Independent Microsoft -> Xbox -> Minecraft flow. Never contacts Solara.
use super::{download::Cancellation, instances::write_json};
use anyhow::{bail, Context, Result};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::Mutex;

const MICROSOFT: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0";
const MINECRAFT: &str = "https://api.minecraftservices.com";

fn default_true() -> bool {
    true
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Profile {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub active: bool,
    #[serde(default = "default_true")]
    pub owned: bool,
}
impl Profile {
    fn validate(&self) -> Result<()> {
        if uuid::Uuid::parse_str(&self.id)?.to_string() != self.id
            || !(1..=16).contains(&self.name.len())
            || !self
                .name
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_')
        {
            bail!("Invalid Minecraft profile")
        }
        Ok(())
    }
}
#[derive(Default, Serialize, Deserialize)]
struct Index {
    accounts: Vec<Profile>,
}
pub struct Session {
    pub profile: Profile,
    pub access_token: String,
    pub xuid: Option<String>,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DevicePrompt {
    pub code: String,
    pub url: String,
    pub expires_in: u64,
    #[serde(default)]
    pub direct: bool,
}

fn client_id() -> Result<&'static str> {
    let value = option_env!("SOLARA_MICROSOFT_CLIENT_ID").unwrap_or_default();
    if !value.is_empty()
        && value.len() <= 100
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-')
    {
        return Ok(value);
    }
    // Default public client ID for Solara Microsoft OAuth Flow
    Ok("6a9238bb-3803-46f4-bf38-2f45bdf8c2ea")
}

pub fn generate_pkce() -> (String, String) {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    let verifier = URL_SAFE_NO_PAD.encode(bytes);
    let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
    (verifier, challenge)
}

pub fn loopback_auth_url(client_id: &str, port: u16, code_challenge: &str) -> String {
    format!(
        "{MICROSOFT}/authorize?client_id={client_id}&response_type=code&redirect_uri=http://localhost:{port}/callback&scope=XboxLive.signin%20offline_access&code_challenge={code_challenge}&code_challenge_method=S256&prompt=select_account"
    )
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

pub fn auth_success_html() -> &'static str {
    r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Authentication Complete - Solara Launcher</title>
  <style>
    * { box-sizing: border-box; margin: 0; padding: 0; }
    body {
      background: #090a0f;
      color: #e4e4e7;
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
      display: flex;
      align-items: center;
      justify-content: center;
      min-height: 100vh;
      padding: 24px;
    }
    .card {
      background: radial-gradient(120% 120% at 50% 0%, #1a1c29 0%, #11121a 100%);
      border: 1px solid rgba(255, 255, 255, 0.08);
      border-radius: 20px;
      padding: 44px 36px;
      max-width: 440px;
      width: 100%;
      text-align: center;
      box-shadow: 0 24px 60px rgba(0, 0, 0, 0.6), 0 0 1px 1px rgba(255, 255, 255, 0.05) inset;
    }
    .icon-wrapper {
      width: 72px;
      height: 72px;
      margin: 0 auto 24px;
      border-radius: 50%;
      background: radial-gradient(circle at 30% 30%, #22c55e, #15803d);
      display: flex;
      align-items: center;
      justify-content: center;
      box-shadow: 0 0 28px rgba(34, 197, 94, 0.45);
    }
    .icon-wrapper svg {
      width: 38px;
      height: 38px;
      stroke: #ffffff;
      stroke-width: 3;
      fill: none;
      stroke-linecap: round;
      stroke-linejoin: round;
    }
    .brand {
      font-size: 13px;
      letter-spacing: 0.12em;
      text-transform: uppercase;
      font-weight: 700;
      color: #06b6d4;
      margin-bottom: 8px;
    }
    h1 {
      font-size: 24px;
      font-weight: 700;
      color: #ffffff;
      margin-bottom: 12px;
      letter-spacing: -0.01em;
    }
    p {
      color: #94a3b8;
      font-size: 15px;
      line-height: 1.55;
      margin-bottom: 24px;
    }
    .badge {
      display: inline-flex;
      align-items: center;
      gap: 6px;
      padding: 6px 14px;
      background: rgba(34, 197, 94, 0.12);
      border: 1px solid rgba(34, 197, 94, 0.25);
      border-radius: 9999px;
      color: #4ade80;
      font-size: 12px;
      font-weight: 500;
    }
  </style>
  <script>
    setTimeout(function() {
      try { window.close(); } catch(e) {}
    }, 2500);
  </script>
</head>
<body>
  <div class="card">
    <div class="icon-wrapper">
      <svg viewBox="0 0 24 24"><polyline points="20 6 9 17 4 12"></polyline></svg>
    </div>
    <div class="brand">Solara Launcher</div>
    <h1>Authentication Complete!</h1>
    <p>You can close this tab and return to Solara Launcher.</p>
    <div class="badge">&#10003; Safe to close window</div>
  </div>
</body>
</html>"#
}

pub fn auth_error_html(msg: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>Sign-in Cancelled - Solara Launcher</title>
  <style>
    body {{ background: #090a0f; color: #e4e4e7; font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; display: flex; align-items: center; justify-content: center; min-height: 100vh; margin: 0; }}
    .card {{ background: #11121a; border: 1px solid rgba(255, 255, 255, 0.08); border-radius: 20px; padding: 44px 36px; max-width: 440px; text-align: center; }}
    h1 {{ color: #f87171; font-size: 22px; margin-bottom: 12px; }}
    p {{ color: #94a3b8; font-size: 15px; line-height: 1.5; }}
  </style>
</head>
<body>
  <div class="card">
    <h1>Authentication Cancelled</h1>
    <p>{msg}</p>
    <p style="margin-top: 12px; font-size: 13px; color: #64748b;">You can close this tab and return to Solara Launcher.</p>
  </div>
</body>
</html>"#,
        msg = html_escape(msg)
    )
}

async fn read_http_request(socket: &mut tokio::net::TcpStream) -> Result<String> {
    let mut buf = Vec::with_capacity(2048);
    let mut temp = [0u8; 1024];
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        if buf.windows(4).any(|w| w == b"\r\n\r\n") || buf.windows(2).any(|w| w == b"\n\n") {
            break;
        }
        if buf.len() >= 16 * 1024 {
            bail!("Request headers exceeded limit");
        }
        let now = tokio::time::Instant::now();
        if now >= deadline {
            bail!("Timed out reading HTTP request");
        }
        let remaining = deadline - now;
        let n = match tokio::time::timeout(remaining, socket.read(&mut temp)).await {
            Ok(Ok(n)) if n > 0 => n,
            _ => break,
        };
        buf.extend_from_slice(&temp[..n]);
    }
    String::from_utf8(buf).map_err(|_| anyhow::anyhow!("Request is not valid UTF-8"))
}

fn handle_connection(
    mut socket: tokio::net::TcpStream,
    peer: std::net::SocketAddr,
    port: u16,
    tx: tokio::sync::mpsc::Sender<Result<String>>,
) {
    tokio::spawn(async move {
        if !peer.ip().is_loopback() {
            return;
        }
        let request = match read_http_request(&mut socket).await {
            Ok(r) => r,
            Err(_) => return,
        };
        let line = match request.lines().find(|l| !l.trim().is_empty()) {
            Some(l) => l,
            None => return,
        };
        let mut parts = line.split_whitespace();
        let method = parts.next().unwrap_or_default();
        let path = parts.next().unwrap_or_default();

        if method != "GET" {
            let _ = socket
                .write_all(b"HTTP/1.1 405 Method Not Allowed\r\nConnection: close\r\n\r\n")
                .await;
            let _ = socket.shutdown().await;
            return;
        }

        if path == "/favicon.ico" {
            let _ = socket
                .write_all(b"HTTP/1.1 404 Not Found\r\nConnection: close\r\n\r\n")
                .await;
            let _ = socket.shutdown().await;
            return;
        }

        if !path.starts_with("/callback") {
            let _ = socket
                .write_all(b"HTTP/1.1 404 Not Found\r\nConnection: close\r\n\r\n")
                .await;
            let _ = socket.shutdown().await;
            return;
        }

        let callback = match reqwest::Url::parse(&format!("http://localhost:{port}{path}")) {
            Ok(u) => u,
            Err(_) => {
                let _ = socket
                    .write_all(b"HTTP/1.1 400 Bad Request\r\nConnection: close\r\n\r\nInvalid URL")
                    .await;
                let _ = socket.shutdown().await;
                return;
            }
        };

        let params: std::collections::HashMap<_, _> = callback.query_pairs().collect();

        if let Some(err) = params.get("error") {
            let desc = params
                .get("error_description")
                .map(|s| s.as_ref())
                .unwrap_or(err.as_ref());
            let html = auth_error_html(desc);
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n{}",
                html.len(),
                html
            );
            let _ = socket.write_all(resp.as_bytes()).await;
            let _ = socket.flush().await;
            let _ = socket.shutdown().await;
            let _ = tx
                .send(Err(anyhow::anyhow!("Microsoft sign-in was cancelled: {desc}")))
                .await;
            return;
        }

        if let Some(code) = params.get("code") {
            let html = auth_success_html();
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n{}",
                html.len(),
                html
            );
            let _ = socket.write_all(resp.as_bytes()).await;
            let _ = socket.flush().await;
            let _ = socket.shutdown().await;
            let _ = tx.send(Ok(code.to_string())).await;
            return;
        }

        let _ = socket
            .write_all(b"HTTP/1.1 400 Bad Request\r\nConnection: close\r\n\r\nMissing authorization code")
            .await;
        let _ = socket.shutdown().await;
    });
}

pub async fn wait_for_callback(
    listener_v4: tokio::net::TcpListener,
    listener_v6: Option<tokio::net::TcpListener>,
    port: u16,
    cancel: &Cancellation,
) -> Result<String> {
    let (tx, mut rx) = tokio::sync::mpsc::channel::<Result<String>>(1);
    loop {
        tokio::select! {
            _ = cancel.cancelled() => {
                bail!("Microsoft sign-in was cancelled");
            }
            _ = tokio::time::sleep(std::time::Duration::from_secs(300)) => {
                bail!("Microsoft sign-in timed out. Try again.");
            }
            Some(res) = rx.recv() => {
                return res;
            }
            conn = listener_v4.accept() => {
                if let Ok((socket, peer)) = conn {
                    handle_connection(socket, peer, port, tx.clone());
                }
            }
            conn = async {
                match &listener_v6 {
                    Some(l) => l.accept().await,
                    None => std::future::pending().await,
                }
            } => {
                if let Ok((socket, peer)) = conn {
                    handle_connection(socket, peer, port, tx.clone());
                }
            }
        }
    }
}
fn client() -> Result<reqwest::Client> {
    Ok(reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(30))
        .build()?)
}
async fn bounded(mut response: reqwest::Response) -> Result<Value> {
    let mut bytes = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| anyhow::anyhow!("Account service response could not be read"))?
    {
        if bytes.len() + chunk.len() > 256 * 1024 {
            bail!("Account service response exceeds limit")
        }
        bytes.extend_from_slice(&chunk);
    }
    serde_json::from_slice(&bytes).map_err(|_| anyhow::anyhow!("Invalid account service response"))
}
fn secret(value: &Value, key: &str) -> Result<String> {
    let text = value[key]
        .as_str()
        .filter(|s| !s.is_empty() && s.len() <= 32768 && s.is_ascii())
        .context("Account service returned invalid credentials")?;
    Ok(text.into())
}
async fn post(url: &str, body: Value) -> Result<Value> {
    let response = client()?
        .post(url)
        .header("x-xbl-contract-version", "1")
        .json(&body)
        .send()
        .await
        .map_err(|_| anyhow::anyhow!("Microsoft, Xbox or Minecraft service is unavailable"))?;
    if !response.status().is_success() {
        bail!("Account verification failed. Check your Minecraft ownership, Xbox account settings and application approval.")
    }
    bounded(response).await
}
async fn minecraft(oauth: &Value) -> Result<Session> {
    let access = secret(oauth, "access_token")?;
    let xbox=post("https://user.auth.xboxlive.com/user/authenticate",json!({"Properties":{"AuthMethod":"RPS","SiteName":"user.auth.xboxlive.com","RpsTicket":format!("d={access}")},"RelyingParty":"http://auth.xboxlive.com","TokenType":"JWT"})).await?;
    let token = secret(&xbox, "Token")?;
    let xsts=post("https://xsts.auth.xboxlive.com/xsts/authorize",json!({"Properties":{"SandboxId":"RETAIL","UserTokens":[token]},"RelyingParty":"rp://api.minecraftservices.com/","TokenType":"JWT"})).await?;
    let uhs = xsts["DisplayClaims"]["xui"][0]["uhs"]
        .as_str()
        .filter(|s| !s.is_empty() && s.len() < 100 && s.bytes().all(|b| b.is_ascii_digit()))
        .context("Invalid Xbox account claim")?;
    let xsts_token = secret(&xsts, "Token")?;
    let login = post(
        &format!("{MINECRAFT}/authentication/login_with_xbox"),
        json!({"identityToken":format!("XBL3.0 x={uhs};{xsts_token}")}),
    )
    .await?;
    let access_token = secret(&login, "access_token")?;
    let http = client()?;
    let response = http
        .get(format!("{MINECRAFT}/entitlements/mcstore"))
        .bearer_auth(&access_token)
        .send()
        .await
        .map_err(|_| anyhow::anyhow!("Minecraft ownership check is unavailable"))?;
    if !response.status().is_success() {
        bail!("Minecraft ownership could not be verified")
    }
    let owned = bounded(response).await?;
    if !owned["items"].as_array().is_some_and(|items| {
        items.iter().any(|i| {
            matches!(
                i["name"].as_str(),
                Some("game_minecraft" | "product_minecraft")
            )
        })
    }) {
        bail!("This account does not have a verified Minecraft Java licence")
    }
    let response = http
        .get(format!("{MINECRAFT}/minecraft/profile"))
        .bearer_auth(&access_token)
        .send()
        .await
        .map_err(|_| anyhow::anyhow!("Minecraft profile service is unavailable"))?;
    if !response.status().is_success() {
        bail!("Minecraft Java profile is not available for this account")
    }
    let value = bounded(response).await?;
    let id = uuid::Uuid::parse_str(
        value["id"]
            .as_str()
            .context("Minecraft profile identifier missing")?,
    )
    .map_err(|_| anyhow::anyhow!("Invalid Minecraft profile identifier"))?
    .to_string();
    let profile = Profile {
        id,
        name: value["name"]
            .as_str()
            .context("Minecraft profile name missing")?
            .into(),
        active: true,
        owned: true,
    };
    profile.validate()?;
    let xuid = xsts["DisplayClaims"]["xui"][0]["xid"]
        .as_str()
        .filter(|s| !s.is_empty() && s.len() < 100 && s.bytes().all(|b| b.is_ascii_digit()))
        .map(String::from);
    Ok(Session {
        profile,
        access_token,
        xuid,
    })
}

pub struct Accounts {
    root: PathBuf,
    mutation: Mutex<()>,
}
impl Accounts {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            mutation: Mutex::new(()),
        }
    }
    async fn index(&self) -> Result<Index> {
        let path = self.root.join("accounts.json");
        let meta = match tokio::fs::symlink_metadata(&path).await {
            Ok(m) => m,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Index::default()),
            Err(e) => return Err(e.into()),
        };
        if !meta.is_file() || meta.file_type().is_symlink() || meta.len() > 64 * 1024 {
            bail!("Invalid local Minecraft account list")
        }
        let index: Index = serde_json::from_slice(&tokio::fs::read(path).await?)
            .map_err(|_| anyhow::anyhow!("Invalid local Minecraft account list"))?;
        if index.accounts.len() > 20 || index.accounts.iter().filter(|a| a.active).count() > 1 {
            bail!("Invalid local Minecraft account list")
        }
        for profile in &index.accounts {
            profile.validate()?
        }
        Ok(index)
    }
    async fn save(&self, index: &Index) -> Result<()> {
        tokio::fs::create_dir_all(&self.root).await?;
        write_json(&self.root.join("accounts.json"), index).await
    }
    pub async fn list(&self) -> Result<Vec<Profile>> {
        Ok(self.index().await?.accounts)
    }
    pub async fn select(&self, id: &str) -> Result<()> {
        super::instances::instance_id(id)?;
        let _lock = self.mutation.lock().await;
        let mut index = self.index().await?;
        if !index.accounts.iter().any(|a| a.id == id) {
            bail!("Choose a saved Minecraft account")
        }
        for profile in &mut index.accounts {
            profile.active = profile.id == id;
        }
        self.save(&index).await
    }
    pub async fn remove(&self, id: &str) -> Result<()> {
        super::instances::instance_id(id)?;
        let _lock = self.mutation.lock().await;
        let mut index = self.index().await?;
        vault_remove(id)?;
        index.accounts.retain(|a| a.id != id);
        if !index.accounts.iter().any(|a| a.active) {
            if let Some(first) = index.accounts.first_mut() {
                first.active = true;
            }
        }
        self.save(&index).await
    }
    async fn accept(&self, session: &Session, refresh: &str) -> Result<()> {
        let mut index = self.index().await?;
        session.profile.validate()?;
        if index.accounts.len() >= 20 && !index.accounts.iter().any(|a| a.id == session.profile.id)
        {
            bail!("Remove a saved Minecraft account before adding another")
        }
        vault_put(&session.profile.id, refresh)?;
        index.accounts.retain(|a| a.id != session.profile.id);
        for a in &mut index.accounts {
            a.active = false;
        }
        index.accounts.push(session.profile.clone());
        self.save(&index).await
    }
    pub async fn session(&self) -> Result<Session> {
        let _lock = self.mutation.lock().await;
        let profile = self
            .index()
            .await?
            .accounts
            .into_iter()
            .find(|a| a.active)
            .context("Add a licensed Minecraft account first")?;
        let refresh = vault_get(&profile.id)?;
        let response = client()?
            .post(format!("{MICROSOFT}/token"))
            .form(&[
                ("client_id", client_id()?),
                ("grant_type", "refresh_token"),
                ("refresh_token", refresh.as_str()),
            ])
            .send()
            .await
            .map_err(|_| anyhow::anyhow!("Microsoft sign-in service is unavailable"))?;
        if !response.status().is_success() {
            bail!("Minecraft session expired. Sign in again.")
        }
        let oauth = bounded(response).await?;
        let session = minecraft(&oauth).await?;
        if session.profile.id != profile.id {
            bail!("Minecraft session account does not match the selected profile")
        }
        let rotated = oauth["refresh_token"].as_str().unwrap_or(&refresh);
        self.accept(&session, rotated).await?;
        Ok(session)
    }
    pub async fn login(
        &self,
        cancel: &Cancellation,
        prompt: impl FnMut(DevicePrompt),
    ) -> Result<()> {
        self.login_loopback(cancel, prompt).await
    }
    pub async fn login_loopback(
        &self,
        cancel: &Cancellation,
        mut prompt: impl FnMut(DevicePrompt),
    ) -> Result<()> {
        if !cfg!(windows) {
            bail!("Minecraft sign-in needs Windows secure account storage")
        }
        let id = client_id()?;
        cancel.check()?;

        let listener_v4 = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .context("Failed to bind ephemeral local loopback listener")?;
        let port = listener_v4.local_addr()?.port();
        let listener_v6 = tokio::net::TcpListener::bind(format!("[::1]:{port}")).await.ok();
        let redirect_uri = format!("http://localhost:{port}/callback");

        let (code_verifier, code_challenge) = generate_pkce();
        let auth_url = loopback_auth_url(id, port, &code_challenge);

        // Notify caller and open browser
        prompt(DevicePrompt {
            code: String::new(),
            url: auth_url,
            expires_in: 300,
            direct: true,
        });

        let code = wait_for_callback(listener_v4, listener_v6, port, cancel).await?;
        cancel.check()?;

        let response = client()?
            .post(format!("{MICROSOFT}/token"))
            .form(&[
                ("client_id", id),
                ("grant_type", "authorization_code"),
                ("code", code.as_str()),
                ("redirect_uri", redirect_uri.as_str()),
                ("code_verifier", code_verifier.as_str()),
            ])
            .send()
            .await
            .map_err(|_| anyhow::anyhow!("Microsoft token exchange service is unavailable"))?;

        let success = response.status().is_success();
        let oauth = bounded(response).await?;
        if !success {
            let reason = oauth["error_description"]
                .as_str()
                .or(oauth["error"].as_str())
                .unwrap_or("Microsoft token exchange failed");
            bail!("{reason}");
        }

        let session = minecraft(&oauth).await?;
        cancel.check()?;
        let refresh = secret(&oauth, "refresh_token")?;
        let _lock = self.mutation.lock().await;
        self.accept(&session, &refresh).await?;
        Ok(())
    }
    pub async fn login_device_code(
        &self,
        cancel: &Cancellation,
        mut prompt: impl FnMut(DevicePrompt),
    ) -> Result<()> {
        if !cfg!(windows) {
            bail!("Minecraft sign-in needs Windows secure account storage")
        }
        let id = client_id()?;
        cancel.check()?;
        let response = client()?
            .post(format!("{MICROSOFT}/devicecode"))
            .form(&[
                ("client_id", id),
                ("scope", "XboxLive.signin offline_access"),
            ])
            .send()
            .await
            .map_err(|_| anyhow::anyhow!("Microsoft sign-in service is unavailable"))?;
        if !response.status().is_success() {
            bail!("Microsoft sign-in could not start. Check the app registration and public-client device flow.")
        }
        let flow = bounded(response).await?;
        let device = secret(&flow, "device_code")?;
        let code = secret(&flow, "user_code")?;
        let url = flow["verification_uri"]
            .as_str()
            .context("Microsoft verification address missing")?;
        let parsed = reqwest::Url::parse(url)?;
        if parsed.scheme() != "https"
            || !matches!(
                parsed.host_str(),
                Some(
                    "microsoft.com"
                        | "www.microsoft.com"
                        | "login.microsoftonline.com"
                        | "login.live.com"
                )
            )
            || !parsed.username().is_empty()
            || parsed.password().is_some()
            || parsed.port().is_some()
        {
            bail!("Untrusted Microsoft verification address")
        }
        let expires = flow["expires_in"]
            .as_u64()
            .filter(|n| *n > 0 && *n <= 1800)
            .context("Invalid Microsoft sign-in expiry")?;
        let mut interval = flow["interval"].as_u64().unwrap_or(5).clamp(5, 30);
        prompt(DevicePrompt {
            code,
            url: url.into(),
            expires_in: expires,
            direct: false,
        });
        let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(expires);
        while tokio::time::Instant::now() < deadline {
            cancel.check()?;
            tokio::time::sleep(std::time::Duration::from_secs(interval)).await;
            cancel.check()?;
            let response = client()?
                .post(format!("{MICROSOFT}/token"))
                .form(&[
                    ("client_id", id),
                    ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
                    ("device_code", device.as_str()),
                ])
                .send()
                .await
                .map_err(|_| anyhow::anyhow!("Microsoft sign-in service is unavailable"))?;
            let success = response.status().is_success();
            let value = bounded(response).await?;
            if success {
                let session = minecraft(&value).await?;
                cancel.check()?;
                let refresh = secret(&value, "refresh_token")?;
                let _lock = self.mutation.lock().await;
                self.accept(&session, &refresh).await?;
                return Ok(());
            }
            match value["error"].as_str() {
                Some("authorization_pending") => {}
                Some("slow_down") => interval = (interval + 5).min(60),
                Some("authorization_declined" | "access_denied") => {
                    bail!("Microsoft sign-in was cancelled")
                }
                Some("expired_token") => break,
                _ => bail!("Microsoft sign-in failed. Check application approval and try again."),
            }
        }
        bail!("Microsoft sign-in expired. Try again.")
    }
}

// Only the refresh token is persisted, inside the OS vault. Minecraft access
// tokens are short-lived in-memory values and never enter account JSON or logs.
#[cfg(windows)]
fn vault_put(id: &str, token: &str) -> Result<()> {
    keyring::Entry::new("in.solara.launcher.minecraft.custom", id)
        .and_then(|e| e.set_password(token))
        .map_err(|_| anyhow::anyhow!("Windows could not save the Minecraft session securely"))?;
    Ok(())
}
#[cfg(windows)]
fn vault_get(id: &str) -> Result<String> {
    keyring::Entry::new("in.solara.launcher.minecraft.custom", id)
        .and_then(|e| e.get_password())
        .map_err(|_| {
            anyhow::anyhow!(
                "Minecraft session is missing from Windows secure storage. Sign in again."
            )
        })
}
#[cfg(windows)]
fn vault_remove(id: &str) -> Result<()> {
    match keyring::Entry::new("in.solara.launcher.minecraft.custom", id)
        .and_then(|e| e.delete_credential())
    {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(_) => bail!("Windows could not remove the saved Minecraft session"),
    }
}
#[cfg(not(windows))]
fn vault_put(_: &str, _: &str) -> Result<()> {
    bail!("Windows secure storage is required")
}
#[cfg(not(windows))]
fn vault_get(_: &str) -> Result<String> {
    bail!("Windows secure storage is required")
}
#[cfg(not(windows))]
fn vault_remove(_: &str) -> Result<()> {
    bail!("Windows secure storage is required")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn public_account_schema_has_no_credential_fields() {
        let profile = Profile {
            id: uuid::Uuid::new_v4().to_string(),
            name: "FixturePlayer".into(),
            active: true,
            owned: true,
        };
        profile.validate().unwrap();
        let value = serde_json::to_value(profile).unwrap();
        assert_eq!(value.as_object().unwrap().len(), 4);
        assert!(value.get("access_token").is_none());
        assert!(value.get("refresh_token").is_none());
        assert_eq!(value.get("owned").unwrap().as_bool(), Some(true));
    }

    #[test]
    fn client_id_is_valid_and_non_empty() {
        let id = client_id().expect("client_id should succeed");
        assert!(!id.is_empty());
        assert!(id.len() <= 100);
        assert!(id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-'));
        if option_env!("SOLARA_MICROSOFT_CLIENT_ID").unwrap_or_default().is_empty() {
            assert_eq!(id, "6a9238bb-3803-46f4-bf38-2f45bdf8c2ea");
        }
    }

    #[test]
    fn pkce_generation_conforms_to_rfc7636() {
        let (verifier, challenge) = generate_pkce();
        assert_eq!(verifier.len(), 43);
        assert_eq!(challenge.len(), 43);
        assert!(verifier.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_'));
        assert!(challenge.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_'));

        let expected = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
        assert_eq!(challenge, expected);
    }

    #[test]
    fn loopback_auth_url_has_all_required_parameters() {
        let id = "6a9238bb-3803-46f4-bf38-2f45bdf8c2ea";
        let port = 54321;
        let challenge = "mock_challenge_43_characters_long_123456789";
        let url_str = loopback_auth_url(id, port, challenge);

        let parsed = reqwest::Url::parse(&url_str).expect("Auth URL must be a valid URL");
        assert_eq!(parsed.scheme(), "https");
        assert_eq!(parsed.host_str(), Some("login.microsoftonline.com"));
        assert_eq!(parsed.path(), "/consumers/oauth2/v2.0/authorize");

        let query: std::collections::HashMap<_, _> = parsed.query_pairs().collect();
        assert_eq!(query.get("client_id").map(|s| s.as_ref()), Some(id));
        assert_eq!(query.get("response_type").map(|s| s.as_ref()), Some("code"));
        assert_eq!(
            query.get("redirect_uri").map(|s| s.as_ref()),
            Some("http://localhost:54321/callback")
        );
        assert_eq!(
            query.get("scope").map(|s| s.as_ref()),
            Some("XboxLive.signin offline_access")
        );
        assert_eq!(query.get("code_challenge").map(|s| s.as_ref()), Some(challenge));
        assert_eq!(query.get("code_challenge_method").map(|s| s.as_ref()), Some("S256"));
        assert_eq!(query.get("prompt").map(|s| s.as_ref()), Some("select_account"));
    }

    #[tokio::test]
    async fn loopback_callback_listener_extracts_code_and_sends_branded_html() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let cancel = Cancellation::default();

        let server = tokio::spawn(async move {
            wait_for_callback(listener, None, port, &cancel).await
        });

        let client = reqwest::Client::new();
        let res = client
            .get(format!("http://127.0.0.1:{port}/callback?code=MOCK_AUTH_CODE_123"))
            .send()
            .await
            .expect("GET request to loopback server must succeed");

        assert_eq!(res.status(), reqwest::StatusCode::OK);
        let body = res.text().await.unwrap();
        assert!(body.contains("Authentication Complete!"));
        assert!(body.contains("You can close this tab and return to Solara Launcher."));

        let code = server.await.unwrap().expect("wait_for_callback should succeed");
        assert_eq!(code, "MOCK_AUTH_CODE_123");
    }

    #[tokio::test]
    async fn loopback_callback_listener_ignores_favicon() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let cancel = Cancellation::default();

        let server = tokio::spawn(async move {
            wait_for_callback(listener, None, port, &cancel).await
        });

        let client = reqwest::Client::new();
        let res_fav = client
            .get(format!("http://127.0.0.1:{port}/favicon.ico"))
            .send()
            .await
            .expect("Favicon request");
        assert_eq!(res_fav.status(), reqwest::StatusCode::NOT_FOUND);

        let res_cb = client
            .get(format!("http://127.0.0.1:{port}/callback?code=REAL_CODE_AFTER_FAVICON"))
            .send()
            .await
            .expect("Callback request");
        assert_eq!(res_cb.status(), reqwest::StatusCode::OK);

        let code = server.await.unwrap().expect("wait_for_callback should succeed");
        assert_eq!(code, "REAL_CODE_AFTER_FAVICON");
    }

    #[tokio::test]
    async fn loopback_callback_handles_chunked_request() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let cancel = Cancellation::default();

        let server = tokio::spawn(async move {
            wait_for_callback(listener, None, port, &cancel).await
        });

        let mut stream = tokio::net::TcpStream::connect(format!("127.0.0.1:{port}"))
            .await
            .expect("Connect TCP");
        stream
            .write_all(b"GET /call")
            .await
            .expect("Write first chunk");
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        stream
            .write_all(b"back?code=CHUNKED_123 HTTP/1.1\r\nHost: localhost\r\n\r\n")
            .await
            .expect("Write second chunk");

        let mut response = Vec::new();
        stream
            .read_to_end(&mut response)
            .await
            .expect("Read response");
        let resp_str = String::from_utf8_lossy(&response);
        assert!(resp_str.starts_with("HTTP/1.1 200 OK"));
        assert!(resp_str.contains("Authentication Complete!"));

        let code = server.await.unwrap().expect("wait_for_callback should succeed");
        assert_eq!(code, "CHUNKED_123");
    }

    #[tokio::test]
    async fn loopback_callback_handles_concurrent_idle_connection() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let cancel = Cancellation::default();

        let server = tokio::spawn(async move {
            wait_for_callback(listener, None, port, &cancel).await
        });

        // First client connects and stays completely idle (e.g. browser speculative pre-connect)
        let _idle = tokio::net::TcpStream::connect(format!("127.0.0.1:{port}"))
            .await
            .expect("Connect idle client");

        // Second client connects immediately and sends callback request
        let start = std::time::Instant::now();
        let client = reqwest::Client::new();
        let res = client
            .get(format!("http://127.0.0.1:{port}/callback?code=CONCURRENT_CODE_OK"))
            .send()
            .await
            .expect("GET request");

        assert_eq!(res.status(), reqwest::StatusCode::OK);
        let elapsed = start.elapsed();
        // Crucial: Idle client must not block the loop with a 5s delay!
        assert!(
            elapsed < std::time::Duration::from_secs(2),
            "Callback must be processed concurrently without stalling on idle connection"
        );

        let code = server.await.unwrap().expect("wait_for_callback should succeed");
        assert_eq!(code, "CONCURRENT_CODE_OK");
    }

    #[tokio::test]
    async fn loopback_callback_handles_error_callback() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let cancel = Cancellation::default();

        let server = tokio::spawn(async move {
            wait_for_callback(listener, None, port, &cancel).await
        });

        let client = reqwest::Client::new();
        let res = client
            .get(format!("http://127.0.0.1:{port}/callback?error=access_denied&error_description=User+declined+permission"))
            .send()
            .await
            .expect("GET request");

        assert_eq!(res.status(), reqwest::StatusCode::OK);
        let body = res.text().await.unwrap();
        assert!(body.contains("Authentication Cancelled"));
        assert!(body.contains("User declined permission"));

        let err = server.await.unwrap().expect_err("Must return Err on error callback");
        assert!(err.to_string().contains("User declined permission"));
    }

    #[tokio::test]
    async fn loopback_callback_accepts_ipv6_when_available() {
        let listener_v4 = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener_v4.local_addr().unwrap().port();
        let listener_v6 = match tokio::net::TcpListener::bind(format!("[::1]:{port}")).await {
            Ok(l) => l,
            Err(_) => return, // IPv6 disabled on host
        };
        let cancel = Cancellation::default();

        let server = tokio::spawn(async move {
            wait_for_callback(listener_v4, Some(listener_v6), port, &cancel).await
        });

        let client = reqwest::Client::new();
        let res = client
            .get(format!("http://[::1]:{port}/callback?code=IPV6_SUCCESS_CODE"))
            .send()
            .await
            .expect("GET request to IPv6 loopback");

        assert_eq!(res.status(), reqwest::StatusCode::OK);
        let code = server.await.unwrap().expect("wait_for_callback should succeed on IPv6");
        assert_eq!(code, "IPV6_SUCCESS_CODE");
    }
}
