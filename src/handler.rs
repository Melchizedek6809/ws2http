use std::collections::HashMap;

use crate::state::GlobalState;
use axum::{
    extract::{
        OriginalUri, Query, State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    http::HeaderMap,
    response::{IntoResponse, Response},
};
use futures::{sink::SinkExt, stream::StreamExt};
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
struct HandlerState {
    query: String,
    path: String,
    user_agent: String,
    cookie: String,
    forwarded_for: String,
}

async fn handle_socket(socket: WebSocket, handler_state: HandlerState, state: GlobalState) {
    let (socket_sender, mut receiver) = socket.split();
    // Bounded channel with capacity of 100 messages to prevent unbounded memory growth
    let (outgoing_tx, mut outgoing_rx) = mpsc::channel::<Message>(100);

    let socket_id = state.register_socket(outgoing_tx.clone()).await;

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

        println!("Msg: {message:?}");

        match message {
            Message::Text(payload) => {
                if outgoing_tx.try_send(Message::Text(payload)).is_err() {
                    break;
                }
            }
            Message::Binary(_) => {}
            Message::Ping(payload) => {
                if outgoing_tx.try_send(Message::Pong(payload)).is_err() {
                    break;
                }
            }
            Message::Pong(_) => {}
            Message::Close(frame) => {
                let _ = outgoing_tx.try_send(Message::Close(frame));
                break;
            }
        }
    }

    state.unregister_socket(socket_id).await;
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

    let handler_state = HandlerState {
        cookie,
        user_agent,
        forwarded_for,
        query: query.to_string(),
        path: path.to_string(),
    };

    let http = state.http_client();
    let form = reqwest::multipart::Form::new()
        .text("method", "connect")
        .text("query", query.to_string())
        .text("path", path.to_string());
    let app = http
        .request(reqwest::Method::POST, "http://localhost:1234/chat.php")
        .header("User-Agent", handler_state.user_agent.to_string())
        .header("Cookie", handler_state.cookie.to_string())
        .header("X-Forwarded-For", handler_state.forwarded_for.to_string())
        .multipart(form)
        .build();

    if let Ok(app) = app {
        let res = http.execute(app).await;
        match res {
            Err(err) => {
                eprintln!("Error: {err:?}");
                (axum::http::StatusCode::BAD_GATEWAY, "Bad request").into_response()
            }
            Ok(res) => {
                eprintln!("Res: {res:?}");
                ws.on_upgrade(|socket| handle_socket(socket, handler_state, state))
            }
        }
    } else {
        (axum::http::StatusCode::BAD_REQUEST, "Bad request").into_response()
    }
}
