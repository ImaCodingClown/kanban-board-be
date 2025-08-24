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
        println!("JWT validation: Checking Authorization header");
        
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .ok_or_else(|| {
                println!("JWT validation: Missing Authorization header");
                (StatusCode::UNAUTHORIZED, "Missing token".into())
            })?;

        println!("JWT validation: Authorization header found: {}", auth_header);

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or_else(|| {
                println!("JWT validation: Invalid token format (missing 'Bearer ' prefix)");
                (StatusCode::UNAUTHORIZED, "Invalid token format".into())
            })?;

        println!("JWT validation: Token extracted: {}", token);

        let decoded = decode::<Claims>(
            token,
            &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|e| {
            println!("JWT validation: Token decode failed: {:?}", e);
            (StatusCode::UNAUTHORIZED, "Invalid or expired token".into())
        })?;

        println!("JWT validation: Token validated successfully for user: {}", decoded.claims.sub);
        Ok(AuthBearer(decoded.claims.sub))
    }
}
