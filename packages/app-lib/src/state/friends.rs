use crate::LabrinthError;
use crate::data::ModrinthCredentials;
use crate::event::FriendPayload;
use crate::event::emit::{emit_friend, emit_notification};
use crate::state::tunnel::InternalTunnelSocket;
use crate::state::{ProcessManager, TunnelSocket};
use crate::util::fetch::{FetchSemaphore, fetch_advanced, fetch_json};
use anyhow::bail;
use ariadne::ids::UserId;
use ariadne::networking::message::{
    ClientToServerMessage, ServerToClientMessage,
};
use ariadne::users::UserStatus;
use async_tungstenite::WebSocketSender;
use async_tungstenite::tokio::{ConnectStream, connect_async};
use async_tungstenite::tungstenite::Message;
use async_tungstenite::tungstenite::client::IntoClientRequest;
use bytes::Bytes;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use either::Either;
use futures::{SinkExt, StreamExt};
use reqwest::Method;
use reqwest::header::HeaderValue;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::net::SocketAddr;
use std::ops::Deref;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::net::tcp::OwnedReadHalf;
use tokio::sync::{Mutex, RwLock};
use uuid::Uuid;

pub(super) type WriteSocket =
    Arc<RwLock<Option<WebSocketSender<ConnectStream>>>>;
pub(super) type TunnelSockets = Arc<DashMap<Uuid, Arc<InternalTunnelSocket>>>;

pub struct FriendsSocket {
    write: WriteSocket,
    connection_lock: Mutex<()>,
    connection_generation: Arc<AtomicU64>,
    user_statuses: Arc<DashMap<UserId, UserStatus>>,
    tunnel_sockets: TunnelSockets,
}

#[derive(Deserialize, Serialize)]
pub struct UserFriend {
    pub id: String,
    pub friend_id: String,
    pub accepted: bool,
    pub created: DateTime<Utc>,
}

impl Default for FriendsSocket {
    fn default() -> Self {
        Self::new()
    }
}

impl FriendsSocket {
    pub fn new() -> Self {
        Self {
            write: Arc::new(RwLock::new(None)),
            connection_lock: Mutex::new(()),
            connection_generation: Arc::new(AtomicU64::new(0)),
            user_statuses: Arc::new(DashMap::new()),
            tunnel_sockets: Arc::new(DashMap::new()),
        }
    }

    #[tracing::instrument(skip_all)]
    pub async fn connect(
        &self,
        _exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite> + Copy,
        _semaphore: &FetchSemaphore,
        _process_manager: &ProcessManager,
    ) -> crate::Result<()> {
        Ok(())
    }

    async fn handle_notification(notification: Value) -> crate::Result<()> {
        tracing::info!(
            notification = %notification,
            "Received websocket notification payload"
        );

        if notification
            .get("body")
            .and_then(|body| body.get("type"))
            .and_then(Value::as_str)
            .is_some()
        {
            emit_notification(notification).await?;
        }

        Ok(())
    }

