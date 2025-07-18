use axum::{
    extract::{Json, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};

use crate::{
    config::AppState,
    models::cards::{AddCardPayload, TeamQuery},
    services::cards::{add_card, get_columns},
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/v1/card", post(handle_add_card))
        .route("/v1/columns", get(handle_get_columns))
}
pub async fn handle_add_card(
    State(state): State<AppState>,
    Json(payload): Json<AddCardPayload>,
) -> impl IntoResponse {
    println!("📥 [add_card] received payload: {:?}", payload);

    match add_card(payload, &state.db).await {
        Ok(card) => (StatusCode::CREATED, Json(card)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn handle_get_columns(
    State(state): State<AppState>,
    Query(query): Query<TeamQuery>,
) -> impl IntoResponse {
    match get_columns(&query.team, &state.db).await {
        Ok(columns) => (StatusCode::OK, Json(columns)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}
