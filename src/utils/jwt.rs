use axum::extract::FromRequestParts;
use axum::http::{request::Parts, StatusCode};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use chrono::{Duration, Utc};
use crate::models::auth::Claims;

pub fn create_jwt(user_id: &str, secret: &str) -> String {
    let claims = Claims {
        sub: user_id.to_owned(),
        exp: (Utc::now() + Duration::hours(1)).timestamp() as usize,

    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .unwrap()
}

pub struct AuthBearer(pub String);

impl<S> FromRequestParts<S> for AuthBearer
where
    S: Send + Sync + 'static,
{
    type Rejection = (StatusCode, String);

    fn from_request_parts<'a>(
        parts: &'a mut Parts,
        _state: &S,
    ) -> impl std::future::Future<Output = Result<Self, <Self as FromRequestParts<S>>::Rejection>> + Send {
        Box::pin(async move {
            let auth_header = parts
                .headers
                .get("Authorization")
                .and_then(|h| h.to_str().ok())
                .ok_or((StatusCode::UNAUTHORIZED, "Missing token".into()))?;

            let token = auth_header
                .strip_prefix("Bearer ")
                .ok_or((StatusCode::UNAUTHORIZED, "Invalid token format".into()))?;

            let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "secret".into());

            let decoded = decode::<Claims>(
                token,
                &DecodingKey::from_secret(secret.as_bytes()),
                &Validation::default(),
            )
            .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid or expired token".into()))?;

            Ok(AuthBearer(decoded.claims.sub))
        })
    }
}
