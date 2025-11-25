use axum::extract::ws::Message as WsMessage;
use std::{collections::HashMap, net::SocketAddr, sync::Arc, time::Duration};
use tokio::sync::{Mutex, mpsc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub bind_addr: SocketAddr,
}

type SocketMap = Arc<Mutex<HashMap<Uuid, mpsc::Sender<WsMessage>>>>;

#[derive(Debug, Clone)]
pub struct GlobalState {
    sockets: SocketMap,
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
            sockets: Arc::new(Mutex::new(HashMap::new())),
            http_client,
            config: AppConfig { bind_addr },
        };
        Ok(state)
    }

    pub async fn register_socket(&self, sender: mpsc::Sender<WsMessage>) -> Uuid {
        let mut sockets = self.sockets.lock().await;
        let socket_id = Uuid::now_v7();
        sockets.insert(socket_id, sender);
        socket_id
    }

    pub async fn unregister_socket(&self, socket_id: Uuid) {
        let mut sockets = self.sockets.lock().await;
        sockets.remove(&socket_id);
    }

    pub fn http_client(&self) -> reqwest::Client {
        self.http_client.clone()
    }
}
