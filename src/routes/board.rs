use crate::services::board::{create_board, get_board};
use crate::{
    config::AppState,
    models::cards::{Board, CreateBoardPayload},
};
use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/v1/board", get(handle_get_board))
        .route("/v1/board", post(handle_create_board))
}

async fn handle_get_board(
    state: axum::extract::State<AppState>,
    Json(payload): Json<Board>,
) -> impl IntoResponse {
    match get_board(payload.team, &state.db).await {
        // Returns (StatusCode, Json) tuple converted into an HTTP response
        Ok(columns) => (StatusCode::OK, Json(columns)).into_response(),

        // Returns 500 error with JSON error message
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("{e}") })),
        )
            .into_response(),
    }
}

async fn handle_create_board(
    state: axum::extract::State<AppState>,
    Json(payload): Json<CreateBoardPayload>,
) -> impl IntoResponse {
    match create_board(payload.team, &state.db).await {
        Ok(board) => (StatusCode::CREATED, Json(board)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("{e}") })),
        )
            .into_response(),
    }
}
