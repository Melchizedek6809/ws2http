use axum::extract::ws::Message as WsMessage;
use std::{collections::{HashMap, HashSet}, net::SocketAddr, sync::Arc, time::Duration};
use tokio::sync::{Mutex, mpsc};
use uuid::Uuid;

use crate::handler::{EndpointResponse, HandlerState};

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub bind_addr: SocketAddr,
}

type SocketMap = Arc<Mutex<HashMap<Uuid, mpsc::Sender<WsMessage>>>>;

#[derive(Debug, Clone)]
pub struct GlobalState {
    sockets: SocketMap,
    aliases: Arc<Mutex<HashMap<String, HashSet<Uuid>>>>,
    http_client: reqwest::Client,
    pub config: AppConfig,
}

impl GlobalState {
    pub async fn new() -> anyhow::Result<Self> {
        let bind_addr = std::env::var("BIND_ADDR")
            .unwrap_or_else(|_| "127.0.0.1:4000".to_owned())
            .parse::<SocketAddr>()?;

        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("WS2HTTP/0.1")
            .build()?;

        let state = Self {
            aliases: Arc::new(Mutex::new(HashMap::new())),
            sockets: Arc::new(Mutex::new(HashMap::new())),
            http_client,
            config: AppConfig { bind_addr },
        };
        Ok(state)
    }

    pub async fn register_socket(&self, socket_id: Uuid, sender: mpsc::Sender<WsMessage>) {
        let mut sockets = self.sockets.lock().await;

        sockets.insert(socket_id, sender);
    }

    pub async fn unregister_socket(&self, socket_id: Uuid) {
        let mut sockets = self.sockets.lock().await;
        sockets.remove(&socket_id);
    }

    pub async fn register_alias(&self, alias: &str, socket_id: Uuid) {
        let mut map = self.aliases.lock().await;
        let set = map.get_mut(alias);
        if let Some(set) = set {
            set.insert(socket_id);
        } else {
            map.insert(alias.to_owned(), [socket_id].into());
        }
    }

    pub async fn handle_endpoint_response(&self, handler_state: &mut HandlerState, response: EndpointResponse) {
        if let Some(aliases) = &response.aliases {
            for alias in aliases.iter() {
                if !handler_state.aliases.contains(alias) {
                    handler_state.aliases.insert(alias.to_owned());
                    self.register_alias(alias, handler_state.socket_id).await;
                }
            }
        }

        if let Some(msgs) = &response.text_messages {
            for msg in msgs {
                let sinks = self.sockets.lock().await;
                let sink = sinks.get(&handler_state.socket_id);
                if let Some(sink) = sink {
                    sink.send(WsMessage::Text(msg.into())).await.expect("MSPC Send error");
                }
            }
        }
    }

    pub fn http_client(&self) -> reqwest::Client {
        self.http_client.clone()
    }
}
