use axum::routing::get;
use axum::{routing::post, Json, Router};

use crate::config::AppState;
use crate::get_board;
use crate::models::auth::AuthPayload;
use crate::services::auth::{login, signup};

pub fn routes() -> Router<AppState> {
    Router::new().route("/board", get(handle_get_board))
}

async fn handle_get_board() -> Json<serde_json::Value> {
    match get_board().await {
        Ok(token) => Json(serde_json::json!({ "board": token })),
        Err(e) => Json(serde_json::json!({ "error": e })),
    }
}
