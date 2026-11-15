use crate::ErrorKind;
use crate::util::fetch::INSECURE_REQWEST_CLIENT;
use anyhow::Context;
use base64::Engine;
use base64::prelude::BASE64_URL_SAFE_NO_PAD;
use chrono::{DateTime, Duration, TimeZone, Utc};
use dashmap::DashMap;
use futures::TryStreamExt;
use heck::ToTitleCase;
use rand::RngExt;
use reqwest::header::HeaderMap;
use reqwest::{Response, StatusCode};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::json;
use sha2::Digest;
use std::borrow::Cow;
use std::collections::HashMap;
use std::error::Error;
use std::future::Future;
use std::hash::{BuildHasherDefault, DefaultHasher};
use std::ops::Deref;
use std::path::Path;
use std::sync::{Arc, LazyLock};
use std::time::Instant;
use std::{io, iter};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::runtime::{Handle, RuntimeFlavor};
use tokio::sync::Mutex;
use tokio::task;
use url::Url;
use uuid::Uuid;

pub const DEFAULT_SOLARA_MICROSOFT_CLIENT_ID: &str =
    "6a9238bb-3803-46f4-bf38-2f45bdf8c2ea";
pub const KEYRING_SERVICE: &str = "in.solara.launcher.minecraft.custom";
pub const MINECRAFT_SERVICES_USER_AGENT: &str =
    "Solara Launcher (https://app.solara-inc.in)";
const MICROSOFT_OAUTH_SCOPE: &str = "XboxLive.signin offline_access";

static MEMORY_TOKEN_CACHE: LazyLock<DashMap<String, String>> =
    LazyLock::new(DashMap::new);

pub fn solara_microsoft_client_id() -> String {
    if let Ok(val) = std::env::var("SOLARA_MICROSOFT_CLIENT_ID") {
        let trimmed = val.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }
    if let Some(val) = option_env!("SOLARA_MICROSOFT_CLIENT_ID") {
        let trimmed = val.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }
    DEFAULT_SOLARA_MICROSOFT_CLIENT_ID.to_string()
}

pub fn store_keyring_secret(key: &str, secret: &str) {
    let trimmed = secret.trim();
    if trimmed.is_empty() {
        return;
    }
    MEMORY_TOKEN_CACHE.insert(key.to_string(), trimmed.to_string());
    if let Ok(entry) = keyring::Entry::new(KEYRING_SERVICE, key) {
        let _ = entry.set_password(trimmed);
    }
}

pub fn read_keyring_secret(key: &str) -> Option<String> {
    if let Some(cached) = MEMORY_TOKEN_CACHE.get(key)
        && !cached.value().trim().is_empty()
    {
        return Some(cached.value().clone());
    }
    if let Ok(entry) = keyring::Entry::new(KEYRING_SERVICE, key)
        && let Ok(secret) = entry.get_password()
    {
        let trimmed = secret.trim().to_string();
        if !trimmed.is_empty() {
            MEMORY_TOKEN_CACHE.insert(key.to_string(), trimmed.clone());
            return Some(trimmed);
        }
    }
    None
}

pub fn delete_keyring_secret(key: &str) {
    MEMORY_TOKEN_CACHE.remove(key);
    if let Ok(entry) = keyring::Entry::new(KEYRING_SERVICE, key) {
        let _ = entry.delete_credential();
    }
}

fn access_token_keyring_key(uuid: &Uuid) -> String {
    format!("access:{}", uuid.as_hyphenated())
}

fn refresh_token_keyring_key(uuid: &Uuid) -> String {
    uuid.as_hyphenated().to_string()
}

#[derive(Debug, Clone, Copy)]
pub enum MinecraftAuthStep {
    GetDeviceToken,
    SisuAuthenticate,
    GetOAuthToken,
    RefreshOAuthToken,
    SisuAuthorize,
    XstsAuthorize,
    MinecraftToken,
    MinecraftEntitlements,
    MinecraftProfile,
}

#[derive(thiserror::Error, Debug)]
pub enum MinecraftAuthenticationError {
    #[error("Error reading public key during generation")]
    ReadingPublicKey,
    #[error("Failed to serialize body to JSON during step {step:?}: {source}")]
    SerializeBody {
        step: MinecraftAuthStep,
        #[source]
        source: serde_json::Error,
    },
    #[error(
        "Failed to deserialize response to JSON during step {step:?}: {source}. Status Code: {status_code} Body: {raw}"
    )]
    DeserializeResponse {
        step: MinecraftAuthStep,
        raw: String,
        #[source]
        source: serde_json::Error,
        status_code: StatusCode,
    },
    #[error("Request failed during step {step:?}: {source}")]
    Request {
        step: MinecraftAuthStep,
        #[source]
        source: reqwest::Error,
    },
    #[error("Error reading XBOX Session ID header")]
    NoSessionId,
    #[error("Error reading user hash")]
    NoUserHash,
    #[error("Microsoft sign-in failed: {0}")]
    OAuthLoopback(String),
}

#[derive(Deserialize)]
struct OAuthErrorResponse {
    error: String,
}

#[derive(Deserialize)]
struct XboxErrorResponse {
    #[serde(rename = "XErr")]
    xerr: u64,
}

