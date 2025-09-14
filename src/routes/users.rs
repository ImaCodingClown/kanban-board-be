use crate::config::AppState;
use crate::models::users::{UserPublic, UsersResponse};
use crate::services::user_info::get_all_users;
use crate::utils::jwt::AuthBearer;
use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Json, Router};

pub fn routes() -> Router<AppState> {
    Router::new().route("/users", get(handle_get_all_users))
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
