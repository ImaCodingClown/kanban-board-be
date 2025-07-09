// File: src/routes/board.rs
use crate::{
    config::AppState,
    models::cards::CreateBoardPayload,
    services::board::{create_board, get_board_by_team},
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct BoardQuery {
    pub team: String,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/board", get(handle_get_board))
        .route("/board", post(handle_create_board))
}

async fn handle_get_board(
    State(state): State<AppState>,
    Query(payload): Query<BoardQuery>,
) -> impl IntoResponse {
    match get_board_by_team(payload.team, &state.db).await {
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
