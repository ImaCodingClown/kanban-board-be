use crate::{
    config::AppState,
    models::users::{UpdateSlackIdPayload, UserPublic, UsersResponse},
    services::user_info::{get_all_users, update_user_slack_id},
    utils::jwt::AuthBearer,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, patch},
    Json, Router,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/users", get(handle_get_all_users))
        .route("/v1/user/{user_id}/slack", patch(handle_update_slack_id))
}

async fn handle_get_all_users(
    State(state): State<AppState>,
    AuthBearer(_user_email): AuthBearer,
) -> impl IntoResponse {
    match get_all_users(&state.db).await {
        Ok(users) => {
            let public_users: Vec<UserPublic> = users.into_iter().map(UserPublic::from).collect();
            (
                StatusCode::OK,
                Json(UsersResponse {
                    success: true,
                    users: public_users,
                    message: Some("Users retrieved successfully".to_string()),
                }),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(UsersResponse {
                success: false,
                users: vec![],
                message: Some(e),
            }),
        ),
    }
}

async fn handle_update_slack_id(
    State(state): State<AppState>,
    AuthBearer(_user_email): AuthBearer,
    Path(user_id): Path<String>,
    Json(payload): Json<UpdateSlackIdPayload>,
) -> impl IntoResponse {
    match update_user_slack_id(&user_id, payload, &state.db).await {
        Ok(user) => {
            let public_user = UserPublic::from(user);
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "success": true,
                    "user": public_user,
                    "message": "Slack ID updated successfully"
                })),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "success": false,
                "error": e.to_string()
            })),
        )
            .into_response(),
    }
}
