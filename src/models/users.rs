use mongodb::bson::{doc, oid::ObjectId, Document};
use serde::{Deserialize, Serialize};

use crate::{
    db::mongo::{MongoModel, MongoService, ODM},
    impl_mongo,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub group: Vec<String>,
    pub permissions: Vec<String>,
    pub teams: Vec<String>,
    pub slack_user_id: Option<String>,
}

impl User {
    pub fn create(
        username: String,
        email: String,
        password_hash: String,
        teams: Vec<String>,
    ) -> Self {
        User {
            id: None,
            username,
            email,
            password_hash,
            group: Vec::new(),
            permissions: Vec::new(),
            teams,
            slack_user_id: None,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct UserPublic {
    pub id: String,
    pub username: String,
    pub email: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub is_active: bool,
}

#[derive(Debug, Serialize)]
pub struct UsersResponse {
    pub success: bool,
    pub users: Vec<UserPublic>,
    pub message: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct UpdateSlackIdPayload {
    pub slack_user_id: String,
}

impl From<User> for UserPublic {
    fn from(user: User) -> Self {
        UserPublic {
            id: user.id.unwrap().to_hex(),
            username: user.username,
            email: Some(user.email),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            is_active: true,
        }
    }
}

impl MongoModel for User {
    fn unique_query(&self) -> Document {
        doc! { "username": &self.username, "email": &self.email }
    }
}
impl_mongo!(User, "users", "general");
