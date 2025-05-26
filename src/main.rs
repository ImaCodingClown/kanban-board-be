use std::env;
use std::str::FromStr;

use axum::Router;
use config::{AppState, Environment};
use dotenvy::dotenv;
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
    dotenv().ok();

    let environment = Environment::from_str(&env::var("ENV").unwrap_or("DEV".to_string()))
        .unwrap_or(Environment::Dev);
    let db_uri = match environment {
        Environment::Test => "".to_string(),
        _ => env::var("MONGO_URI").expect("MongoURI not set"),
    };
    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET not set");

    let db = db::mongo::db_client(&db_uri).await;
    let state = AppState {
        environment,
        db,
        jwt_secret,
    };
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
