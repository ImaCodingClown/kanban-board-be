use std::sync::Arc;

use mongodb::Client;

#[derive(Debug, Clone)]
pub struct AppState {
    pub db: Arc<Client>,
    pub jwt_secret: String,
}
