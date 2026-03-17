use crate::{
    config::AppState,
    models::cards::{Board, BoardIdQuery, TeamQuery},
    services::{
        board::{create_board, delete_board, get_boards_by_team, update_board},
        permission::{
            check_board_access, require_board_write_permission, require_team_leader,
            require_team_membership,
        },
    },
    utils::jwt::AuthBearer,
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
    AuthBearer(user_email): AuthBearer,
    Query(payload): Query<BoardIdQuery>,
) -> impl IntoResponse {
    match check_board_access(&state.db, &user_email, &payload.board_id).await {
        Ok(board) => (StatusCode::OK, Json(board)).into_response(),
        Err(e) => (e.to_status_code(), Json(e.to_error_response())).into_response(),
    }
}

async fn handle_get_boards(
    State(state): State<AppState>,
    AuthBearer(user_email): AuthBearer,
    Query(payload): Query<TeamQuery>,
) -> impl IntoResponse {
    if let Err(e) = require_team_membership(&state.db, &user_email, &payload.team).await {
        return (e.to_status_code(), Json(e.to_error_response())).into_response();
    }

    match get_boards_by_team(payload.team, &state.db).await {
        Ok(boards) => (StatusCode::OK, Json(boards)).into_response(),
        Err(e) => (e.to_status_code(), Json(e.to_error_response())).into_response(),
    }
}

async fn handle_create_board(
    State(state): State<AppState>,
    AuthBearer(user_email): AuthBearer,
    Json(payload): Json<CreateBoardPayload>,
) -> impl IntoResponse {
    if let Err(e) = require_team_membership(&state.db, &user_email, &payload.team).await {
        return (e.to_status_code(), Json(e.to_error_response())).into_response();
    }

    match create_board(payload.team, payload.board_name, &state.db).await {
        Ok(board) => (StatusCode::CREATED, Json(board)).into_response(),
        Err(e) => (e.to_status_code(), Json(e.to_error_response())).into_response(),
    }
}

async fn handle_update_board(
    State(state): State<AppState>,
    AuthBearer(user_email): AuthBearer,
    Json(payload): Json<UpdateBoardQuery>,
) -> impl IntoResponse {
    let board_id = match &payload.board.id {
        Some(id) => id.to_hex(),
        None => {
            let err = crate::utils::errors::CustomError::NotFound("Board ID required".to_string());
            return (err.to_status_code(), Json(err.to_error_response())).into_response();
        }
    };

    if let Err(e) = require_board_write_permission(&state.db, &user_email, &board_id).await {
        return (e.to_status_code(), Json(e.to_error_response())).into_response();
    }

    match update_board(payload.board, &state.db).await {
        Ok(board) => (StatusCode::OK, Json(board)).into_response(),
        Err(e) => (e.to_status_code(), Json(e.to_error_response())).into_response(),
    }
}

async fn handle_delete_board(
    State(state): State<AppState>,
    AuthBearer(user_email): AuthBearer,
    Query(payload): Query<BoardIdQuery>,
) -> impl IntoResponse {
    let board = match check_board_access(&state.db, &user_email, &payload.board_id).await {
        Ok(b) => b,
        Err(e) => return (e.to_status_code(), Json(e.to_error_response())).into_response(),
    };

    if let Err(e) = require_team_leader(&state.db, &user_email, &board.team).await {
        return (e.to_status_code(), Json(e.to_error_response())).into_response();
    }

    match delete_board(payload.board_id, &state.db).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (e.to_status_code(), Json(e.to_error_response())).into_response(),
    }
}
