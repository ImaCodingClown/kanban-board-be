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
        }
    }
}

impl MongoModel for User {
    fn unique_query(&self) -> Document {
        doc! { "username": &self.username, "email": &self.email }
    }
}
impl_mongo!(User, "users", "general");
