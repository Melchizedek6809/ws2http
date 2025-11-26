use axum::{Json, Router, extract::State, response::IntoResponse, routing::{get, post}};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize, Debug)]
struct ApiSend {
    aliases: Option<Vec<String>>,
    text_messages: Option<Vec<String>>,
}

use crate::state::GlobalState;

async fn api_send(
    State(state): State<GlobalState>,
    axum::extract::Json(form): axum::extract::Json<ApiSend>,
) -> impl IntoResponse {
    eprintln!("{form:?}");

    let mut recipients = 0;
    if let Some(aliases) = &form.aliases && let Some(msgs) = &form.text_messages {
        for msg in msgs {
            for alias in aliases {
                recipients += state.send_text_message_to_alias(alias.to_owned(), msg.to_owned()).await;
            }
        }
    }

    Json(json!({
        "success": true,
        "recipients": recipients,
    }))
}

async fn api_stats(
    State(state): State<GlobalState>,
) -> impl IntoResponse {
    Json(state.get_stats().await)
}


pub fn router() -> Router<GlobalState> {
    Router::new()
        .route("/send", post(api_send))
        .route("/stats", get(api_stats))
}
