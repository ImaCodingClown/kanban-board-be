use axum::{routing::get, Json, Router, response::IntoResponse, http::StatusCode};
use crate::config::AppState;
use crate::services::board::get_board;

pub fn routes() -> Router<AppState> {
    Router::new().route("/board", get(handle_get_board))
    //http://mydomain.com/board
}

// Changed return type from `Json<Vec<Column>>` to `impl IntoResponse`
// because we're returning a full HTTP response (status + JSON).
async fn handle_get_board() -> impl IntoResponse {
    match get_board().await {
        // Returns (StatusCode, Json) tuple converted into an HTTP response
        Ok(columns) => (StatusCode::OK, Json(columns)).into_response(),

        // Returns 500 error with JSON error message
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}
