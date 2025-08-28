use crate::config::AppState;
use crate::models::teams::{CreateTeamPayload, UpdateTeamPayload, AddMemberPayload, RemoveMemberPayload, TeamResponse, TeamsResponse};
use crate::services::teams::{create_team, get_team, get_user_teams, update_team, add_member, remove_member, leave_team, delete_team};
use crate::utils::jwt::AuthBearer;
use axum::{extract::{Path, State}, http::StatusCode, response::IntoResponse, routing::{get, post, put, delete}, Json, Router};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", post(handle_create_team))
        .route("/", get(handle_get_user_teams))
        .route("/{team_name}", get(handle_get_team))
        .route("/{team_name}", put(handle_update_team))
        .route("/{team_name}", delete(handle_delete_team))
        .route("/{team_name}/members", post(handle_add_member))
        .route("/{team_name}/members", delete(handle_remove_member))
        .route("/{team_name}/leave", post(handle_leave_team))
}

async fn handle_create_team(
    State(state): State<AppState>,
    AuthBearer(user_email): AuthBearer,
    Json(payload): Json<CreateTeamPayload>,
) -> impl IntoResponse {
    match create_team(&state.db, &user_email, payload).await {
        Ok(team) => (
            StatusCode::CREATED,
            Json(TeamResponse {
                success: true,
                team: Some(team),
                message: Some("Team created successfully".to_string()),
            }),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(TeamResponse {
                success: false,
                team: None,
                message: Some(e),
            }),
        ),
    }
}

async fn handle_get_team(
    State(state): State<AppState>,
    AuthBearer(_user_email): AuthBearer,
    Path(team_name): Path<String>,
) -> impl IntoResponse {
    match get_team(&state.db, &team_name).await {
        Ok(Some(team)) => (
            StatusCode::OK,
            Json(TeamResponse {
                success: true,
                team: Some(team),
                message: Some("Team retrieved successfully".to_string()),
            }),
        ),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(TeamResponse {
                success: false,
                team: None,
                message: Some("Team not found".to_string()),
            }),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(TeamResponse {
                success: false,
                team: None,
                message: Some(e),
            }),
        ),
    }
}

async fn handle_get_user_teams(
    State(state): State<AppState>,
    AuthBearer(user_email): AuthBearer,
) -> impl IntoResponse {
    match get_user_teams(&state.db, &user_email).await {
        Ok(teams) => (
            StatusCode::OK,
            Json(TeamsResponse {
                success: true,
                teams,
                message: Some("User teams retrieved successfully".to_string()),
            }),
        
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(TeamsResponse {
                success: false,
                teams: vec![],
                message: Some(e),
            }),
        ),
    }
}

async fn handle_update_team(
    State(state): State<AppState>,
    AuthBearer(user_email): AuthBearer,
    Path(team_name): Path<String>,
    Json(payload): Json<UpdateTeamPayload>,
) -> impl IntoResponse {
    match update_team(&state.db, &user_email, &team_name, payload).await {
        Ok(team) => (
            StatusCode::OK,
            Json(TeamResponse {
                success: true,
                team: Some(team),
                message: Some("Team updated successfully".to_string()),
            }),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(TeamResponse {
                success: false,
                team: None,
                message: Some(e),
            }),
        ),
    }
}

async fn handle_add_member(
    State(state): State<AppState>,
    AuthBearer(user_email): AuthBearer,
    Path(team_name): Path<String>,
    Json(payload): Json<AddMemberPayload>,
) -> impl IntoResponse {
    match add_member(&state.db, &user_email, &team_name, payload).await {
        Ok(team) => (
            StatusCode::OK,
            Json(TeamResponse {
                success: true,
                team: Some(team),
                message: Some("Member added successfully".to_string()),
            }),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(TeamResponse {
                success: false,
                team: None,
                message: Some(e),
            }),
        ),
    }
}

async fn handle_remove_member(
    State(state): State<AppState>,
    AuthBearer(user_email): AuthBearer,
    Path(team_name): Path<String>,
    Json(payload): Json<RemoveMemberPayload>,
) -> impl IntoResponse {
    match remove_member(&state.db, &user_email, &team_name, payload).await {
        Ok(team) => (
            StatusCode::OK,
            Json(TeamResponse {
                success: true,
                team: Some(team),
                message: Some("Member removed successfully".to_string()),
            }),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(TeamResponse {
                success: false,
                team: None,
                message: Some(e),
            }),
        ),
    }
}

async fn handle_leave_team(
    State(state): State<AppState>,
    AuthBearer(user_email): AuthBearer,
    Path(team_name): Path<String>,
) -> impl IntoResponse {
    match leave_team(&state.db, &user_email, &team_name).await {
        Ok(()) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "message": "Successfully left the team"
            })),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "success": false,
                "message": e
            })),
        ),
    }
}

async fn handle_delete_team(
    State(state): State<AppState>,
    AuthBearer(user_email): AuthBearer,
    Path(team_name): Path<String>,
) -> impl IntoResponse {
    match delete_team(&state.db, &user_email, &team_name).await {
        Ok(()) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "message": "Team deleted successfully"
            })),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "success": false,
                "message": e
            })),
        ),
    }
}