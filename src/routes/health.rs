use axum::response::Response;
use axum::routing::get;
use axum::Router;

use crate::config::AppState;

pub fn routes() -> Router<AppState> {
    Router::new().route("/health", get(handle_check_health))
}

async fn handle_check_health() -> Response<String> {
    Response::new("Good!".to_string())
}