    #[tracing::instrument(skip_all)]
    pub async fn socket_loop(state: Arc<crate::State>) -> crate::Result<()> {
        tokio::task::spawn(async move {
            let mut last_connection = Utc::now();
            let mut last_ping = Utc::now();

            loop {
                let connected =
                    state.presence.friends_socket.is_connected().await;

                if !connected
                    && Utc::now().signed_duration_since(last_connection)
                        > chrono::Duration::seconds(30)
                {
                    last_connection = Utc::now();
                    last_ping = Utc::now();
                    let _ = state
                        .presence
                        .friends_socket
                        .connect(
                            &state.pool,
                            &state.api_semaphore,
                            &state.process_manager,
                        )
                        .await;
                } else if connected
                    && Utc::now().signed_duration_since(last_ping)
                        > chrono::Duration::seconds(10)
                {
                    last_ping = Utc::now();
                    let mut write =
                        state.presence.friends_socket.write.write().await;
                    if let Some(write) = write.as_mut() {
                        let _ = write.send(Message::Ping(Bytes::new())).await;
                    }
                }

                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        });

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn disconnect(&self) -> crate::Result<()> {
        let _connection_guard = self.connection_lock.lock().await;
        self.connection_generation.fetch_add(1, Ordering::AcqRel);

        let mut write_lock = self.write.write().await;
        let write_half = write_lock.take();
        drop(write_lock);

        self.user_statuses.clear();
        self.tunnel_sockets.clear();

        if let Some(mut write_half) = write_half {
            SinkExt::close(&mut write_half).await?;
        }

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn update_status(
        &self,
        instance_name: Option<String>,
    ) -> crate::Result<()> {
        Self::send_message(
            &self.write,
            ClientToServerMessage::StatusUpdate {
                profile_name: instance_name,
            },
        )
        .await
    }

    #[tracing::instrument(skip_all)]
    pub async fn friends(
        exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite> + Copy,
        semaphore: &FetchSemaphore,
    ) -> crate::Result<Vec<UserFriend>> {
        fetch_json(
            Method::GET,
            concat!(env!("MODRINTH_API_URL_V3"), "friends"),
            None,
            None,
            Some("/v3/friends"),
            semaphore,
            exec,
        )
        .await
    }

    #[tracing::instrument(skip(self))]
    pub fn friend_statuses(&self) -> Vec<UserStatus> {
        self.user_statuses
            .iter()
            .map(|x| x.value().clone())
            .collect()
    }

    #[tracing::instrument(skip(exec, semaphore))]
    pub async fn add_friend(
        user_id: &str,
        exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite> + Copy,
        semaphore: &FetchSemaphore,
    ) -> crate::Result<()> {
        let result = fetch_advanced(
            Method::POST,
            &format!("{}friend/{user_id}", env!("MODRINTH_API_URL_V3")),
            None,
            None,
            None,
            None,
            None,
            Some("/v3/friend/:user_id"),
            semaphore,
            exec,
        )
        .await;

        if let Err(ref e) = result
            && let Some(e) = e.downcast_ref::<LabrinthError>()
            && e.error == "not_found"
        {
            bail!("no user found with username \"{user_id}\"");
        }
        result?;

        Ok(())
    }

    #[tracing::instrument(skip(exec, semaphore))]
    pub async fn remove_friend(
        user_id: &str,
        exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite> + Copy,
        semaphore: &FetchSemaphore,
    ) -> crate::Result<()> {
        fetch_advanced(
            Method::DELETE,
            &format!("{}friend/{user_id}", env!("MODRINTH_API_URL_V3")),
            None,
            None,
            None,
            None,
            None,
            Some("/v3/friend/:user_id"),
            semaphore,
            exec,
        )
        .await?;

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn open_port(&self, port: u16) -> crate::Result<TunnelSocket> {
        let socket_id = Uuid::new_v4();
        let socket = self.tunnel_sockets.entry(socket_id).insert(Arc::new(
            InternalTunnelSocket::Listening(SocketAddr::new(
                "127.0.0.1".parse().unwrap(),
                port,
            )),
        ));
        Self::send_message(
            &self.write,
            ClientToServerMessage::SocketListen { socket: socket_id },
        )
        .await?;
        self.create_tunnel_socket(socket_id, socket)
    }

    pub async fn is_connected(&self) -> bool {
        self.write.read().await.is_some()
    }

    fn create_tunnel_socket(
        &self,
        socket_id: Uuid,
        socket: impl Deref<Target = Arc<InternalTunnelSocket>>,
    ) -> crate::Result<TunnelSocket> {
        Ok(TunnelSocket {
            socket_id,
            write: self.write.clone(),
            sockets: self.tunnel_sockets.clone(),
            internal: socket.clone(),
        })
    }

    fn socket_read_loop(
        write: WriteSocket,
        mut read_half: OwnedReadHalf,
        socket_id: Uuid,
    ) {
        tokio::spawn(async move {
            let mut read_buffer = [0u8; 8192];
            loop {
                match read_half.read(&mut read_buffer).await {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        let _ = Self::send_message(
                            &write,
                            ClientToServerMessage::SocketSend {
                                socket: socket_id,
                                data: read_buffer[..n].to_vec(),
                            },
                        )
                        .await;
                    }
                };
            }
        });
    }

    #[tracing::instrument(skip(write))]
    pub(super) async fn send_message(
        write: &WriteSocket,
        message: ClientToServerMessage,
    ) -> crate::Result<()> {
        let serialized = match message.serialize()? {
            Either::Left(text) => Message::text(text),
            Either::Right(bytes) => Message::binary(bytes),
        };

        let mut write_lock = write.write().await;
        if let Some(ref mut write_half) = *write_lock {
            write_half.send(serialized).await?;
        }

        Ok(())
    }
}
