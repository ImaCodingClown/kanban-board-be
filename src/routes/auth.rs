use axum::{routing::post, Json, Router};

use crate::config::AppState;
use crate::models::auth::AuthLoginPayload;
use crate::models::auth::AuthPayload;
use crate::services::auth::{login, signup};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/signup", post(handle_signup))
        .route("/login", post(handle_login))
}

async fn handle_signup(
    state: axum::extract::State<AppState>,
    Json(payload): Json<AuthPayload>,
) -> Json<serde_json::Value> {
    match signup(
        payload.username,
        payload.email,
        payload.password,
        &state.db,
        &state.jwt_secret,
    )
    .await
    {
        Ok(token) => Json(serde_json::json!({ "token": token })),
        Err(e) => Json(serde_json::json!({ "error": e })),
    }
}

async fn handle_login(
    state: axum::extract::State<AppState>,
    Json(payload): Json<AuthLoginPayload>,
) -> Json<serde_json::Value> {
    match login(
        payload.user_or_email,
        payload.password,
        &state.db,
        &state.jwt_secret,
    )
    .await
    {
        Ok(token) => Json(serde_json::json!({ "token": token })),
        Err(e) => Json(serde_json::json!({ "error": e })),
    }
}
