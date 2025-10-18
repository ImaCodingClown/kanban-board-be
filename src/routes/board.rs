use crate::{
    config::AppState,
    models::cards::{Board, BoardIdQuery, TeamQuery},
    services::board::{create_board, delete_board, get_board_by_id, get_boards_by_team, update_board},
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct CreateBoardPayload {
    pub team: String,
    pub board_name: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct UpdateBoardQuery {
    pub board: Board,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/board", get(handle_get_board))
        .route("/boards", get(handle_get_boards))
        .route("/board", post(handle_create_board))
        .route("/board", put(handle_update_board))
        .route("/board", delete(handle_delete_board))
}

async fn handle_get_board(
    State(state): State<AppState>,
    Query(payload): Query<BoardIdQuery>,
) -> impl IntoResponse {
    match get_board_by_id(payload.board_id, &state.db).await {
        Ok(board) => (StatusCode::OK, Json(board)).into_response(),
        Err(e) => (
            e.to_status_code(),
            Json(e.to_error_response()),
        ).into_response(),
    }
}

async fn handle_get_boards(
    State(state): State<AppState>,
    Query(payload): Query<TeamQuery>,
) -> impl IntoResponse {
    match get_boards_by_team(payload.team, &state.db).await {
        Ok(boards) => (StatusCode::OK, Json(boards)).into_response(),
        Err(e) => (
            e.to_status_code(),
            Json(e.to_error_response()),
        ).into_response(),
    }
}

async fn handle_create_board(
    State(state): State<AppState>,
    Json(payload): Json<CreateBoardPayload>,
) -> impl IntoResponse {
    match create_board(payload.team, payload.board_name, &state.db).await {
        Ok(board) => (StatusCode::CREATED, Json(board)).into_response(),
        Err(e) => (
            e.to_status_code(),
            Json(e.to_error_response()),
        ).into_response(),
    }
}

async fn handle_update_board(
    State(state): State<AppState>,
    Json(payload): Json<UpdateBoardQuery>,
) -> impl IntoResponse {
    match update_board(payload.board, &state.db).await {
        Ok(board) => (StatusCode::OK, Json(board)).into_response(),
        Err(e) => (
            e.to_status_code(),
            Json(e.to_error_response()),
        ).into_response(),
    }
}

async fn handle_delete_board(
    State(state): State<AppState>,
    Query(payload): Query<BoardIdQuery>,
) -> impl IntoResponse {
    match delete_board(payload.board_id, &state.db).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            e.to_status_code(),
            Json(e.to_error_response()),
        ).into_response(),
    }
}
