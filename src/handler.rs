use std::collections::{HashMap, HashSet};

use crate::state::GlobalState;
use anyhow::Result;
use axum::{
    extract::{
        OriginalUri, Query, State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    http::HeaderMap,
    response::{IntoResponse, Response},
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

#[derive(Deserialize, Debug)]
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
            .multipart(form)
            .build()?;

        let res = http.execute(app).await?;
        let raw = res.text().await?;
        let res: EndpointResponse = serde_json::from_str(&raw)?;

        if let Some(meta) = &res.meta {
            for (key, val) in meta.iter() {
                self.meta.insert(key.to_owned(), val.to_owned());
            }
        }

        Ok(res)
    }
}

async fn handle_socket(socket: WebSocket, mut handler_state: HandlerState, state: GlobalState, res: EndpointResponse) {
    let (socket_sender, mut receiver) = socket.split();
    // Bounded channel with capacity of 100 messages to prevent unbounded memory growth
    let (outgoing_tx, mut outgoing_rx) = mpsc::channel::<Message>(100);

    state.register_socket(handler_state.socket_id, outgoing_tx.clone()).await;
    state.handle_endpoint_response(&mut handler_state, res).await;

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
                if let Ok(req) = req {
                    state.handle_endpoint_response(&mut handler_state, req).await;
                } else {
                    break;
                }

                if outgoing_tx.try_send(Message::Text(payload)).is_err() {
                    break;
                }
            }
            Message::Binary(_) => {}
            Message::Ping(_) => {}
            Message::Pong(_) => {}
            Message::Close(frame) => {
                let req = handler_state.endpoint_request(state.http_client(), EndpointEvent::Close, None).await;
                if let Ok(req) = req {
                    state.handle_endpoint_response(&mut handler_state, req).await;
                } else {
                    break;
                }

                let _ = outgoing_tx.try_send(Message::Close(frame));
                break;
            }
        }
    }

    state.unregister_socket(handler_state.socket_id).await;
    state.unregister_handler_state(&handler_state).await;
    drop(outgoing_tx);
    let _ = send_task.await;
}

pub async fn main_handler(
    ws: WebSocketUpgrade,
    headers: HeaderMap,
    uri: OriginalUri,
    Query(params): Query<HashMap<String, String>>,
    State(state): State<GlobalState>,
) -> Response {
    let uri = uri.0.path_and_query();
    let (path, query) = uri
        .map(|v| (v.path(), v.query().unwrap_or_default()))
        .unwrap_or_default();

    println!("{path:?} {query:?} {headers:?}\n{params:?}");

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

    let mut handler_state = HandlerState {
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

    let req = handler_state.endpoint_request(state.http_client(), EndpointEvent::Connect, None).await;
    if let Ok(req) = req {
        ws.on_upgrade(|socket| handle_socket(socket, handler_state, state, req))
    } else {
        (axum::http::StatusCode::BAD_REQUEST, "Bad request").into_response()
    }
}
