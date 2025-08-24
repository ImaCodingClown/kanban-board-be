use crate::config::AppState;
use crate::services::teams::{create_team, delete_team, update_user_teams};
use crate::utils::jwt::AuthBearer;
use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::post, Json, Router};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
pub struct UpdateTeamsPayload {
    pub teams: Vec<String>,
}

#[derive(Deserialize)]
pub struct CreateTeamPayload {
    pub team_name: String,
}

#[derive(Deserialize)]
pub struct DeleteTeamPayload {
    pub team_name: String,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/update", post(handle_update_teams))
        .route("/create", post(handle_create_team))
        .route("/delete", post(handle_delete_team))
}

async fn handle_update_teams(
    State(state): State<AppState>,
    AuthBearer(user_email): AuthBearer,
    Json(payload): Json<UpdateTeamsPayload>,
) -> impl IntoResponse {
    match update_user_teams(&state.db, &user_email, payload.teams).await {
        Ok(_) => (StatusCode::OK, Json(json!({ "success": true }))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e })),
        ),
    }
}

async fn handle_create_team(
    State(state): State<AppState>,
    AuthBearer(user_email): AuthBearer,
    Json(payload): Json<CreateTeamPayload>,
) -> impl IntoResponse {
    println!("Creating team '{}' for user: {}", payload.team_name, user_email);
    
    match create_team(&state.db, &user_email, &payload.team_name).await {
        Ok(teams) => {
            println!("Successfully created team. Updated teams: {:?}", teams);
            (
                StatusCode::CREATED,
                Json(json!({ "success": true, "teams": teams })),
            )
        },
        Err(e) => {
            println!("Failed to create team: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e })),
            )
        },
    }
}

async fn handle_delete_team(
    State(state): State<AppState>,
    AuthBearer(user_email): AuthBearer,
    Json(payload): Json<DeleteTeamPayload>,
) -> impl IntoResponse {
    if payload.team_name == "LJY Members" {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "Cannot delete the LJY Members team" })),
        );
    }

    match delete_team(&state.db, &user_email, &payload.team_name).await {
        Ok(teams) => (
            StatusCode::OK,
            Json(json!({ "success": true, "teams": teams })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e })),
        ),
    }
}
