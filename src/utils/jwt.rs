use crate::models::auth::Claims;
use jsonwebtoken::{encode, EncodingKey, Header};

pub fn create_jwt(user_id: &str, secret: &str) -> String {
    let claims = Claims {
        sub: user_id.to_owned(),
        exp: 200000,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .unwrap()
}
