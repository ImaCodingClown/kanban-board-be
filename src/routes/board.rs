use crate::{
    config::AppState,
    models::cards::{Board, GetTeamPayload},
    services::board::{create_board, get_board_by_team, update_board},
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, put},
    Json, Router,
};

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct BoardQuery {
    pub team: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct UpdateBoardQuery {
    pub board: Board,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/board", get(handle_get_board))
        .route("/board", post(handle_create_board))
        .route("/board", put(handle_update_board))
}

async fn handle_get_board(
    State(state): State<AppState>,
    Query(payload): Query<BoardQuery>,
) -> impl IntoResponse {
    match get_board_by_team(payload.team, &state.db).await {
        Ok(board) => (StatusCode::OK, Json(board)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("{e}") })),
        )
            .into_response(),
    }
}

async fn handle_create_board(
    State(state): State<AppState>,
    Json(payload): Json<GetTeamPayload>,
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

async fn handle_update_board(
    State(state): State<AppState>,
    Json(payload): Json<UpdateBoardQuery>,
) -> impl IntoResponse {
    match update_board(payload.board, &state.db).await {
        Ok(board) => (StatusCode::OK, Json(board)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("{e}") })),
        )
            .into_response(),
    }
}
