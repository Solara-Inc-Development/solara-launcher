use anyhow::{bail, Context, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::Mutex;
const AUTH: &str = "https://auth.solara-inc.in";
const API: &str = "https://api.solara-inc.in";
static SESSION_LOCK: Mutex<()> = Mutex::const_new(());
#[derive(Serialize, Deserialize)]
struct Session {
    access_token: String,
    refresh_token: String,
    expires_at: i64,
}
#[cfg(windows)]
fn vault_read() -> Result<Option<Session>> {
    let e = keyring::Entry::new("in.solara.launcher.solara", "session")
        .map_err(|_| anyhow::anyhow!("Windows credential vault unavailable"))?;
    match e.get_password() {
        Ok(s) => Ok(Some(serde_json::from_str(&s)?)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(_) => bail!("Windows credential vault read failed"),
    }
}
#[cfg(not(windows))]
fn vault_read() -> Result<Option<Session>> {
    Ok(None)
}
#[cfg(windows)]
fn vault_write(s: &Session) -> Result<()> {
    keyring::Entry::new("in.solara.launcher.solara", "session")
        .and_then(|e| e.set_password(&serde_json::to_string(s).unwrap_or_default()))
        .map_err(|_| anyhow::anyhow!("Windows credential vault write failed"))?;
    Ok(())
}
#[cfg(not(windows))]
fn vault_write(_: &Session) -> Result<()> {
    bail!("Secure Solara sessions currently require Windows Credential Manager")
}
#[cfg(windows)]
fn vault_clear() -> Result<()> {
    let e = keyring::Entry::new("in.solara.launcher.solara", "session")
        .map_err(|_| anyhow::anyhow!("Credential vault unavailable"))?;
    match e.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(_) => bail!("Credential vault deletion failed"),
    }
}
#[cfg(not(windows))]
fn vault_clear() -> Result<()> {
    Ok(())
}
fn http() -> Result<reqwest::Client> {
    Ok(reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .redirect(reqwest::redirect::Policy::none())
        .build()?)
}
fn client_id() -> Result<&'static str> {
    let id = option_env!("SOLARA_CLIENT_ID").unwrap_or_default();
    if !id.is_empty() {
        return Ok(id);
    }
    if let Ok(val) = std::env::var("SOLARA_CLIENT_ID") {
        if !val.is_empty() {
            return Ok(Box::leak(val.into_boxed_str()));
        }
    }
    Ok("solara-launcher-desktop")
}
fn random() -> String {
    let mut b = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut b);
    URL_SAFE_NO_PAD.encode(b)
}
async fn token_request(form: &[(&str, &str)]) -> Result<Session> {
    let response = http()?
        .post(format!("{AUTH}/native/token"))
        .form(form)
        .send()
        .await
        .context("Solara SSO token service unavailable")?;
    if !response.status().is_success() {
        bail!("Solara authorization expired, was cancelled or is not configured; sign in again")
    };
    let d: Value = response.json().await?;
    let access = d["access_token"]
        .as_str()
        .context("Invalid Solara token response")?;
    let refresh = d["refresh_token"]
        .as_str()
        .context("Missing rotating Solara refresh token")?;
    let expires = d["expires_in"]
        .as_i64()
        .filter(|n| *n > 0 && *n <= 3600)
        .context("Invalid Solara session expiry")?;
    Ok(Session {
        access_token: access.into(),
        refresh_token: refresh.into(),
        expires_at: chrono::Utc::now().timestamp() + expires,
    })
}
pub async fn sign_in(open: impl FnOnce(String) -> Result<()>) -> Result<Value> {
    let _lock = SESSION_LOCK.lock().await;
    client_id()?;
    if !cfg!(windows) {
        bail!("Native sign-in requires Windows secure credential storage")
    };
    let listener = match tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 54321)).await {
        Ok(l) => l,
        Err(_) => tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)).await?,
    };
    let port = listener.local_addr()?.port();
    let redirect = format!("http://127.0.0.1:{port}/solara/callback");
    let verifier = random();
    let state = random();
    let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
    let mut url = reqwest::Url::parse(&format!("{AUTH}/native/authorize"))?;
    url.query_pairs_mut().extend_pairs([
        ("client_id", client_id()?),
        ("redirect_uri", redirect.as_str()),
        ("response_type", "code"),
        ("scope", "profile services"),
        ("state", state.as_str()),
        ("code_challenge", challenge.as_str()),
        ("code_challenge_method", "S256"),
    ]);
    open(url.into())?;
    let code = tokio::time::timeout(std::time::Duration::from_secs(180), async {
        loop {
            let (mut socket, peer) = match listener.accept().await {
                Ok(conn) => conn,
                Err(_) => continue,
            };
            if !peer.ip().is_loopback() {
                continue;
            }
            let mut buf = vec![0; 8192];
            let count = match tokio::time::timeout(std::time::Duration::from_secs(5), socket.read(&mut buf)).await {
                Ok(Ok(c)) if c > 0 => c,
                _ => continue,
            };
            let raw = String::from_utf8_lossy(&buf[..count]);
            let line = raw.lines().next().unwrap_or_default();
            let mut parts = line.split_whitespace();
            let method = parts.next().unwrap_or_default();
            let path = parts.next().unwrap_or_default();
            if path.is_empty() || path == "/favicon.ico" {
                let _ = socket.write_all(b"HTTP/1.1 404 Not Found\r\nConnection: close\r\n\r\n").await;
                continue;
            }
            let callback = match reqwest::Url::parse(&format!("http://127.0.0.1:{port}{path}")) {
                Ok(u) => u,
                Err(_) => {
                    let _ = socket.write_all(b"HTTP/1.1 400 Bad Request\r\nConnection: close\r\n\r\nInvalid URL").await;
                    continue;
                }
            };
            let params = callback.query_pairs().collect::<std::collections::HashMap<_, _>>();
            let valid = method == "GET"
                && callback.path() == "/solara/callback"
                && params.get("state").map(|s| s.as_ref()) == Some(state.as_str());
            let html_response = if valid {
                "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n<!DOCTYPE html><html><head><meta charset='utf-8'><title>Sign-in Complete</title><style>body{background:#09090b;color:#f4f4f5;font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif;display:flex;align-items:center;justify-content:center;height:100vh;margin:0}.box{background:#18181b;border:1px solid #27272a;border-radius:12px;padding:32px;text-align:center;max-width:380px}h2{color:#38bdf8;margin:0 0 8px 0}p{color:#a1a1aa;font-size:14px;margin:0}</style><script>setTimeout(function(){try{window.close();}catch(e){}},1000);</script></head><body><div class='box'><h2>Signed in to Solara</h2><p>You can close this tab and return to the launcher.</p></div></body></html>"
            } else {
                "HTTP/1.1 400 Bad Request\r\nContent-Type: text/plain\r\nConnection: close\r\n\r\nInvalid callback request"
            };
            let _ = socket.write_all(html_response.as_bytes()).await;
            let _ = socket.flush().await;
            if !valid {
                continue;
            }
            if params.contains_key("error") {
                bail!("Solara sign-in was cancelled")
            }
            return Ok::<String, anyhow::Error>(
                params
                    .get("code")
                    .context("Missing authorization code")?
                    .to_string(),
            );
        }
    })
    .await
    .context("Solara sign-in timed out")??;
    let session = token_request(&[
        ("grant_type", "authorization_code"),
        ("client_id", client_id()?),
        ("redirect_uri", &redirect),
        ("code_verifier", &verifier),
        ("code", &code),
    ])
    .await?;
    vault_write(&session)?;
    drop(_lock);
    profile()
        .await?
        .context("Profile unavailable after sign-in")
}
async fn access() -> Result<String> {
    let _guard = SESSION_LOCK.lock().await;
    let mut s = vault_read()?.context("Sign in to Solara to use this service")?;
    if s.expires_at < chrono::Utc::now().timestamp() + 60 {
        match token_request(&[
            ("grant_type", "refresh_token"),
            ("client_id", client_id()?),
            ("refresh_token", &s.refresh_token),
        ])
        .await
        {
            Ok(new) => {
                s = new;
                vault_write(&s)?
            }
            Err(e) => {
                vault_clear()?;
                return Err(e);
            }
        }
    };
    Ok(s.access_token)
}
pub async fn profile() -> Result<Option<Value>> {
    if vault_read()?.is_none() {
        return Ok(None);
    };
    match request("GET", "/v1/launcher/me", None).await {
        Ok(p) => Ok(Some(p)),
        Err(e) => {
            if vault_read()?.is_none() {
                return Ok(None);
            };
            Err(e)
        }
    }
}
pub async fn request(method: &str, path: &str, body: Option<Value>) -> Result<Value> {
    if !crate::resource_policy::allowed(method, path) {
        bail!("Unsupported Solara resource path")
    };
    let token = access().await?;
    let req = http()?
        .request(
            reqwest::Method::from_bytes(method.as_bytes())?,
            format!("{API}{path}"),
        )
        .bearer_auth(token);
    let response = if let Some(body) = body {
        req.json(&body)
    } else {
        req
    }
    .send()
    .await
    .context("Main Solara API unavailable")?;
    let status = response.status();
    if status.as_u16() == 401 {
        vault_clear()?;
        bail!("Solara session expired or was revoked; sign in again")
    };
    let mut response = response;
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await? {
        if bytes.len() + chunk.len() > 2_000_000 {
            bail!("Solara API response too large")
        }
        bytes.extend_from_slice(&chunk);
    }
    if !status.is_success() {
        let body: Value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
        let message = match body["error"]["code"].as_str().unwrap_or_default() {
            "endpoint_verification_required" => "Verify control of this public endpoint before requesting or updating its address. Contact support if verification is not available.",
            "dns_provider_required" | "dns_provider_invalid" | "dns_namespace_required" => "Public hosting is not configured yet. Contact Solara support.",
            "assignment_not_found" => "This address is unavailable or no longer belongs to your account.",
            "assignment_quota" => "Your account has reached its public address limit.",
            "entitlement_required" => "Public hosting access is unavailable, expired or revoked for this account.",
            "dns_update_failed" | "dns_provision_failed" => "The DNS provider could not complete the change. Refresh your addresses and contact support before retrying.",
            "rate_limited" => "Too many requests. Wait before trying again.",
            "invalid_name" => "Choose an unreserved name of 3–32 lowercase letters, digits or inner hyphens.",
            "invalid_endpoint" => "Enter a public IP address you control.",
            "invalid_port" => "Choose a port from 1024 to 65535.",
            "consent_required" => "Confirm endpoint control and public IP exposure.",
            _ => return Err(anyhow::anyhow!("Solara API request rejected ({status}); verify permissions and service configuration")),
        };
        bail!("{message}")
    }
    Ok(serde_json::from_slice(&bytes)?)
}

pub async fn sign_out() -> Result<()> {
    let _guard = SESSION_LOCK.lock().await;
    if let Some(s) = vault_read()? {
        let result = http()?
            .post(format!("{AUTH}/native/revoke"))
            .form(&[
                ("token", s.refresh_token),
                ("client_id", client_id()?.to_string()),
            ])
            .send()
            .await;
        vault_clear()?;
        if !matches!(result,Ok(r)if r.status().is_success()) {
            bail!("Local Solara credentials removed; server-side revocation could not be confirmed. Revoke the session from your central account.")
        }
    };
    Ok(())
}
#[cfg(test)]
mod tests {
    use crate::resource_policy::allowed;
    #[test]
    fn protects_destination_and_routes() {
        assert!(allowed("GET", "/v1/launcher/me"));
        assert!(allowed("POST", "/v1/launcher/servers/abc123/power"));
        for p in [
            "https://evil.example/",
            "/v1/launcher/../secrets",
            "/v1/launcher/servers/../../power",
            "/v1/launcher/me?token=x",
            "/v1/launcher/me#x",
        ] {
            assert!(!allowed("GET", p))
        }
        assert!(!allowed("DELETE", "/v1/launcher/me"));
    }
}
