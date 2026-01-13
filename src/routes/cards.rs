use axum::{
    extract::{Json, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};

use crate::{
    config::AppState,
    models::cards::{AddCardPayload, BoardIdQuery, DeleteCardPayload, EditCardPayload},
    services::{
        cards::{add_card, delete_card, edit_card, get_columns},
        permission::{check_board_access, require_board_write_permission},
    },
    utils::jwt::AuthBearer,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/card", post(handle_add_card))
        .route("/columns", get(handle_get_columns))
        .route("/card/delete", post(handle_delete_card))
        .route("/card/edit", post(handle_edit_card))
}

pub async fn handle_add_card(
    State(state): State<AppState>,
    AuthBearer(user_email): AuthBearer,
    Json(payload): Json<AddCardPayload>,
) -> impl IntoResponse {
    if let Err(e) = require_board_write_permission(&state.db, &user_email, &payload.board_id).await
    {
        return (e.to_status_code(), Json(e.to_error_response())).into_response();
    }

    match add_card(payload, &state).await {
        Ok(card) => (StatusCode::CREATED, Json(card)).into_response(),
        Err(e) => (e.to_status_code(), Json(e.to_error_response())).into_response(),
    }
}

pub async fn handle_get_columns(
    State(state): State<AppState>,
    AuthBearer(user_email): AuthBearer,
    Query(query): Query<BoardIdQuery>,
) -> impl IntoResponse {
    if let Err(e) = check_board_access(&state.db, &user_email, &query.board_id).await {
        return (e.to_status_code(), Json(e.to_error_response())).into_response();
    }

    match get_columns(&query.board_id, &state.db).await {
        Ok(columns) => (StatusCode::OK, Json(columns)).into_response(),
        Err(e) => (e.to_status_code(), Json(e.to_error_response())).into_response(),
    }
}

pub async fn handle_delete_card(
    State(state): State<AppState>,
    AuthBearer(user_email): AuthBearer,
    Json(payload): Json<DeleteCardPayload>,
) -> impl IntoResponse {
    if let Err(e) = require_board_write_permission(&state.db, &user_email, &payload.board_id).await
    {
        return (e.to_status_code(), Json(e.to_error_response())).into_response();
    }

    match delete_card(payload, &state.db).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (e.to_status_code(), Json(e.to_error_response())).into_response(),
    }
}

pub async fn handle_edit_card(
    State(state): State<AppState>,
    AuthBearer(user_email): AuthBearer,
    Json(payload): Json<EditCardPayload>,
) -> impl IntoResponse {
    if let Err(e) = require_board_write_permission(&state.db, &user_email, &payload.board_id).await
    {
        return (e.to_status_code(), Json(e.to_error_response())).into_response();
    }

    match edit_card(payload, &state).await {
        Ok(card) => (StatusCode::OK, Json(card)).into_response(),
        Err(e) => (e.to_status_code(), Json(e.to_error_response())).into_response(),
    }
}
