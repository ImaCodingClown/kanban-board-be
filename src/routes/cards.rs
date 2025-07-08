use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Router,
};

use crate::{
    config::AppState,
    services::cards::add_card,
    models::cards::AddCardPayload,
};

pub fn routes() -> Router<AppState> {
    Router::new().route("/v1/card", post(handle_add_card))
}

async fn handle_add_card(
    State(state): State<AppState>,
    Json(payload): Json<AddCardPayload>,
) -> impl IntoResponse {
    println!("Add card called");
    match add_card(payload, &state.db).await {
        Ok(card) => (StatusCode::CREATED, Json(card)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}
