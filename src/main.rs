use std::fmt::Error;

use axum::Router;
use config::AppState;
use routes::{auth, board, cards, health, teams, users};
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};

use crate::utils::errors::{self, CustomError};

mod config;
mod db;
mod models;
mod routes;
mod services;
mod utils;

#[tokio::main]
async fn main() -> Result<(), CustomError> {
    let state = AppState::build().await;

    let env_filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info")).map_err(|e| CustomError::Server(e.to_string()))?;

    if let (Ok(url), Ok(username), Ok(password)) = (
        std::env::var("LOKI_URL"),
        std::env::var("LOKI_USERNAME"),
        std::env::var("LOKI_PASSWORD"),
    ) {
        let mut loki_url = tracing_loki::url::Url::parse(&url).map_err(|e| CustomError::Server(e.to_string()))?;
        loki_url.set_username(&username).map_err(|e| CustomError::Server("Invalid loki credentials".to_string()))?;
        loki_url.set_password(Some(&password)).map_err(|e| CustomError::Server("Invalid loki credentials".to_string()))?;
        let (loki_layer, task) = tracing_loki::builder()
            .label("service", "kanban-api")?
            .label("environment", "production")?
            .build_url(loki_url)?;
        tokio::spawn(task);

        tracing_subscriber::registry()
            .with(env_filter)
            .with(loki_layer)
            .init();
    } else {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(tracing_subscriber::fmt::layer().json())
            .init();
            
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
        .nest("/v1", auth::routes())
        .nest("/v1", board::routes())
        .nest("/v1", cards::routes())
        .nest("/v1", users::routes())
        .nest("/v1/teams", teams::routes())
        .merge(health::routes())
        .layer(cors)
        .with_state(state)
}
