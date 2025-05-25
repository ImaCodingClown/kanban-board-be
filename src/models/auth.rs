use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

#[derive(Deserialize)]
pub struct AuthPayload {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthLoginPayload {
    pub user_or_email: String,
    pub password: String,
}