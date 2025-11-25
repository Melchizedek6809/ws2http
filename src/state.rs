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

    pub async fn unregister_handler_state(&self, handler_state: &HandlerState) {
        let mut map = self.aliases.lock().await;
        for alias in &handler_state.aliases {
            let set = map.get_mut(alias);
            if let Some(set) = set {
                if !set.remove(&handler_state.socket_id) {
                    eprintln!("Socket {} already removed from {alias}... this shouldn't happen", &handler_state.socket_id);
                }
            } else {
                eprintln!("Alias {alias} already removed from state.aliases... this shouldn't happen");
            }
        }
    }

    pub async fn send_text_message_to_alias(&self, alias: String, msg: String) -> i32  {
        let mut recipients = 0;
        let map = self.aliases.lock().await;
        let set = map.get(&alias);
        if let Some(set) = set {
            let ids = set.iter().map(|u| u.to_owned()).collect::<Vec<Uuid>>();
            drop(map);
            for socket_id in ids {
                let sockets = self.sockets.lock().await;
                let socket = sockets.get(&socket_id);
                if let Some(socket) = socket && socket.send(WsMessage::Text(msg.clone().into())).await.is_ok() {
                    recipients += 1;
                }
            }
        }
        recipients
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
