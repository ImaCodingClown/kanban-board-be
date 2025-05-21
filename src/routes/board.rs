use axum::{routing::get, Json, Router};
use crate::config::AppState;
use crate::services::board::get_board;
use crate::models::cards::Column;

pub fn routes() -> Router<AppState> {
    Router::new().route("/board", get(handle_get_board))
}


async fn handle_get_board() -> Json<Vec<Column>> {
    match get_board().await {
        Ok(columns) => (StatusCode::OK, Json(columns)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

