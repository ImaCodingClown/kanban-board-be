use axum::http::StatusCode;
use serde::Serialize;

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

#[derive(Debug)]
pub enum CustomError {
    Database(String),
    Authentication(String),
    Forbidden(String),
    Server(String),
    NotFound(String),
    Conflict(String),
    MongoError(mongodb::error::Error),
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
        }
    }

    pub fn to_error_response(&self) -> ErrorResponse {
        let (code, message, details, retry_after) = match self {
            CustomError::Database(msg) => {
                tracing::error!(status_code = 500, error = %msg, "Database error occurred");
                (
                    "DATABASE_ERROR".to_string(),
                    "Database operation failed".to_string(),
                    Some(msg.clone()),
                    None,
                )
            }
            CustomError::Authentication(msg) => {
                tracing::error!(status_code = 401, error = %msg, "Authentication error occurred");
                (
                    "AUTHENTICATION_ERROR".to_string(),
                    "Invalid email or password".to_string(),
                    Some(msg.clone()),
                    None,
                )
            }
            CustomError::Forbidden(msg) => {
                tracing::error!(status_code = 403, error = %msg, "Forbidden error occurred");
                (
                    "FORBIDDEN".to_string(),
                    "Access denied".to_string(),
                    Some(msg.clone()),
                    None,
                )
            }
            CustomError::Server(msg) => {
                tracing::error!(status_code = 500, error = %msg, "Server error occurred");
                (
                    "SERVER_ERROR".to_string(),
                    "Internal server error".to_string(),
                    Some(msg.clone()),
                    None,
                )
            }
            CustomError::NotFound(msg) => {
                tracing::error!(status_code = 404, error = %msg, "Not found error occurred");
                (
                    "NOT_FOUND".to_string(),
                    "Resource not found".to_string(),
                    Some(msg.clone()),
                    None,
                )
            }
            CustomError::Conflict(msg) => {
                tracing::error!(status_code = 409, error = %msg, "Conflict error occurred");
                (
                    "CONFLICT".to_string(),
                    "Resource conflict".to_string(),
                    Some(msg.clone()),
                    None,
                )
            }
            CustomError::MongoError(err) => {
                tracing::error!(status_code = 500, error = %err, "MongoDB error occurred");
                (
                    "DATABASE_ERROR".to_string(),
                    "Database operation failed".to_string(),
                    Some(err.to_string()),
                    None,
                )
            }
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

impl std::fmt::Display for CustomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CustomError::Database(msg) => write!(f, "Database error: {}", msg),
            CustomError::Authentication(msg) => write!(f, "Authentication error: {}", msg),
            CustomError::Forbidden(msg) => write!(f, "Forbidden: {}", msg),
            CustomError::Server(msg) => write!(f, "Server error: {}", msg),
            CustomError::NotFound(msg) => write!(f, "Not found: {}", msg),
            CustomError::Conflict(msg) => write!(f, "Conflict: {}", msg),
            CustomError::MongoError(err) => write!(f, "MongoDB error: {}", err),
        }
    }
}

impl std::error::Error for CustomError {}

impl From<String> for CustomError {
    fn from(err: String) -> Self {
        tracing::error!(status_code = 500, error = %err, "Server error occurred");
        CustomError::Server(err)
    }
}

impl From<&str> for CustomError {
    fn from(err: &str) -> Self {
        tracing::error!(status_code = 500, error = %err, "Server error occurred");
        CustomError::Server(err.to_string())
    }
}

impl From<mongodb::error::Error> for CustomError {
    fn from(err: mongodb::error::Error) -> Self {
        tracing::error!(status_code = 500, error = %err, "MongoDB error occurred");
        CustomError::MongoError(err)
    }
}

impl From<tracing_loki::Error> for CustomError {
    fn from(err: tracing_loki::Error) -> Self {
        tracing::error!(status_code = 500, error = %err, "Loki logging error occurred");
        CustomError::Server(format!("Loki logging error: {}", err))
    }
}
