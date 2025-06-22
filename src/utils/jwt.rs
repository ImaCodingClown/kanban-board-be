use crate::config::AppState;
use crate::models::auth::Claims;
use axum::extract::FromRequestParts;
use axum::http::{request::Parts, StatusCode};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};

pub struct JWTValidator {}

pub trait JWTMethods {
    fn create_jwt(user_email: &str, secret: &str) -> String {
        let claims = Claims {
            sub: user_email.to_owned(),
            exp: (Utc::now() + Duration::hours(1)).timestamp() as usize,
        };
        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .unwrap()
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
            .ok_or((StatusCode::UNAUTHORIZED, "Missing token".into()))?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or((StatusCode::UNAUTHORIZED, "Invalid token format".into()))?;

        let decoded = decode::<Claims>(
            token,
            &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid or expired token".into()))?;

        Ok(AuthBearer(decoded.claims.sub))
    }
}
