use crate::config::AppState;
use crate::models::cards::{AddCard};
use crate::services::column::add_card;
use axum::{
    extract::{State, Path, Json}, 
    http::StatusCode, 
    response::IntoResponse, 
    routing::post, 
    Router
};
use uuid::Uuid;

pub fn routes() -> Router<AppState> {
    Router::new()
    .route("/column/{column_id}/cards", post(handle_add_card))
}

async fn handle_add_card(
    state: State<AppState>,
    column_id: Path<Uuid>,
    payload: Json<AddCard>,
) -> impl IntoResponse {
    match add_card(state, column_id, payload).await {
        Ok(card) => (StatusCode::OK, Json(card)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}