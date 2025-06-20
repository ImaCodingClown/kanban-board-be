use crate::config::AppState;
use crate::models::auth::{AuthLoginPayload, AuthPayload};
use crate::services::auth::{login, signup};
use crate::services::user_info::get_user_by_email;
use crate::utils::jwt::AuthBearer;
use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use serde_json::json;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/signup", post(handle_signup))
        .route("/login", post(handle_login))
        .route("/me", get(handle_get_me))
}

pub async fn handle_get_me(
    State(state): State<AppState>,
    AuthBearer(user_email): AuthBearer,
) -> Json<serde_json::Value> {
    let db = state.db.database("general");

    match get_user_by_email(&db, &user_email).await {
        Ok(user) => Json(serde_json::json!({
            "id": user.id,
            "username": user.username,
            "email": user.email,
        })),
        Err(_) => Json(serde_json::json!({ "error": "User not found" })),
    }
}

async fn handle_signup(
    State(state): State<AppState>,
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
        Err(e) => Json(serde_json::json!({ "error": format!("{e}") })),
    }
}

async fn handle_login(
    State(state): State<AppState>,
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
        Ok(token) => Json(json!({ "token": token })),
        Err(e) => Json(json!({ "error": e })),
    }
}
