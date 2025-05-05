use std::{env, net::TcpListener};

use axum::{routing::get, Router};
use config::AppState;
use dotenvy::dotenv;
use routes::{auth, board};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

mod config;
mod db;
mod models;
mod routes;
mod services;
mod utils;

// Models
#[derive(Serialize, Deserialize)]
struct Card {
    id: Uuid,
    title: String,
}

#[derive(Serialize, Deserialize)]
struct Column {
    id: Uuid,
    title: String,
    cards: Vec<Card>,
}

// API Handler
async fn get_board() -> Result<Vec<Column>, String> {
    let board = vec![
        Column {
            id: Uuid::new_v4(),
            title: "To Do".to_string(),
            cards: vec![
                Card {
                    id: Uuid::new_v4(),
                    title: "Learn Rust".to_string(),
                },
                Card {
                    id: Uuid::new_v4(),
                    title: "Build a Kanban app".to_string(),
                },
            ],
        },
        Column {
            id: Uuid::new_v4(),
            title: "In Progress".to_string(),
            cards: vec![],
        },
        Column {
            id: Uuid::new_v4(),
            title: "Done".to_string(),
            cards: vec![],
        },
    ];

    Ok(board)
}

// Main
#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let db_uri = env::var("MONGO_URI").expect("MongoURI not set");
    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET not set");

    let db = db::mongo::db_client(&db_uri).await;
    let state = AppState { db, jwt_secret };
    let app = create_app(state);

    println!("Server running at http://127.0.0.1:8080");

    let listener = tokio::net::TcpListener::bind(&"0.0.0.0:8080").await;
    axum::serve(listener.unwrap(), app.into_make_service())
        .await
        .unwrap();
    Ok(())
}

pub fn create_app(state: AppState) -> Router {
    Router::new()
        .merge(auth::routes())
        .merge(board::routes())
        .with_state(state)
}