impl MinecraftAuthenticationError {
    pub(crate) fn code(&self) -> &'static str {
        match self {
            Self::DeserializeResponse {
                step: MinecraftAuthStep::RefreshOAuthToken,
                ..
            } => "minecraft_auth_token_expired",
            Self::DeserializeResponse {
                step: MinecraftAuthStep::SisuAuthenticate,
                ..
            } => "minecraft_auth_clock_out_of_sync",
            Self::DeserializeResponse {
                step: MinecraftAuthStep::MinecraftToken,
                status_code: StatusCode::TOO_MANY_REQUESTS,
                ..
            } => "minecraft_auth_rate_limited",
            Self::DeserializeResponse {
                step: MinecraftAuthStep::MinecraftToken,
                status_code,
                ..
            } if status_code.is_server_error() => {
                "minecraft_auth_service_unavailable"
            }
            Self::Request { source, .. } if source.is_request() => {
                let connection_refused = iter::successors(
                    Some(source as &(dyn Error + 'static)),
                    |&error| error.source(),
                )
                .any(|error| {
                    error.downcast_ref::<io::Error>().is_some_and(|error| {
                        error.kind() == io::ErrorKind::ConnectionRefused
                    })
                });

                if connection_refused {
                    "minecraft_auth_connection_refused"
                } else {
                    "minecraft_auth_network"
                }
            }
            Self::DeserializeResponse { step, raw, .. } => {
                match serde_json::from_str::<XboxErrorResponse>(raw)
                    .map(|response| response.xerr)
                {
                    Ok(2148916222) => "minecraft_auth_age_verification",
                    Ok(2148916227) => "minecraft_auth_banned",
                    Ok(2148916229) => "minecraft_auth_restricted",
                    Ok(2148916233) => "minecraft_auth_no_xbox_profile",
                    Ok(2148916234) => "minecraft_auth_terms_not_accepted",
                    Ok(2148916235) => "minecraft_auth_region_unavailable",
                    Ok(2148916236 | 2148916237) => {
                        "minecraft_auth_adult_verification"
                    }
                    Ok(2148916238) => "minecraft_auth_underage",
                    _ if matches!(step, MinecraftAuthStep::XstsAuthorize) => {
                        "minecraft_auth_xsts_rejected"
                    }
                    _ => "minecraft_auth_error",
                }
            }
            _ => "minecraft_auth_error",
        }
    }

    fn is_invalid_grant(&self) -> bool {
        matches!(
            self,
            Self::DeserializeResponse {
                step: MinecraftAuthStep::RefreshOAuthToken,
                raw,
                status_code: StatusCode::BAD_REQUEST,
                ..
            } if serde_json::from_str::<OAuthErrorResponse>(raw)
                .is_ok_and(|response| response.error == "invalid_grant")
        )
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MinecraftLoginFlow {
    pub verifier: String,
    pub challenge: String,
    pub session_id: String,
    pub auth_request_uri: String,
    #[serde(default)]
    pub redirect_uri: String,
}

pub fn generate_pkce_pair() -> (String, String, String) {
    let mut rng = rand::rng();
    let verifier_bytes: [u8; 32] = rng.random();
    let state_bytes: [u8; 16] = rng.random();
    let verifier = BASE64_URL_SAFE_NO_PAD.encode(verifier_bytes);
    let challenge =
        BASE64_URL_SAFE_NO_PAD.encode(sha2::Sha256::digest(verifier.as_bytes()));
    let state = BASE64_URL_SAFE_NO_PAD.encode(state_bytes);
    (verifier, challenge, state)
}

pub fn build_authorize_url(
    client_id: &str,
    redirect_uri: &str,
    state: &str,
    challenge: &str,
) -> crate::Result<String> {
    let mut url = Url::parse(
        "https://login.microsoftonline.com/consumers/oauth2/v2.0/authorize",
    )?;
    url.query_pairs_mut()
        .append_pair("client_id", client_id)
        .append_pair("response_type", "code")
        .append_pair("redirect_uri", redirect_uri)
        .append_pair("response_mode", "query")
        .append_pair("scope", MICROSOFT_OAUTH_SCOPE)
        .append_pair("state", state)
        .append_pair("code_challenge", challenge)
        .append_pair("code_challenge_method", "S256")
        .append_pair("prompt", "select_account");
    Ok(url.to_string())
}

#[tracing::instrument(skip(_exec))]
pub async fn login_begin(
    _exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite> + Copy,
) -> crate::Result<MinecraftLoginFlow> {
    let (verifier, challenge, state) = generate_pkce_pair();
    let redirect_uri = "https://login.microsoftonline.com/common/oauth2/nativeclient".to_string();
    let client_id = solara_microsoft_client_id();
    let auth_request_uri =
        build_authorize_url(&client_id, &redirect_uri, &state, &challenge)?;

    Ok(MinecraftLoginFlow {
        verifier,
        challenge,
        session_id: state,
        auth_request_uri,
        redirect_uri,
    })
}

#[tracing::instrument(skip(exec))]
pub async fn login_finish(
    code: &str,
    flow: MinecraftLoginFlow,
    exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite> + Copy,
) -> crate::Result<Credentials> {
    let redirect_uri = if flow.redirect_uri.is_empty() {
        "https://login.microsoftonline.com/common/oauth2/nativeclient"
    } else {
        flow.redirect_uri.as_str()
    };
    let oauth_token = oauth_token(code, &flow.verifier, redirect_uri).await?;
    complete_login_from_oauth(oauth_token, exec).await
}

/// Performs Solara's loopback PKCE Microsoft OAuth 2.0 login flow (`127.0.0.1:0` + `[::1]:port`),
/// opening the system browser via `open_url` and completing Xbox Live + XSTS + Mojang authentication.
pub async fn login_with_loopback<F>(
    open_url: F,
    exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite> + Copy,
) -> crate::Result<Credentials>
where
    F: FnOnce(&str),
{
    let listener_v4 = TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|e| MinecraftAuthenticationError::OAuthLoopback(e.to_string()))?;
    let port = listener_v4
        .local_addr()
        .map_err(|e| MinecraftAuthenticationError::OAuthLoopback(e.to_string()))?
        .port();
    let listener_v6 = TcpListener::bind(format!("[::1]:{port}")).await.ok();
    let redirect_uri = format!("http://localhost:{port}/callback");
    let (verifier, challenge, state) = generate_pkce_pair();
    let client_id = solara_microsoft_client_id();
    let authorize_url =
        build_authorize_url(&client_id, &redirect_uri, &state, &challenge)?;

    open_url(&authorize_url);

    let code = tokio::time::timeout(
        std::time::Duration::from_secs(180),
        wait_for_loopback_code(listener_v4, listener_v6, &state),
    )
    .await
    .map_err(|_| {
        MinecraftAuthenticationError::OAuthLoopback(
            "Timed out waiting for Microsoft sign-in in your browser".into(),
        )
    })??;

    let oauth_token = oauth_token(&code, &verifier, &redirect_uri).await?;
    complete_login_from_oauth(oauth_token, exec).await
}

async fn complete_login_from_oauth(
    oauth_token: RequestWithDate<OAuthToken>,
    exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite> + Copy,
) -> crate::Result<Credentials> {
    let xbl = xbox_user_authenticate(&oauth_token.value.access_token).await?;
    let xsts = xsts_authorize(&xbl.token).await?;
    let minecraft_token = minecraft_token(xsts).await?;

    minecraft_entitlements(&minecraft_token.access_token).await?;

    let refresh_token = oauth_token
        .value
        .refresh_token
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_default();

    let mut credentials = Credentials {
        offline_profile: MinecraftProfile::default(),
        access_token: minecraft_token.access_token,
        refresh_token,
        expires: oauth_token.date
            + Duration::seconds(oauth_token.value.expires_in as i64),
        active: true,
    };

    let online_profile = credentials
        .online_profile()
        .await
        .ok_or(ErrorKind::NoMinecraftProfile)?;
    credentials.offline_profile = MinecraftProfile {
        id: online_profile.id,
        name: online_profile.name.clone(),
        ..credentials.offline_profile
    };

    credentials.upsert(exec).await?;

    Ok(credentials)
}

async fn wait_for_loopback_code(
    listener_v4: TcpListener,
    listener_v6: Option<TcpListener>,
    expected_state: &str,
) -> Result<String, MinecraftAuthenticationError> {
    loop {
        let mut stream = select_loopback_stream(&listener_v4, &listener_v6)
            .await
            .map_err(|e| {
                MinecraftAuthenticationError::OAuthLoopback(e.to_string())
            })?;
        let mut buf = vec![0u8; 8192];
        let read = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            stream.read(&mut buf),
        )
        .await;
        let Ok(Ok(n)) = read else {
            continue;
        };
        if n == 0 {
            continue;
        }
        let req = String::from_utf8_lossy(&buf[..n]);
        let Some(first_line) = req.lines().next() else {
            continue;
        };
        let mut parts = first_line.split_whitespace();
        let method = parts.next().unwrap_or("");
        let target = parts.next().unwrap_or("");

        if method != "GET" || !target.starts_with("/callback") {
            let _ = write_loopback_html(
                &mut stream,
                "404 Not Found",
                "Page Not Found",
                "You can close this tab and return to Solara Launcher.",
                false,
            )
            .await;
            continue;
        }

        let parsed = Url::parse(&format!("http://localhost{target}")).map_err(
            |e| MinecraftAuthenticationError::OAuthLoopback(e.to_string()),
        )?;
        let params: HashMap<String, String> =
            parsed.query_pairs().into_owned().collect();

        if let Some(error) = params.get("error") {
            let desc = params
                .get("error_description")
                .map(|s| s.as_str())
                .unwrap_or(error.as_str());
            let _ = write_loopback_html(
                &mut stream,
                "400 Bad Request",
                "Sign-In Not Completed",
                "Microsoft sign-in was cancelled or rejected. Return to Solara Launcher to try again.",
                false,
            )
            .await;
            return Err(MinecraftAuthenticationError::OAuthLoopback(
                desc.to_string(),
            ));
        }

        if params.get("state").map(String::as_str) != Some(expected_state) {
            let _ = write_loopback_html(
                &mut stream,
                "400 Bad Request",
                "Security Check Failed",
                "The sign-in state token did not match. Return to Solara Launcher and try again.",
                false,
            )
            .await;
            return Err(MinecraftAuthenticationError::OAuthLoopback(
                "Microsoft OAuth state mismatch".into(),
            ));
        }

        let Some(code) = params
            .get("code")
            .filter(|c| !c.trim().is_empty())
            .cloned()
        else {
            let _ = write_loopback_html(
                &mut stream,
                "400 Bad Request",
                "Missing Authorization Code",
                "Microsoft did not return an authorization code. Return to Solara Launcher and try again.",
                false,
            )
            .await;
            return Err(MinecraftAuthenticationError::OAuthLoopback(
                "Missing authorization code".into(),
            ));
        };

        let _ = write_loopback_html(
            &mut stream,
            "200 OK",
            "Microsoft Account Connected",
            "Your Minecraft Java profile is now linked. You can close this window and return to Solara Launcher.",
            true,
        )
        .await;
        return Ok(code);
    }
}

async fn select_loopback_stream(
    listener_v4: &TcpListener,
    listener_v6: &Option<TcpListener>,
) -> io::Result<TcpStream> {
    if let Some(v6) = listener_v6 {
        tokio::select! {
            res = listener_v4.accept() => Ok(res?.0),
            res = v6.accept() => Ok(res?.0),
        }
    } else {
        Ok(listener_v4.accept().await?.0)
    }
}

async fn write_loopback_html(
    stream: &mut TcpStream,
    status: &str,
    heading: &str,
    message: &str,
    ok: bool,
) -> io::Result<()> {
    let badge_bg = if ok {
        "rgba(59,130,246,0.16)"
    } else {
        "rgba(248,113,113,0.16)"
    };
    let badge_fg = if ok { "#60a5fa" } else { "#f87171" };
    let badge_label = if ok {
        "Authentication Complete"
    } else {
        "Authentication Notice"
    };
    let body = format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\">\
<meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">\
<title>{heading} - Solara Launcher</title>\
<style>\
*{{box-sizing:border-box;margin:0;padding:0}}\
body{{min-height:100vh;display:flex;align-items:center;justify-content:center;padding:24px;\
background:radial-gradient(circle at 50% 0%,#132038 0%,#0b0f17 68%);color:#f8fafc;\
font-family:'Segoe UI',system-ui,-apple-system,sans-serif}}\
.card{{width:100%;max-width:440px;padding:32px 28px;border-radius:16px;\
background:rgba(17,24,39,0.92);border:1px solid rgba(148,163,184,0.16);\
box-shadow:0 24px 56px rgba(0,0,0,0.52);text-align:center}}\
.brand{{font-size:12px;font-weight:700;letter-spacing:0.16em;text-transform:uppercase;color:#93c5fd;margin-bottom:14px}}\
.badge{{display:inline-block;padding:6px 12px;border-radius:999px;font-size:12px;font-weight:600;\
background:{badge_bg};color:{badge_fg};margin-bottom:16px}}\
h1{{font-size:22px;font-weight:700;line-height:1.3;margin-bottom:10px;color:#ffffff}}\
p{{font-size:14px;line-height:1.6;color:#cbd5e1;margin-bottom:20px}}\
.hint{{font-size:12px;color:#94a3b8;padding-top:14px;border-top:1px solid rgba(148,163,184,0.12)}}\
</style></head><body>\
<main class=\"card\">\
<div class=\"brand\">Solara Launcher</div>\
<div class=\"badge\">{badge_label}</div>\
<h1>{heading}</h1>\
<p>{message}</p>\
<div class=\"hint\">Return to Solara Launcher</div>\
</main></body></html>"
    );
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: text/html; charset=utf-8\r\nCache-Control: no-store\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    stream.write_all(response.as_bytes()).await?;
    stream.flush().await
}

#[derive(Deserialize, Debug, Clone)]
pub struct Credentials {
    /// The offline profile of the user these credentials are for.
    #[serde(rename = "profile")]
    pub offline_profile: MinecraftProfile,
    #[serde(default)]
    pub access_token: String,
    #[serde(default)]
    pub refresh_token: String,
    pub expires: DateTime<Utc>,
    pub active: bool,
}

/// An entry in the player profile cache, keyed by player UUID.
pub(super) enum ProfileCacheEntry {
    Hit(Arc<MinecraftProfile>),
    AuthErrorBackoff {
        likely_expired_token: String,
        last_attempt: Instant,
    },
}

pub(super) static PROFILE_CACHE: Mutex<
    HashMap<Uuid, ProfileCacheEntry, BuildHasherDefault<DefaultHasher>>,
> = Mutex::const_new(HashMap::with_hasher(BuildHasherDefault::new()));

const ONLINE_PROFILE_CACHE_MAX_AGE: std::time::Duration =
    std::time::Duration::from_secs(60);
const ONLINE_PROFILE_LIVE_STATE_MAX_AGE: std::time::Duration =
    std::time::Duration::from_secs(5);
const ONLINE_PROFILE_AUTH_ERROR_BACKOFF: std::time::Duration =
    std::time::Duration::from_secs(60);

#[derive(Debug, Clone, Copy)]
enum OnlineProfileCacheIntent {
    NormalRead,
    LiveStateRead,
    RefreshFromMojang,
}

impl OnlineProfileCacheIntent {
    fn max_age(self) -> std::time::Duration {
        match self {
            Self::NormalRead => ONLINE_PROFILE_CACHE_MAX_AGE,
            Self::LiveStateRead => ONLINE_PROFILE_LIVE_STATE_MAX_AGE,
            Self::RefreshFromMojang => std::time::Duration::ZERO,
        }
    }

    fn can_use_stale_on_fetch_error(self) -> bool {
        matches!(self, Self::LiveStateRead)
    }
}

impl Credentials {
    fn hydrate_tokens_from_keyring(&mut self) {
        let uuid = self.offline_profile.id;
        let access_key = access_token_keyring_key(&uuid);
        let refresh_key = refresh_token_keyring_key(&uuid);

        if let Some(secret) = read_keyring_secret(&access_key) {
            self.access_token = secret;
        } else if !self.access_token.trim().is_empty() {
            store_keyring_secret(&access_key, &self.access_token);
        }

        if let Some(secret) = read_keyring_secret(&refresh_key) {
            self.refresh_token = secret;
        } else if !self.refresh_token.trim().is_empty() {
            store_keyring_secret(&refresh_key, &self.refresh_token);
        }
    }

    /// Refreshes the authentication tokens for this user if they are expired, or
    /// very close to expiration.
    async fn refresh(
        &mut self,
        exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite> + Copy,
    ) -> crate::Result<()> {
        self.hydrate_tokens_from_keyring();

        if !self.access_token.is_empty()
            && self.expires > Utc::now() + Duration::minutes(5)
        {
            return Ok(());
        }

        if self.refresh_token.trim().is_empty() {
            return Err(MinecraftAuthenticationError::OAuthLoopback(
                "No refresh token available in Windows Credential Manager"
                    .into(),
            )
            .into());
        }

        let oauth_token = oauth_refresh(&self.refresh_token).await?;
        let xbl =
            xbox_user_authenticate(&oauth_token.value.access_token).await?;
        let xsts = xsts_authorize(&xbl.token).await?;
        let minecraft_token = minecraft_token(xsts).await?;

        self.access_token = minecraft_token.access_token;
        if let Some(new_refresh) = oauth_token
            .value
            .refresh_token
            .filter(|s| !s.trim().is_empty())
        {
            self.refresh_token = new_refresh;
        }
        self.expires = oauth_token.date
            + Duration::seconds(oauth_token.value.expires_in as i64);

        self.upsert(exec).await?;

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn online_profile(&self) -> Option<Arc<MinecraftProfile>> {
        self.online_profile_with_cache_intent(
            OnlineProfileCacheIntent::NormalRead,
        )
        .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn online_profile_fresh(&self) -> Option<Arc<MinecraftProfile>> {
        self.online_profile_with_cache_intent(
            OnlineProfileCacheIntent::LiveStateRead,
        )
        .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn refresh_online_profile(
        &self,
    ) -> Option<Arc<MinecraftProfile>> {
        self.online_profile_with_cache_intent(
            OnlineProfileCacheIntent::RefreshFromMojang,
        )
        .await
    }

    async fn online_profile_with_cache_intent(
        &self,
        cache_intent: OnlineProfileCacheIntent,
    ) -> Option<Arc<MinecraftProfile>> {
        if self.access_token.trim().is_empty() {
            return None;
        }

        let max_age = cache_intent.max_age();
        let stale_profile = {
            let mut profile_cache = PROFILE_CACHE.lock().await;
            let mut remove_cached_entry = false;

            let stale_profile = if let Some(cache_entry) =
                profile_cache.get(&self.offline_profile.id)
            {
                match cache_entry {
                    ProfileCacheEntry::Hit(profile)
                        if profile.is_fresh(max_age) =>
                    {
                        return Some(Arc::clone(profile));
                    }
                    ProfileCacheEntry::Hit(profile) => {
                        Some(Arc::clone(profile))
                    }
                    ProfileCacheEntry::AuthErrorBackoff {
                        likely_expired_token,
                        last_attempt,
                    } if &self.access_token != likely_expired_token
                        || Instant::now()
                            .saturating_duration_since(*last_attempt)
                            > ONLINE_PROFILE_AUTH_ERROR_BACKOFF =>
                    {
                        remove_cached_entry = true;
                        None
                    }
                    ProfileCacheEntry::AuthErrorBackoff { .. } => {
                        return None;
                    }
                }
            } else {
                None
            };

            if remove_cached_entry {
                profile_cache.remove(&self.offline_profile.id);
            }

            stale_profile
        };

        match minecraft_profile(&self.access_token).await {
            Ok(profile) => {
                let profile = Arc::new(profile);
                let cache_entry = ProfileCacheEntry::Hit(Arc::clone(&profile));

                let mut profile_cache = PROFILE_CACHE.lock().await;
                if self.offline_profile.id != profile.id {
                    profile_cache.remove(&self.offline_profile.id);
                }
                profile_cache.insert(profile.id, cache_entry);

                Some(profile)
            }
            Err(
                err @ MinecraftAuthenticationError::DeserializeResponse {
                    status_code: StatusCode::UNAUTHORIZED,
                    ..
                },
            ) => {
                tracing::warn!(
                    "Failed to fetch online profile for UUID {} likely due to stale credentials, backing off: {err}",
                    self.offline_profile.id
                );

                let mut profile_cache = PROFILE_CACHE.lock().await;
                profile_cache.insert(
                    self.offline_profile.id,
                    ProfileCacheEntry::AuthErrorBackoff {
                        likely_expired_token: self.access_token.clone(),
                        last_attempt: Instant::now(),
                    },
                );

                None
            }
            Err(err) => {
                tracing::warn!(
                    "Failed to fetch online profile for UUID {}: {err}",
                    self.offline_profile.id
                );

                if cache_intent.can_use_stale_on_fetch_error() {
                    stale_profile
                } else {
                    None
                }
            }
        }
    }

    pub async fn maybe_online_profile(
        &self,
    ) -> MaybeOnlineMinecraftProfile<'_> {
        let online_profile = self.online_profile().await;
        online_profile.map_or_else(
            || MaybeOnlineMinecraftProfile::Offline(&self.offline_profile),
            MaybeOnlineMinecraftProfile::Online,
        )
    }

    #[tracing::instrument(skip(exec))]
    pub async fn get_default_credential(
        exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite> + Copy,
    ) -> crate::Result<Option<Credentials>> {
        let credentials = Self::get_active(exec).await?;

        if let Some(mut creds) = credentials {
            let res = creds.refresh(exec).await;

            match res {
                Ok(_) => Ok(Some(creds)),
                Err(err) => {
                    let auth_error =
                        err.downcast_ref::<MinecraftAuthenticationError>();

                    if let Some(MinecraftAuthenticationError::Request {
                        source,
                        ..
                    }) = auth_error
                        && (source.is_connect() || source.is_timeout())
                    {
                        return Ok(Some(creds));
                    }

                    if auth_error.is_some_and(|error| error.is_invalid_grant())
                    {
                        Self::remove(creds.offline_profile.id, exec).await?;

                        if let Some((_, mut user)) =
                            Self::get_all(exec).await?.into_iter().next()
                        {
                            user.active = true;
                            user.upsert(exec).await?;
                        }

                        return Ok(None);
                    }

                    // If offline or refresh token not yet populated in test/offline mode, return existing profile
                    if !creds.offline_profile.name.is_empty() {
                        return Ok(Some(creds));
                    }

                    Err(err)
                }
            }
        } else {
            Ok(None)
        }
    }

    pub async fn get_active(
        exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite> + Copy,
    ) -> crate::Result<Option<Self>> {
        let res = sqlx::query!(
            "
            SELECT
                uuid, active, username, access_token, refresh_token, expires
            FROM minecraft_users
            WHERE active = TRUE
            "
        )
        .fetch_optional(exec)
        .await?;

        Ok(match res {
            Some(x) => {
                let mut credentials = Self {
                    offline_profile: MinecraftProfile {
                        id: Uuid::parse_str(&x.uuid).unwrap_or_default(),
                        name: x.username,
                        ..MinecraftProfile::default()
                    },
                    access_token: x.access_token,
                    refresh_token: x.refresh_token,
                    expires: Utc
                        .timestamp_opt(x.expires, 0)
                        .single()
                        .unwrap_or_else(Utc::now),
                    active: x.active == 1,
                };
                credentials.hydrate_tokens_from_keyring();
                credentials.refresh(exec).await.ok();
                Some(credentials)
            }
            None => None,
        })
    }

    pub async fn get_all(
        exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite> + Copy,
    ) -> crate::Result<DashMap<Uuid, Self>> {
        let res = sqlx::query!(
            "
            SELECT
                uuid, active, username, access_token, refresh_token, expires
            FROM minecraft_users
            "
        )
        .fetch(exec)
        .try_fold(DashMap::new(), |acc, x| {
            let uuid = Uuid::parse_str(&x.uuid).unwrap_or_default();
            let mut credentials = Self {
                offline_profile: MinecraftProfile {
                    id: uuid,
                    name: x.username,
                    ..MinecraftProfile::default()
                },
                access_token: x.access_token,
                refresh_token: x.refresh_token,
                expires: Utc
                    .timestamp_opt(x.expires, 0)
                    .single()
                    .unwrap_or_else(Utc::now),
                active: x.active == 1,
            };

            async move {
                credentials.hydrate_tokens_from_keyring();
                credentials.refresh(exec).await.ok();
                acc.insert(uuid, credentials);

                Ok(acc)
            }
        })
        .await?;

        Ok(res)
    }

    pub async fn upsert(
        &self,
        exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite> + Copy,
    ) -> crate::Result<()> {
        let profile = if self.offline_profile.id != Uuid::default()
            && !self.offline_profile.name.is_empty()
        {
            MaybeOnlineMinecraftProfile::Offline(&self.offline_profile)
        } else {
            self.maybe_online_profile().await
        };
        let expires = self.expires.timestamp();
        let uuid = profile.id.as_hyphenated().to_string();

        store_keyring_secret(
            &access_token_keyring_key(&profile.id),
            &self.access_token,
        );
        store_keyring_secret(
            &refresh_token_keyring_key(&profile.id),
            &self.refresh_token,
        );

        if self.active {
            sqlx::query!(
                "
                UPDATE minecraft_users
                SET active = FALSE
                ",
            )
            .execute(exec)
            .await?;
        }

        // Never store plaintext access_token or refresh_token in SQLite.
        let redacted_token = "";
        sqlx::query!(
            "
            INSERT INTO minecraft_users (uuid, active, username, access_token, refresh_token, expires)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (uuid) DO UPDATE SET
                active = $2,
                username = $3,
                access_token = $4,
                refresh_token = $5,
                expires = $6
            ",
            uuid,
            self.active,
            profile.name,
            redacted_token,
            redacted_token,
            expires,
        )
        .execute(exec)
        .await?;

        Ok(())
    }

    pub async fn remove(
        uuid: Uuid,
        exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite>,
    ) -> crate::Result<()> {
        delete_keyring_secret(&access_token_keyring_key(&uuid));
        delete_keyring_secret(&refresh_token_keyring_key(&uuid));

        let uuid = uuid.as_hyphenated().to_string();

        sqlx::query!(
            "
            DELETE FROM minecraft_users WHERE uuid = $1
            ",
            uuid,
        )
        .execute(exec)
        .await?;

        Ok(())
    }
}

impl Serialize for Credentials {
    fn serialize<S: Serializer>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let profile = match Handle::try_current().ok() {
            Some(runtime)
                if runtime.runtime_flavor() == RuntimeFlavor::MultiThread =>
            {
                task::block_in_place(|| {
                    runtime.block_on(self.maybe_online_profile())
                })
            }
            Some(_) => MaybeOnlineMinecraftProfile::Offline(&self.offline_profile),
            None => tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_or_else(
                    |_| {
                        MaybeOnlineMinecraftProfile::Offline(
                            &self.offline_profile,
                        )
                    },
                    |runtime| runtime.block_on(self.maybe_online_profile()),
                ),
        };

        // Never serialize plaintext tokens over IPC to the frontend.
        let redacted = "";
        let mut ser = serializer.serialize_struct("Credentials", 5)?;
        ser.serialize_field("profile", &*profile)?;
        ser.serialize_field("access_token", &redacted)?;
        ser.serialize_field("refresh_token", &redacted)?;
        ser.serialize_field("expires", &self.expires)?;
        ser.serialize_field("active", &self.active)?;
        ser.end()
    }
}

pub struct RequestWithDate<T> {
    pub date: DateTime<Utc>,
    pub value: T,
}

#[derive(Deserialize)]
struct OAuthToken {
    pub expires_in: u64,
    pub access_token: String,
    #[serde(default)]
    pub refresh_token: Option<String>,
}

#[tracing::instrument(skip(code, verifier))]
async fn oauth_token(
    code: &str,
    verifier: &str,
    redirect_uri: &str,
) -> Result<RequestWithDate<OAuthToken>, MinecraftAuthenticationError> {
    let client_id = solara_microsoft_client_id();
    let mut query = HashMap::new();
    query.insert("client_id", client_id.as_str());
    query.insert("code", code);
    query.insert("code_verifier", verifier);
    query.insert("grant_type", "authorization_code");
    query.insert("redirect_uri", redirect_uri);
    query.insert("scope", MICROSOFT_OAUTH_SCOPE);

    let res = auth_retry(|| {
        INSECURE_REQWEST_CLIENT
            .post("https://login.microsoftonline.com/consumers/oauth2/v2.0/token")
            .header("Accept", "application/json")
            .form(&query)
            .send()
    })
    .await
    .map_err(|source| MinecraftAuthenticationError::Request {
        source,
        step: MinecraftAuthStep::GetOAuthToken,
    })?;

    let status = res.status();
    let current_date = get_date_header(res.headers());
    let text = res.text().await.map_err(|source| {
        MinecraftAuthenticationError::Request {
            source,
            step: MinecraftAuthStep::GetOAuthToken,
        }
    })?;

    let body = serde_json::from_str(&text).map_err(|source| {
        MinecraftAuthenticationError::DeserializeResponse {
            source,
            raw: text,
            step: MinecraftAuthStep::GetOAuthToken,
            status_code: status,
        }
    })?;

    Ok(RequestWithDate {
        date: current_date,
        value: body,
    })
}

#[tracing::instrument(skip(refresh_token))]
async fn oauth_refresh(
    refresh_token: &str,
) -> Result<RequestWithDate<OAuthToken>, MinecraftAuthenticationError> {
    let client_id = solara_microsoft_client_id();
    let mut query = HashMap::new();
    query.insert("client_id", client_id.as_str());
    query.insert("refresh_token", refresh_token);
    query.insert("grant_type", "refresh_token");
    query.insert("scope", MICROSOFT_OAUTH_SCOPE);

    let res = auth_retry(|| {
        INSECURE_REQWEST_CLIENT
            .post("https://login.microsoftonline.com/consumers/oauth2/v2.0/token")
            .header("Accept", "application/json")
            .form(&query)
            .send()
    })
    .await
    .map_err(|source| MinecraftAuthenticationError::Request {
        source,
        step: MinecraftAuthStep::RefreshOAuthToken,
    })?;

    let status = res.status();
    let current_date = get_date_header(res.headers());
    let text = res.text().await.map_err(|source| {
        MinecraftAuthenticationError::Request {
            source,
            step: MinecraftAuthStep::RefreshOAuthToken,
        }
    })?;

    let body = serde_json::from_str(&text).map_err(|source| {
        MinecraftAuthenticationError::DeserializeResponse {
            source,
            raw: text,
            step: MinecraftAuthStep::RefreshOAuthToken,
            status_code: status,
        }
    })?;

    Ok(RequestWithDate {
        date: current_date,
        value: body,
    })
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct XboxAuthResponse {
    pub token: String,
    pub display_claims: HashMap<String, serde_json::Value>,
}

#[tracing::instrument(skip(ms_access_token))]
async fn xbox_user_authenticate(
    ms_access_token: &str,
) -> Result<XboxAuthResponse, MinecraftAuthenticationError> {
    let res = auth_retry(|| {
        INSECURE_REQWEST_CLIENT
            .post("https://user.auth.xboxlive.com/user/authenticate")
            .header("Accept", "application/json")
            .json(&json!({
                "Properties": {
                    "AuthMethod": "RPS",
                    "SiteName": "user.auth.xboxlive.com",
                    "RpsTicket": format!("d={ms_access_token}"),
                },
                "RelyingParty": "http://auth.xboxlive.com",
                "TokenType": "JWT",
            }))
            .send()
    })
    .await
    .map_err(|source| MinecraftAuthenticationError::Request {
        source,
        step: MinecraftAuthStep::SisuAuthorize,
    })?;

    let status = res.status();
    let text = res.text().await.map_err(|source| {
        MinecraftAuthenticationError::Request {
            source,
            step: MinecraftAuthStep::SisuAuthorize,
        }
    })?;

    serde_json::from_str(&text).map_err(|source| {
        MinecraftAuthenticationError::DeserializeResponse {
            source,
            raw: text,
            step: MinecraftAuthStep::SisuAuthorize,
            status_code: status,
        }
    })
}

#[tracing::instrument(skip(xbl_token))]
async fn xsts_authorize(
    xbl_token: &str,
) -> Result<XboxAuthResponse, MinecraftAuthenticationError> {
    let res = auth_retry(|| {
        INSECURE_REQWEST_CLIENT
            .post("https://xsts.auth.xboxlive.com/xsts/authorize")
            .header("Accept", "application/json")
            .json(&json!({
                "Properties": {
                    "SandboxId": "RETAIL",
                    "UserTokens": [xbl_token],
                },
                "RelyingParty": "rp://api.minecraftservices.com/",
                "TokenType": "JWT",
            }))
            .send()
    })
    .await
    .map_err(|source| MinecraftAuthenticationError::Request {
        source,
        step: MinecraftAuthStep::XstsAuthorize,
    })?;

    let status = res.status();
    let text = res.text().await.map_err(|source| {
        MinecraftAuthenticationError::Request {
            source,
            step: MinecraftAuthStep::XstsAuthorize,
        }
    })?;

    serde_json::from_str(&text).map_err(|source| {
        MinecraftAuthenticationError::DeserializeResponse {
            source,
            raw: text,
            step: MinecraftAuthStep::XstsAuthorize,
            status_code: status,
        }
    })
}

#[derive(Deserialize)]
struct MinecraftToken {
    pub access_token: String,
}

#[tracing::instrument(skip(token))]
async fn minecraft_token(
    token: XboxAuthResponse,
) -> Result<MinecraftToken, MinecraftAuthenticationError> {
    let uhs = token
        .display_claims
        .get("xui")
        .and_then(|x| x.get(0))
        .and_then(|x| x.get("uhs"))
        .and_then(|x| x.as_str().map(String::from))
        .ok_or(MinecraftAuthenticationError::NoUserHash)?;

    let token = token.token;

    let res = auth_retry(|| {
        INSECURE_REQWEST_CLIENT
            .post(
                "https://api.minecraftservices.com/authentication/login_with_xbox",
            )
            .header("Accept", "application/json")
            .header("User-Agent", MINECRAFT_SERVICES_USER_AGENT)
            .json(&json!({
                "identityToken": format!("XBL3.0 x={uhs};{token}"),
            }))
            .send()
    })
    .await
    .map_err(|source| MinecraftAuthenticationError::Request {
        source,
        step: MinecraftAuthStep::MinecraftToken,
    })?;

    let status = res.status();
    let text = res.text().await.map_err(|source| {
        MinecraftAuthenticationError::Request {
            source,
            step: MinecraftAuthStep::MinecraftToken,
        }
    })?;

    serde_json::from_str(&text).map_err(|source| {
        MinecraftAuthenticationError::DeserializeResponse {
            source,
            raw: text,
            step: MinecraftAuthStep::MinecraftToken,
            status_code: status,
        }
    })
}

#[derive(
    sqlx::Type, Deserialize, Serialize, Debug, Copy, Clone, PartialEq, Eq,
)]
#[serde(rename_all = "UPPERCASE")]
#[sqlx(rename_all = "UPPERCASE")]
pub enum MinecraftSkinVariant {
    Classic,
    Slim,
    #[serde(other)]
    Unknown,
}

#[derive(Deserialize, Serialize, Debug, Copy, Clone, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum MinecraftCharacterExpressionState {
    Active,
    Inactive,
    #[serde(other)]
    Unknown,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct MinecraftSkin {
    pub id: Uuid,
    pub state: MinecraftCharacterExpressionState,
    pub url: Arc<Url>,
    #[serde(default, rename = "textureKey")]
    pub texture_key: Option<Arc<str>>,
    pub variant: MinecraftSkinVariant,
    #[serde(
        default,
        rename = "alias",
        deserialize_with = "normalize_skin_alias_case"
    )]
    pub name: Option<String>,
}

impl MinecraftSkin {
    pub fn texture_key(&self) -> Arc<str> {
        self.texture_key.as_ref().cloned().unwrap_or_else(|| {
            self.url
                .path_segments()
                .and_then(|mut path_segments| {
                    path_segments.next_back().map(String::from)
                })
                .unwrap_or_else(|| self.id.as_simple().to_string())
                .into()
        })
    }
}

fn normalize_skin_alias_case<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<String>, D::Error> {
    Ok(<Option<Cow<'_, str>>>::deserialize(deserializer)?
        .map(|alias| alias.to_title_case()))
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct MinecraftCape {
    pub id: Uuid,
    pub state: MinecraftCharacterExpressionState,
    pub url: Arc<Url>,
    #[serde(rename = "alias")]
    pub name: Arc<str>,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct MinecraftProfile {
    #[serde(default)]
    pub id: Uuid,
    pub name: String,
    #[serde(default)]
    pub skins: Vec<MinecraftSkin>,
    #[serde(default)]
    pub capes: Vec<MinecraftCape>,
    #[serde(skip)]
    pub fetch_time: Option<Instant>,
}

impl MinecraftProfile {
    fn is_fresh(&self, max_age: std::time::Duration) -> bool {
        self.fetch_time.is_some_and(|last_profile_fetch_time| {
            Instant::now().saturating_duration_since(last_profile_fetch_time)
                < max_age
        })
    }

    pub fn current_skin(&self) -> crate::Result<&MinecraftSkin> {
        self.skins
            .iter()
            .find(|skin| {
                skin.state == MinecraftCharacterExpressionState::Active
            })
            .context("no active skin found")
    }

    pub fn current_cape(&self) -> Option<&MinecraftCape> {
        self.capes.iter().find(|cape| {
            cape.state == MinecraftCharacterExpressionState::Active
        })
    }
}

pub enum MaybeOnlineMinecraftProfile<'profile> {
    Online(Arc<MinecraftProfile>),
    Offline(&'profile MinecraftProfile),
}

impl Deref for MaybeOnlineMinecraftProfile<'_> {
    type Target = MinecraftProfile;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Online(profile) => profile,
            Self::Offline(profile) => profile,
        }
    }
}

#[tracing::instrument(skip(token))]
async fn minecraft_profile(
    token: &str,
) -> Result<MinecraftProfile, MinecraftAuthenticationError> {
    let res = auth_retry(|| {
        INSECURE_REQWEST_CLIENT
            .get("https://api.minecraftservices.com/minecraft/profile")
            .header("Accept", "application/json")
            .header("User-Agent", MINECRAFT_SERVICES_USER_AGENT)
            .bearer_auth(token)
            .timeout(std::time::Duration::from_secs(10))
            .send()
    })
    .await
    .map_err(|source| MinecraftAuthenticationError::Request {
        source,
        step: MinecraftAuthStep::MinecraftProfile,
    })?;

    let status = res.status();
    let text = res.text().await.map_err(|source| {
        MinecraftAuthenticationError::Request {
            source,
            step: MinecraftAuthStep::MinecraftProfile,
        }
    })?;

    let mut profile =
        serde_json::from_str::<MinecraftProfile>(&text).map_err(|source| {
            MinecraftAuthenticationError::DeserializeResponse {
                source,
                raw: text,
                step: MinecraftAuthStep::MinecraftProfile,
                status_code: status,
            }
        })?;
    profile.fetch_time = Some(Instant::now());

    tracing::debug!(
        "Successfully fetched Minecraft profile for {}",
        profile.name
    );

    Ok(profile)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct MinecraftEntitlements {}

#[tracing::instrument(skip(token))]
async fn minecraft_entitlements(
    token: &str,
) -> Result<MinecraftEntitlements, MinecraftAuthenticationError> {
    let res = auth_retry(|| {
        INSECURE_REQWEST_CLIENT
            .get("https://api.minecraftservices.com/entitlements/mcstore")
            .header("Accept", "application/json")
            .header("User-Agent", MINECRAFT_SERVICES_USER_AGENT)
            .bearer_auth(token)
            .send()
    })
    .await
    .map_err(|source| MinecraftAuthenticationError::Request {
        source,
        step: MinecraftAuthStep::MinecraftEntitlements,
    })?;

    let status = res.status();
    let text = res.text().await.map_err(|source| {
        MinecraftAuthenticationError::Request {
            source,
            step: MinecraftAuthStep::MinecraftEntitlements,
        }
    })?;

    serde_json::from_str(&text).map_err(|source| {
        MinecraftAuthenticationError::DeserializeResponse {
            source,
            raw: text,
            step: MinecraftAuthStep::MinecraftEntitlements,
            status_code: status,
        }
    })
}

#[tracing::instrument(skip(reqwest_request))]
async fn auth_retry<F>(
    reqwest_request: impl Fn() -> F,
) -> Result<reqwest::Response, reqwest::Error>
where
    F: Future<Output = Result<Response, reqwest::Error>>,
{
    const RETRY_COUNT: usize = 5;
    const RETRY_WAIT: std::time::Duration =
        std::time::Duration::from_millis(250);

    let mut resp = reqwest_request().await;
    for i in 0..RETRY_COUNT {
        match &resp {
            Ok(_) => {
                break;
            }
            Err(err) => {
                if err.is_connect() || err.is_timeout() {
                    if i < RETRY_COUNT - 1 {
                        tracing::debug!(
                            "Request failed with connect error, retrying...",
                        );
                        tokio::time::sleep(RETRY_WAIT).await;
                        resp = reqwest_request().await;
                    } else {
                        break;
                    }
                }
            }
        }
    }

    resp
}

#[tracing::instrument]
fn get_date_header(headers: &HeaderMap) -> DateTime<Utc> {
    headers
        .get(reqwest::header::DATE)
        .and_then(|x| x.to_str().ok())
        .and_then(|x| DateTime::parse_from_rfc2822(x).ok())
        .map_or(Utc::now(), |x| x.with_timezone(&Utc))
}

#[derive(Deserialize)]
struct SolaraLegacyAccountsFile {
    #[serde(default)]
    accounts: Vec<SolaraLegacyAccountRecord>,
}

#[derive(Deserialize)]
struct SolaraLegacyAccountRecord {
    id: String,
    name: String,
    #[serde(default)]
    active: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SolaraLegacyInstanceRecord {
    id: String,
    name: String,
    game_version: String,
    loader: String,
    #[serde(default)]
    loader_version: Option<String>,
    #[serde(default)]
    state: String,
    #[serde(default)]
    group: Option<String>,
    #[serde(default)]
    playtime_seconds: u64,
}

/// Migrates existing Solara `V-0.0.6x` `accounts.json` records into SQLite `minecraft_users`
/// without overwriting existing accounts. Refresh tokens remain in `in.solara.launcher.minecraft.custom`.
pub async fn migrate_solara_v006x_accounts_from_dir(
    legacy_dir: &Path,
    exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite> + Copy,
) -> crate::Result<usize> {
    let candidates = [
        legacy_dir
            .join("custom-minecraft-accounts")
            .join("accounts.json"),
        legacy_dir.join("accounts.json"),
    ];
    let existing = Credentials::get_all(exec).await?;
    let mut migrated = 0usize;

    for path in candidates {
        let Ok(bytes) = tokio::fs::read(&path).await else {
            continue;
        };
        let Ok(parsed) = serde_json::from_slice::<SolaraLegacyAccountsFile>(&bytes)
        else {
            continue;
        };
        for record in parsed.accounts {
            let Ok(uuid) = Uuid::parse_str(record.id.trim()) else {
                continue;
            };
            if existing.contains_key(&uuid) || record.name.trim().is_empty() {
                continue;
            }
            let creds = Credentials {
                offline_profile: MinecraftProfile {
                    id: uuid,
                    name: record.name.trim().to_string(),
                    ..MinecraftProfile::default()
                },
                access_token: read_keyring_secret(&access_token_keyring_key(
                    &uuid,
                ))
                .unwrap_or_default(),
                refresh_token: read_keyring_secret(&refresh_token_keyring_key(
                    &uuid,
                ))
                .unwrap_or_default(),
                expires: Utc::now() - Duration::seconds(1),
                active: record.active || (existing.is_empty() && migrated == 0),
            };
            creds.upsert(exec).await?;
            migrated += 1;
        }
    }

    Ok(migrated)
}

/// Migrates existing Solara `V-0.0.6x` `custom-instances/<id>/instance.json` records and `game/` directories
/// into the launcher's SQLite `instances` table and `instances/` directory.
pub async fn migrate_solara_v006x_instances_from_dir(
    legacy_dir: &Path,
    instances_dir: &Path,
    pool: &sqlx::SqlitePool,
) -> crate::Result<usize> {
    use crate::state::instances::{
        ContentSet, ContentSetStatus, ContentSourceKind, Instance,
        InstanceLaunchOverrides, InstanceLink,
        adapters::sqlite::{content_rows, instance_rows},
    };
    use crate::state::{
        InstanceInstallStage, LauncherFeatureVersion, ModLoader, ReleaseChannel,
    };

    let root = legacy_dir.join("custom-instances");
    let Ok(mut entries) = tokio::fs::read_dir(&root).await else {
        return Ok(0);
    };

    tokio::fs::create_dir_all(instances_dir).await?;
    let mut migrated = 0usize;

    while let Ok(Some(entry)) = entries.next_entry().await {
        let dir = entry.path();
        if !dir.is_dir() {
            continue;
        }
        let meta_path = dir.join("instance.json");
        let Ok(bytes) = tokio::fs::read(&meta_path).await else {
            continue;
        };
        let Ok(record) =
            serde_json::from_slice::<SolaraLegacyInstanceRecord>(&bytes)
        else {
            continue;
        };
        let Ok(raw_uuid) = Uuid::parse_str(record.id.trim()) else {
            continue;
        };
        let instance_id = format!("local:{raw_uuid}");
        if instance_rows::get_instance_by_id(&instance_id, pool)
            .await?
            .is_some()
        {
            continue;
        }

        let safe_folder = record
            .name
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == ' '
                {
                    c
                } else {
                    '_'
                }
            })
            .collect::<String>()
            .trim()
            .to_string();
        let base_folder = if safe_folder.is_empty() {
            raw_uuid.to_string()
        } else {
            safe_folder
        };
        let mut rel_path = base_folder.clone();
        let mut idx = 1usize;
        while instances_dir.join(&rel_path).exists()
            || instance_rows::get_instance_by_path(&rel_path, pool)
                .await?
                .is_some()
        {
            rel_path = format!("{base_folder} ({idx})");
            idx += 1;
        }

        let target_dir = instances_dir.join(&rel_path);
        tokio::fs::create_dir_all(&target_dir).await?;
        let legacy_game_dir = dir.join("game");
        if legacy_game_dir.is_dir() {
            copy_dir_recursive(&legacy_game_dir, &target_dir).await.ok();
        }

        let now = Utc::now();
        let content_set_id = format!("content-set:{raw_uuid}");
        let install_stage = if record.state == "installed" {
            InstanceInstallStage::Installed
        } else {
            InstanceInstallStage::NotInstalled
        };
        let instance = Instance {
            id: instance_id.clone(),
            path: rel_path,
            applied_content_set_id: Some(content_set_id.clone()),
            install_stage,
            launcher_feature_version: LauncherFeatureVersion::MOST_RECENT,
            update_channel: ReleaseChannel::Release,
            name: record.name,
            icon_path: None,
            created: now,
            modified: now,
            last_played: None,
            submitted_time_played: 0,
            recent_time_played: record.playtime_seconds,
        };
        let content_set = ContentSet {
            id: content_set_id,
            instance_id: instance_id.clone(),
            name: "Default".to_string(),
            source_kind: ContentSourceKind::Local,
            status: ContentSetStatus::Available,
            game_version: record.game_version,
            protocol_version: None,
            loader: ModLoader::from_string(&record.loader),
            loader_version: record.loader_version,
            created: now,
            modified: now,
        };
        let overrides = InstanceLaunchOverrides::empty(instance_id.clone());
        let groups: Vec<String> = record
            .group
            .filter(|g| !g.trim().is_empty())
            .into_iter()
            .collect();

        let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;
        instance_rows::insert_instance(&instance, &mut tx).await?;
        instance_rows::insert_default_instance_sync_preferences(
            &instance_id,
            &mut tx,
        )
        .await?;
        content_rows::insert_content_set(&content_set, &mut tx).await?;
        instance_rows::upsert_instance_link(
            &instance_id,
            &InstanceLink::Unmanaged,
            &mut tx,
        )
        .await?;
        instance_rows::replace_instance_groups(&instance_id, &groups, &mut tx)
            .await?;
        instance_rows::upsert_instance_launch_overrides(&overrides, &mut tx)
            .await?;
        tx.commit().await?;

        migrated += 1;
    }

    Ok(migrated)
}

async fn copy_dir_recursive(src: &Path, dst: &Path) -> io::Result<()> {
    let mut stack = vec![(src.to_path_buf(), dst.to_path_buf())];
    while let Some((from, to)) = stack.pop() {
        tokio::fs::create_dir_all(&to).await?;
        let mut dir = tokio::fs::read_dir(&from).await?;
        while let Some(entry) = dir.next_entry().await? {
            let file_type = entry.file_type().await?;
            if file_type.is_symlink() {
                continue;
            }
            let src_path = entry.path();
            let dst_path = to.join(entry.file_name());
            if file_type.is_dir() {
                stack.push((src_path, dst_path));
            } else if file_type.is_file() && !dst_path.exists() {
                let _ = tokio::fs::copy(&src_path, &dst_path).await;
            }
        }
    }
    Ok(())
}

pub async fn migrate_solara_v006x_data(
    instances_dir: &Path,
    pool: &sqlx::SqlitePool,
) -> crate::Result<()> {
    let mut candidate_dirs = Vec::new();
    if let Some(local_data) = dirs::data_local_dir() {
        candidate_dirs.push(local_data.join("in.solara.launcher"));
    }
    if let Some(roaming_data) = dirs::data_dir() {
        let dir = roaming_data.join("in.solara.launcher");
        if !candidate_dirs.contains(&dir) {
            candidate_dirs.push(dir);
        }
    }
    for dir in candidate_dirs {
        if dir.exists() {
            let _ = migrate_solara_v006x_accounts_from_dir(&dir, pool).await;
            let _ = migrate_solara_v006x_instances_from_dir(
                &dir,
                instances_dir,
                pool,
            )
            .await;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solara_client_id_defaults_to_canonical_azure_app_id() {
        assert_eq!(
            DEFAULT_SOLARA_MICROSOFT_CLIENT_ID,
            "6a9238bb-3803-46f4-bf38-2f45bdf8c2ea"
        );
        assert_eq!(
            solara_microsoft_client_id(),
            "6a9238bb-3803-46f4-bf38-2f45bdf8c2ea"
        );
    }

    #[test]
    fn pkce_and_authorize_url_use_microsoft_consumers_oauth2_v2() {
        let (verifier, challenge, state) = generate_pkce_pair();
        assert!(verifier.len() >= 43);
        assert!(!challenge.is_empty());
        assert!(!state.is_empty());

        let url = build_authorize_url(
            DEFAULT_SOLARA_MICROSOFT_CLIENT_ID,
            "http://localhost:54321/callback",
            &state,
            &challenge,
        )
        .expect("authorize url");
        assert!(url.starts_with(
            "https://login.microsoftonline.com/consumers/oauth2/v2.0/authorize?"
        ));
        assert!(url.contains("client_id=6a9238bb-3803-46f4-bf38-2f45bdf8c2ea"));
        assert!(url.contains("code_challenge_method=S256"));
        assert!(url.contains("XboxLive.signin+offline_access"));
        assert!(!url.contains("00000000402b5328"));
    }

    #[tokio::test]
    async fn loopback_callback_validates_state_and_extracts_code() {
        crate::install_crypto_provider();
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let handle = tokio::spawn(async move {
            wait_for_loopback_code(listener, None, "expected-state-123").await
        });

        let resp = reqwest::get(format!(
            "http://127.0.0.1:{port}/callback?code=ms_oauth_code_abc&state=expected-state-123"
        ))
        .await
        .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let html = resp.text().await.unwrap();
        assert!(html.contains("Solara Launcher"));
        assert!(html.contains("Microsoft Account Connected"));

        let code = handle.await.unwrap().expect("oauth code");
        assert_eq!(code, "ms_oauth_code_abc");
    }

    #[tokio::test]
    async fn credentials_never_serialize_or_persist_plaintext_tokens_in_sqlite() {
        let dir = tempfile::tempdir().unwrap();
        let db_url = format!("sqlite://{}/app.db?mode=rwc", dir.path().display());
        let pool = sqlx::SqlitePool::connect(&db_url).await.unwrap();
        sqlx::query(
            "CREATE TABLE minecraft_users (
                uuid TEXT PRIMARY KEY,
                active INTEGER NOT NULL DEFAULT 0,
                username TEXT NOT NULL,
                access_token TEXT NOT NULL,
                refresh_token TEXT NOT NULL,
                expires INTEGER NOT NULL
            )",
        )
        .execute(&pool)
        .await
        .unwrap();

        let user_id = Uuid::new_v4();
        let creds = Credentials {
            offline_profile: MinecraftProfile {
                id: user_id,
                name: "SolaraPlayer".to_string(),
                ..MinecraftProfile::default()
            },
            access_token: "secret-mc-access-token".to_string(),
            refresh_token: "secret-ms-refresh-token".to_string(),
            expires: Utc::now() + Duration::hours(1),
            active: true,
        };

        // 1. IPC serialization must redact tokens
        let serialized = serde_json::to_value(&creds).unwrap();
        assert_eq!(serialized["access_token"], "");
        assert_eq!(serialized["refresh_token"], "");
        assert_eq!(serialized["profile"]["name"], "SolaraPlayer");

        // 2. SQLite persistence must redact tokens while keyring/memory retains them
        creds.upsert(&pool).await.unwrap();
        let row: (String, String) = sqlx::query_as(
            "SELECT access_token, refresh_token FROM minecraft_users WHERE uuid = $1",
        )
        .bind(user_id.as_hyphenated().to_string())
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(row.0, "");
        assert_eq!(row.1, "");

        let loaded = Credentials::get_active(&pool).await.unwrap().unwrap();
        assert_eq!(loaded.access_token, "secret-mc-access-token");
        assert_eq!(loaded.refresh_token, "secret-ms-refresh-token");
    }

    #[tokio::test]
    async fn v006x_accounts_migration_imports_accounts_and_preserves_keyring_tokens() {
        crate::install_crypto_provider();
        let dir = tempfile::tempdir().unwrap();
        let db_url = format!("sqlite://{}/app.db?mode=rwc", dir.path().display());
        let pool = sqlx::SqlitePool::connect(&db_url).await.unwrap();
        sqlx::query(
            "CREATE TABLE minecraft_users (
                uuid TEXT PRIMARY KEY,
                active INTEGER NOT NULL DEFAULT 0,
                username TEXT NOT NULL,
                access_token TEXT NOT NULL,
                refresh_token TEXT NOT NULL,
                expires INTEGER NOT NULL
            )",
        )
        .execute(&pool)
        .await
        .unwrap();

        let user_id = Uuid::new_v4();
        store_keyring_secret(
            &refresh_token_keyring_key(&user_id),
            "v0062-refresh-token",
        );

        let accounts_dir = dir.path().join("custom-minecraft-accounts");
        tokio::fs::create_dir_all(&accounts_dir).await.unwrap();
        tokio::fs::write(
            accounts_dir.join("accounts.json"),
            serde_json::to_vec(&json!({
                "accounts": [{
                    "id": user_id.to_string(),
                    "name": "LegacySolaraUser",
                    "active": true,
                    "owned": true
                }]
            }))
            .unwrap(),
        )
        .await
        .unwrap();

        let count = migrate_solara_v006x_accounts_from_dir(dir.path(), &pool)
            .await
            .unwrap();
        assert_eq!(count, 1);

        let all = Credentials::get_all(&pool).await.unwrap();
        let migrated = all.get(&user_id).expect("migrated user");
        assert_eq!(migrated.offline_profile.name, "LegacySolaraUser");
        assert_eq!(migrated.refresh_token, "v0062-refresh-token");
    }
}
