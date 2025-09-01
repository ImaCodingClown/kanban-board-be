use axum::Router;
use config::AppState;
use routes::{auth, board, cards, health, teams};
use services::auth::create_performance_indexes;
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
    
    match create_performance_indexes(&state.db).await {
        Ok(_) => println!("Performance indexes created successfully"),
        Err(e) => eprintln!("Failed to create performance indexes: {}", e),
    }
    
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
        .merge(cards::routes())
        .nest("/teams", teams::routes())
        .layer(cors)
        .with_state(state)
}
