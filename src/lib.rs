pub mod config;
pub mod db;
pub mod models;
pub mod routes;
pub mod services;
pub mod utils;

use axum::Router;
use config::AppState;
use routes::{auth, board, health};
use tower_http::cors::{Any, CorsLayer};

pub fn create_app(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .nest("/auth", auth::routes())
        .nest("/board", board::routes())
        .merge(health::routes())
        .with_state(state)
        .layer(cors)
}
