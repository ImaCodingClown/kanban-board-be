use crate::config::AppState;
use crate::models::{
    AddMemberPayload, CompaniesResponse, CompanyResponse, CompanyWithUsernamesResponse,
    CreateCompanyPayload, RemoveMemberPayload, UpdateCompanyPayload,
};
use crate::services::{
    add_member, create_company, delete_company, get_company, get_company_with_usernames,
    get_user_companies, remove_member, update_company,
};
use crate::utils::jwt::AuthBearer;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/companies", post(handle_create_company))
        .route("/companies", get(handle_get_user_companies))
        .route("/companies/{company_id}", get(handle_get_company))
        .route("/companies/{company_id}", put(handle_update_company))
        .route("/companies/{company_id}", delete(handle_delete_company))
        .route(
            "/companies/{company_id}/with-usernames",
            get(handle_get_company_with_usernames),
        )
        .route("/companies/{company_id}/members", post(handle_add_member))
        .route(
            "/companies/{company_id}/members",
            delete(handle_remove_member),
        )
}

async fn handle_create_company(
    State(state): State<AppState>,
    AuthBearer(user_email): AuthBearer,
    Json(payload): Json<CreateCompanyPayload>,
) -> impl IntoResponse {
    match create_company(&state.db, &user_email, payload).await {
        Ok(company) => (
            StatusCode::CREATED,
            Json(CompanyResponse {
                success: true,
                company: Some(company),
                message: Some("Company created successfully".to_string()),
            }),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(CompanyResponse {
                success: false,
                company: None,
                message: Some(e.to_string()),
            }),
        ),
    }
}

async fn handle_get_company(
    State(state): State<AppState>,
    AuthBearer(_user_email): AuthBearer,
    Path(company_id): Path<String>,
) -> impl IntoResponse {
    match get_company(&state.db, &company_id).await {
        Ok(Some(company)) => (
            StatusCode::OK,
            Json(CompanyResponse {
                success: true,
                company: Some(company),
                message: Some("Company retrieved successfully".to_string()),
            }),
        ),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(CompanyResponse {
                success: false,
                company: None,
                message: Some("Company not found".to_string()),
            }),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CompanyResponse {
                success: false,
                company: None,
                message: Some(e.to_string()),
            }),
        ),
    }
}

async fn handle_get_user_companies(
    State(state): State<AppState>,
    AuthBearer(user_email): AuthBearer,
) -> impl IntoResponse {
    match get_user_companies(&state.db, &user_email).await {
        Ok(companies) => (
            StatusCode::OK,
            Json(CompaniesResponse {
                success: true,
                companies,
                message: Some("User companies retrieved successfully".to_string()),
            }),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CompaniesResponse {
                success: false,
                companies: vec![],
                message: Some(e.to_string()),
            }),
        ),
    }
}

async fn handle_update_company(
    State(state): State<AppState>,
    AuthBearer(user_email): AuthBearer,
    Path(company_id): Path<String>,
    Json(payload): Json<UpdateCompanyPayload>,
) -> impl IntoResponse {
    match update_company(&state.db, &user_email, &company_id, payload).await {
        Ok(company) => (
            StatusCode::OK,
            Json(CompanyResponse {
                success: true,
                company: Some(company),
                message: Some("Company updated successfully".to_string()),
            }),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(CompanyResponse {
                success: false,
                company: None,
                message: Some(e.to_string()),
            }),
        ),
    }
}

async fn handle_delete_company(
    State(state): State<AppState>,
    AuthBearer(user_email): AuthBearer,
    Path(company_id): Path<String>,
) -> impl IntoResponse {
    match delete_company(&state.db, &user_email, &company_id).await {
        Ok(()) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "message": "Company deleted successfully"
            })),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "success": false,
                "message": e.to_string()
            })),
        ),
    }
}

async fn handle_get_company_with_usernames(
    State(state): State<AppState>,
    AuthBearer(_user_email): AuthBearer,
    Path(company_id): Path<String>,
) -> impl IntoResponse {
    match get_company_with_usernames(&state.db, &company_id).await {
        Ok(company) => (
            StatusCode::OK,
            Json(CompanyWithUsernamesResponse {
                success: true,
                company: Some(company),
                message: Some("Company retrieved successfully".to_string()),
            }),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CompanyWithUsernamesResponse {
                success: false,
                company: None,
                message: Some(e.to_string()),
            }),
        ),
    }
}

async fn handle_add_member(
    State(state): State<AppState>,
    AuthBearer(user_email): AuthBearer,
    Path(company_id): Path<String>,
    Json(payload): Json<AddMemberPayload>,
) -> impl IntoResponse {
    match add_member(&state.db, &user_email, &company_id, &payload.user_id).await {
        Ok(company) => (
            StatusCode::OK,
            Json(CompanyResponse {
                success: true,
                company: Some(company),
                message: Some("Member added successfully".to_string()),
            }),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(CompanyResponse {
                success: false,
                company: None,
                message: Some(e.to_string()),
            }),
        ),
    }
}

async fn handle_remove_member(
    State(state): State<AppState>,
    AuthBearer(user_email): AuthBearer,
    Path(company_id): Path<String>,
    Json(payload): Json<RemoveMemberPayload>,
) -> impl IntoResponse {
    match remove_member(&state.db, &user_email, &company_id, &payload.user_id).await {
        Ok(company) => (
            StatusCode::OK,
            Json(CompanyResponse {
                success: true,
                company: Some(company),
                message: Some("Member removed successfully".to_string()),
            }),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(CompanyResponse {
                success: false,
                company: None,
                message: Some(e.to_string()),
            }),
        ),
    }
}
