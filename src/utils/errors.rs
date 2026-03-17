use axum::http::StatusCode;
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub success: bool,
    pub error: ErrorDetail,
    pub timestamp: i64,
}

#[derive(Debug, Serialize)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
    pub details: Option<String>,
    pub retry_after: Option<u64>,
}

#[derive(Debug, Error)]
pub enum CustomError {
    #[error("Database error: {0}")]
    Database(String),
    #[error("Authentication error: {0}")]
    Authentication(String),
    #[error("Forbidden: {0}")]
    Forbidden(String),
    #[error("Server error: {0}")]
    Server(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Conflict: {0}")]
    Conflict(String),
    #[error("MongoDB error: {0}")]
    MongoError(#[from] mongodb::error::Error),
    #[error("ObjectId parsing error: {0}")]
    OIDParseError(#[from] mongodb::bson::oid::Error),
}

impl CustomError {
    pub fn to_status_code(&self) -> StatusCode {
        match self {
            CustomError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            CustomError::Authentication(_) => StatusCode::UNAUTHORIZED,
            CustomError::Forbidden(_) => StatusCode::FORBIDDEN,
            CustomError::Server(_) => StatusCode::INTERNAL_SERVER_ERROR,
            CustomError::NotFound(_) => StatusCode::NOT_FOUND,
            CustomError::Conflict(_) => StatusCode::CONFLICT,
            CustomError::MongoError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            CustomError::OIDParseError(_) => StatusCode::BAD_REQUEST,
        }
    }

    pub fn to_error_response(&self) -> ErrorResponse {
        let (code, message, details, retry_after) = match self {
            CustomError::Database(msg) => (
                "DATABASE_ERROR".to_string(),
                "Database operation failed".to_string(),
                Some(msg.clone()),
                None,
            ),
            CustomError::Authentication(msg) => (
                "AUTHENTICATION_ERROR".to_string(),
                "Invalid email or password".to_string(),
                Some(msg.clone()),
                None,
            ),
            CustomError::Forbidden(msg) => (
                "FORBIDDEN".to_string(),
                "Access denied".to_string(),
                Some(msg.clone()),
                None,
            ),
            CustomError::Server(msg) => (
                "SERVER_ERROR".to_string(),
                "Internal server error".to_string(),
                Some(msg.clone()),
                None,
            ),
            CustomError::NotFound(msg) => (
                "NOT_FOUND".to_string(),
                "Resource not found".to_string(),
                Some(msg.clone()),
                None,
            ),
            CustomError::Conflict(msg) => (
                "CONFLICT".to_string(),
                "Resource conflict".to_string(),
                Some(msg.clone()),
                None,
            ),
            CustomError::MongoError(err) => (
                "DATABASE_ERROR".to_string(),
                "Database operation failed".to_string(),
                Some(err.to_string()),
                None,
            ),
            CustomError::OIDParseError(err) => (
                "INVALID_ID".to_string(),
                "Invalid ID format".to_string(),
                Some(err.to_string()),
                None,
            ),
        };

        ErrorResponse {
            success: false,
            error: ErrorDetail {
                code,
                message,
                details,
                retry_after,
            },
            timestamp: chrono::Utc::now().timestamp(),
        }
    }
}

impl From<String> for CustomError {
    fn from(err: String) -> Self {
        CustomError::Server(err)
    }
}

impl From<&str> for CustomError {
    fn from(err: &str) -> Self {
        CustomError::from(err.to_string())
    }
}

impl From<tracing_loki::Error> for CustomError {
    fn from(err: tracing_loki::Error) -> Self {
        tracing::error!(status_code = 500, error = %err, "Loki logging error occurred");
        CustomError::Server(format!("Loki logging error: {}", err))
    }
}

impl From<CustomError> for &CustomError {
    fn from(err: CustomError) -> Self {
        Box::leak(Box::new(err))
    }
}
