use axum::Router;
use config::AppState;
use routes::{auth, board, health};
use tower_http::cors::{Any, CorsLayer};

mod config;
mod db;
mod models;
mod routes;
mod services;
mod utils;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let state = AppState::build().await;
    let app = create_app(state);

    println!("Server running at http://127.0.0.1:8080");

    let listener = tokio::net::TcpListener::bind(&"0.0.0.0:8080").await;
    axum::serve(listener.unwrap(), app.into_make_service())
        .await
        .unwrap();
    Ok(())
}

pub fn create_app(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);
    Router::new()
        .merge(auth::routes())
        .merge(board::routes())
        .merge(health::routes())
        .layer(cors)
        .with_state(state)
}
