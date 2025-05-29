use crate::models::auth::Claims;
use jsonwebtoken::{encode, EncodingKey, Header};
use chrono::{Utc, Duration};

pub fn create_jwt(user_id: &str, secret: &str) -> String {
    let expiration = (Utc::now() + Duration::days(7)).timestamp() as usize; 

    let claims = Claims {
        sub: user_id.to_owned(),
        exp: expiration,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .unwrap()
}
