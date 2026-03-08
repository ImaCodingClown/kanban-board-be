use axum::Router;
use config::AppState;
use dotenvy::dotenv;
use routes::{auth, board, cards, company, health, teams, users};
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::utils::errors::CustomError;

mod config;
mod db;
mod models;
mod routes;
mod services;
mod utils;

#[tokio::main]
async fn main() -> Result<(), CustomError> {
    dotenv().ok();
    let env_filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .map_err(|e| CustomError::from(e.to_string()))?;

    let mut loki_setup_failed = false;

    if let (Ok(url), Ok(username), Ok(password)) = (
        std::env::var("LOKI_URL"),
        std::env::var("LOKI_USERNAME"),
        std::env::var("LOKI_PASSWORD"),
    ) {
        let setup_result = (|| -> Result<(), CustomError> {
            let mut loki_url = tracing_loki::url::Url::parse(&url)
                .map_err(|e| CustomError::from(e.to_string()))?;
            loki_url
                .set_username(&username)
                .map_err(|_e| CustomError::from("Invalid loki credentials".to_string()))?;
            loki_url
                .set_password(Some(&password))
                .map_err(|_e| CustomError::from("Invalid loki credentials".to_string()))?;
            let environment = std::env::var("ENV").unwrap_or_else(|_| "production".to_string());
            let (loki_layer, task) = tracing_loki::builder()
                .label("service", "kanban-api")?
                .label("environment", &environment)?
                .build_url(loki_url)?;
            tokio::spawn(task);

            tracing_subscriber::registry()
                .with(env_filter.clone())
                .with(loki_layer)
                .init();
            Ok(())
        })();

        if let Err(err) = setup_result {
            tracing::warn!("Failed to set up Loki logging: {}", err);
            loki_setup_failed = true;
        }
    }

    if loki_setup_failed
        || std::env::var("LOKI_URL").is_err()
        || std::env::var("LOKI_USERNAME").is_err()
        || std::env::var("LOKI_PASSWORD").is_err()
    {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(tracing_subscriber::fmt::layer().json())
            .init();
        tracing::warn!("Loki logging is not configured. Falling back to console logging.");
    }
    let state = AppState::build().await;
    let app = create_app(state);

    let listener = tokio::net::TcpListener::bind(&"0.0.0.0:8080")
        .await
        .map_err(|e| CustomError::Server(e.to_string()))?;
    tracing::info!("Server is running on http://localhost:8080");
    axum::serve(listener, app.into_make_service())
        .await
        .map_err(|e| CustomError::Server(e.to_string()))?;
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
        .nest("/v1", company::routes()) 
        .merge(health::routes())
        .layer(cors)
        .with_state(state)
}
