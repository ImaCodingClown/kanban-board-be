use std::{env, str::FromStr, sync::Arc};

use dotenvy::dotenv;
use mongodb::Client;
use serde::{Deserialize, Serialize};
use strum_macros::EnumString;

use crate::db;

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

impl AppState {
    pub async fn build() -> Self {
        dotenv().ok();

        let environment = Environment::from_str(&env::var("ENV").unwrap_or("DEV".to_string()))
            .unwrap_or(Environment::Dev);
        let db_uri = match environment {
            Environment::Test => "".to_string(),
            _ => env::var("MONGO_URI").expect("MongoURI not set"),
        };
        let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET not set");

        let db = db::mongo::db_client(&db_uri).await;
        AppState {
            environment,
            db,
            jwt_secret,
        }
    }
}
