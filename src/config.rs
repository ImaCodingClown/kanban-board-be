use std::sync::Arc;

use mongodb::Client;
use serde::{Deserialize, Serialize};
use strum_macros::EnumString;

#[derive(Serialize, Deserialize, Debug, Clone, EnumString)]
#[strum(serialize_all = "UPPERCASE")]
pub enum Environment {
    Dev,
    Prod,
    Test,
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub environment: Environment,
    pub db: Arc<Client>,
    pub jwt_secret: String,
}
