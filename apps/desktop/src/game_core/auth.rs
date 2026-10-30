//! Independent Microsoft -> Xbox -> Minecraft flow. Never contacts Solara.
use super::{download::Cancellation, instances::write_json};
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::PathBuf;
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
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DevicePrompt {
    pub code: String,
    pub url: String,
    pub expires_in: u64,
}

fn client_id() -> Result<&'static str> {
    let value = option_env!("SOLARA_MICROSOFT_CLIENT_ID").unwrap_or_default();
    if value.is_empty()
        || value.len() > 100
        || !value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-')
    {
        bail!("Microsoft sign-in is not configured. An approved Solara Microsoft application is required.")
    }
    Ok(value)
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
}
