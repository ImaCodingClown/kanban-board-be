use crate::config::AppState;
use crate::models::auth::{AuthLoginPayload, AuthPayload, RefreshTokenPayload};
use crate::services::auth::{login, logout, refresh_access_token, signup};
use crate::services::user_info::get_user_by_email;
use crate::utils::jwt::AuthBearer;
use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde_json::json;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/signup", post(handle_signup))
        .route("/login", post(handle_login))
        .route("/refresh", post(handle_refresh))
        .route("/logout", post(handle_logout))
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
            "teams": user.teams,
        })),
        Err(_) => Json(serde_json::json!({ "error": "User not found" })),
    }
}

async fn handle_signup(
    State(state): State<AppState>,
    Json(payload): Json<AuthPayload>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match signup(
        payload.username,
        payload.email,
        payload.password,
        &state.db,
        &state.jwt_secret,
    )
    .await
    {
        Ok(auth_response) => Ok(Json(serde_json::json!({
            "success": true,
            "access_token": auth_response.access_token,
            "refresh_token": auth_response.refresh_token,
            "expires_in": auth_response.expires_in,
        }))),
        Err(e) => {
            let error_response = e.to_error_response();
            Err((e.to_status_code(), Json(json!(error_response))))
        }
    }
}

async fn handle_login(
    State(state): State<AppState>,
    Json(payload): Json<AuthLoginPayload>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match login(
        payload.user_or_email,
        payload.password,
        &state.db,
        &state.jwt_secret,
    )
    .await
    {
        Ok(auth_response) => Ok(Json(json!({
            "success": true,
            "access_token": auth_response.access_token,
            "refresh_token": auth_response.refresh_token,
            "expires_in": auth_response.expires_in,
        }))),
        Err(e) => {
            let error_response = e.to_error_response();
            Err((e.to_status_code(), Json(json!(error_response))))
        }
    }
}

async fn handle_refresh(
    State(state): State<AppState>,
    Json(payload): Json<RefreshTokenPayload>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match refresh_access_token(payload.refresh_token, &state.db, &state.jwt_secret).await {
        Ok(auth_response) => Ok(Json(json!({
            "success": true,
            "access_token": auth_response.access_token,
            "refresh_token": auth_response.refresh_token,
            "expires_in": auth_response.expires_in,
        }))),
        Err(e) => {
            let error_response = e.to_error_response();
            Err((e.to_status_code(), Json(json!(error_response))))
        }
    }
}

async fn handle_logout(
    State(state): State<AppState>,
    AuthBearer(user_email): AuthBearer,
    Json(payload): Json<RefreshTokenPayload>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match logout(&user_email, &payload.refresh_token, &state.db).await {
        Ok(_) => Ok(Json(json!({
            "success": true,
            "message": "Logged out successfully"
        }))),
        Err(e) => {
            let error_response = e.to_error_response();
            Err((e.to_status_code(), Json(json!(error_response))))
        }
    }
}
