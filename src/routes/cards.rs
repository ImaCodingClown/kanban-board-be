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
    services::cards::{add_card, delete_card, edit_card, get_columns},
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/v1/card", post(handle_add_card))
        .route("/v1/columns", get(handle_get_columns))
        .route("/v1/card/delete", post(handle_delete_card))
        .route("/v1/card/edit", post(handle_edit_card))
}

pub async fn handle_add_card(
    State(state): State<AppState>,
    Json(payload): Json<AddCardPayload>,
) -> impl IntoResponse {
    match add_card(payload, &state).await {
        Ok(card) => (StatusCode::CREATED, Json(card)).into_response(),
        Err(e) => (e.to_status_code(), Json(e.to_error_response())).into_response(),
    }
}

pub async fn handle_get_columns(
    State(state): State<AppState>,
    Query(query): Query<BoardIdQuery>,
) -> impl IntoResponse {
    match get_columns(&query.board_id, &state.db).await {
        Ok(columns) => (StatusCode::OK, Json(columns)).into_response(),
        Err(e) => (e.to_status_code(), Json(e.to_error_response())).into_response(),
    }
}

pub async fn handle_delete_card(
    State(state): State<AppState>,
    Json(payload): Json<DeleteCardPayload>,
) -> impl IntoResponse {
    match delete_card(payload, &state.db).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (e.to_status_code(), Json(e.to_error_response())).into_response(),
    }
}

pub async fn handle_edit_card(
    State(state): State<AppState>,
    Json(payload): Json<EditCardPayload>,
) -> impl IntoResponse {
    match edit_card(payload, &state).await {
        Ok(card) => (StatusCode::OK, Json(card)).into_response(),
        Err(e) => (e.to_status_code(), Json(e.to_error_response())).into_response(),
    }
}
