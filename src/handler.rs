use crate::state::GlobalState;
use axum::{
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::Response,
};
use futures::{sink::SinkExt, stream::StreamExt};
use tokio::sync::mpsc;
use tower_cookies::Cookies;

async fn handle_socket(socket: WebSocket, _cookies: Cookies, state: GlobalState) {
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
    cookies: Cookies,
    State(state): State<GlobalState>,
) -> Response {
    println!("main_handler {cookies:?}");
    ws.on_upgrade(|socket| handle_socket(socket, cookies, state))
}
