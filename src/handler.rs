use std::{collections::{HashMap, HashSet}, time::Duration};

use crate::state::GlobalState;
use anyhow::{Result, anyhow};
use axum::{
    extract::{
        OriginalUri, State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    http::HeaderMap,
    response::{Response},
};
use futures::{sink::SinkExt, stream::StreamExt};
use reqwest::Client;
use tokio::sync::mpsc;
use uuid::Uuid;
use serde::Deserialize;

enum EndpointEvent {
    Connect,
    Message,
    Close,
}

#[derive(Debug, Clone)]
pub struct HandlerState {
    pub endpoint: String,
    pub query: String,
    pub path: String,
    pub user_agent: String,
    pub cookie: String,
    pub forwarded_for: String,

    pub socket_id: Uuid,
    pub aliases: HashSet<String>,
    pub meta: HashMap<String, String>,
}

#[derive(Deserialize, Debug, Default)]
pub struct EndpointResponse {
    pub aliases: Option<Vec<String>>,
    pub meta: Option<HashMap<String, String>>,
    pub text_messages: Option<Vec<String>>,
}

impl HandlerState {
    async fn endpoint_request(&mut self, http: Client, event_type: EndpointEvent, text: Option<&str>) -> Result<EndpointResponse> {
        let method = match event_type {
            EndpointEvent::Close => "close",
            EndpointEvent::Connect => "connect",
            EndpointEvent::Message => "message",
        };

        let form = reqwest::multipart::Form::new()
            .text("method", method)
            .text("query", self.query.to_string())
            .text("path", self.path.to_string());

        let form = if let Some(text) = text {
            form.text("text", text.to_string())
        } else {
            form
        };

        let app = http
            .request(reqwest::Method::POST, &self.endpoint)
            .header("User-Agent", self.user_agent.to_string())
            .header("Cookie", self.cookie.to_string())
            .header("X-Forwarded-For", self.forwarded_for.to_string())
            .timeout(Duration::from_millis(3000)) // 3 second timeout for the endpoint - if the endpoint is overloaded we should drop the connection
            .multipart(form)
            .build()?;

        let res = http.execute(app).await?;
        if !res.status().is_success() {
            return Err(anyhow!("Endpoint request returned {}", res.status()));
        }
        let raw = res.text().await?;
        let res: EndpointResponse = serde_json::from_str(&raw).unwrap_or_default();

        if let Some(meta) = &res.meta {
            for (key, val) in meta.iter() {
                self.meta.insert(key.to_owned(), val.to_owned());
            }
        }

        Ok(res)
    }
}

async fn handle_socket(socket: WebSocket, mut handler_state: HandlerState, state: GlobalState) {
    let (socket_sender, mut receiver) = socket.split();
    // Bounded channel with capacity of 100 messages to prevent unbounded memory growth
    let (outgoing_tx, mut outgoing_rx) = mpsc::channel::<Message>(512);

    let res = handler_state.endpoint_request(state.http_client(), EndpointEvent::Connect, None).await;
    let res = match res {
        Ok(res) => res,
        Err(_) => return,
    };

    state.register_socket(handler_state.socket_id, outgoing_tx.clone()).await;
    if state.handle_endpoint_response(&mut handler_state, res).await.is_err() {
        state.unregister_socket(handler_state.socket_id).await;
        state.unregister_handler_state(&handler_state).await;
        let _ = outgoing_tx.try_send(Message::Close(None));
        return;
    }

    let send_task = tokio::spawn(async move {
        let mut socket_sender = socket_sender;

        while let Some(message) = outgoing_rx.recv().await {
            if socket_sender.send(message).await.is_err() {
                break;
            }
        }
    });

    while let Some(message_result) = receiver.next().await {
        let message = match message_result {
            Ok(message) => message,
            Err(_) => break,
        };

        match message {
            Message::Text(payload) => {
                let req = handler_state.endpoint_request(state.http_client(), EndpointEvent::Message, Some(&payload)).await;
                match req {
                    Ok(req) => {
                        let res = state.handle_endpoint_response(&mut handler_state, req).await;
                        if res.is_err() {
                            eprintln!("{res:?}");
                            break;
                        }
                    },
                    Err(err) => {
                        eprintln!("{err:?}");
                        break;
                    },
                }
            }
            Message::Binary(_) => {}
            Message::Pong(_) => {}
            Message::Ping(payload) => {
                if outgoing_tx.try_send(Message::Pong(payload)).is_err() {
                    break;
                }
            }
            Message::Close(frame) => {
                // Doesn't matter if there's an error in the endpoint here since we're closing the socket
                let _ = outgoing_tx.try_send(Message::Close(frame));
                let _ = handler_state.endpoint_request(state.http_client(), EndpointEvent::Close, None).await;
                break;
            }
        }
    }

    // Always try and explicitly close the socket on the client
    let _ = outgoing_tx.try_send(Message::Close(None));
    state.unregister_socket(handler_state.socket_id).await;
    state.unregister_handler_state(&handler_state).await;
    drop(outgoing_tx);
    let _ = send_task.await;
}

pub async fn main_handler(
    ws: WebSocketUpgrade,
    headers: HeaderMap,
    uri: OriginalUri,
    State(state): State<GlobalState>,
) -> Response {
    let uri = uri.0.path_and_query();
    let (path, query) = uri
        .map(|v| (v.path(), v.query().unwrap_or_default()))
        .unwrap_or_default();

    let cookie = headers
        .get("Cookie")
        .map(|h| h.to_str().unwrap_or(""))
        .unwrap_or("")
        .to_string();

    let user_agent = headers
        .get("User-Agent")
        .map(|h| h.to_str().unwrap_or(""))
        .unwrap_or("")
        .to_string();

    let forwarded_for = headers
        .get("X-Forwarded-For")
        .map(|h| h.to_str().unwrap_or(""))
        .unwrap_or("")
        .to_string();

    let socket_id = Uuid::now_v7();

    let handler_state = HandlerState {
        endpoint: "http://localhost:1234/chat.php".to_string(),
        cookie,
        user_agent,
        forwarded_for,
        query: query.to_string(),
        path: path.to_string(),

        socket_id,
        aliases: HashSet::new(),
        meta: HashMap::new(),
    };

    ws.on_upgrade(|socket| handle_socket(socket, handler_state, state))
}
