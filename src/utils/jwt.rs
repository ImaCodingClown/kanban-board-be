use crate::config::AppState;
use crate::models::auth::Claims;
use axum::extract::FromRequestParts;
use axum::http::{request::Parts, StatusCode};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use sha2::{Digest, Sha256};
use uuid::Uuid;

pub struct JWTValidator {}

pub trait JWTMethods {
    fn create_jwt(user_email: &str, secret: &str) -> (String, String) {
        let refresh_token_id = Uuid::new_v4().to_string();
        let claims = Claims {
            sub: user_email.to_owned(),
            exp: (Utc::now() + Duration::hours(12)).timestamp() as usize,
            refresh_token_id: refresh_token_id.clone(),
        };

        let access_token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .unwrap();

        let refresh_token = Uuid::new_v4().to_string();

        (access_token, refresh_token)
    }

    fn create_access_token(user_email: &str, refresh_token_id: &str, secret: &str) -> String {
        let claims = Claims {
            sub: user_email.to_owned(),
            exp: (Utc::now() + Duration::hours(12)).timestamp() as usize,
            refresh_token_id: refresh_token_id.to_owned(),
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .unwrap()
    }

    fn hash_refresh_token(token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

impl JWTMethods for JWTValidator {}

pub struct AuthBearer(pub String);

impl FromRequestParts<AppState> for AuthBearer {
    type Rejection = (StatusCode, String);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .ok_or_else(|| {
                (
                    StatusCode::UNAUTHORIZED,
                    "Missing authorization header".into(),
                )
            })?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or_else(|| (StatusCode::UNAUTHORIZED, "Invalid token format".into()))?;

        let decoded = decode::<Claims>(
            token,
            &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|e| {
            let error_msg = match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => "Token expired",
                jsonwebtoken::errors::ErrorKind::InvalidToken => "Invalid token",
                jsonwebtoken::errors::ErrorKind::InvalidSignature => "Invalid signature",
                _ => "Token validation failed",
            };
            (StatusCode::UNAUTHORIZED, error_msg.into())
        })?;

        Ok(AuthBearer(decoded.claims.sub))
    }
}
