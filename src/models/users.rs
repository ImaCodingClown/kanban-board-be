use async_trait::async_trait;
use mongodb::{
    bson::{doc, oid::ObjectId, Document},
    Client, Collection,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::db::mongo::{MongoModel, MongoService, ODM};

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
    pub fn create(username: String, email: String, password_hash: String) -> Self {
        User {
            id: None,
            username,
            email,
            password_hash,
            group: Vec::new(),
            permissions: Vec::new(),
            teams: Vec::new(),
        }
    }
}

impl MongoModel for User {
    fn unique_query(&self) -> Document {
        doc! { "username": &self.username, "email": &self.email }
    }
}

#[async_trait]
impl MongoService<User> for ODM<User> {
    const COLLECTION: &str = "users";
    const DATABASE: &str = "general";

    async fn build(client: &Client) -> Self {
        let collection: Collection<User> =
            client.database(Self::DATABASE).collection(Self::COLLECTION);
        ODM::<User> {
            client: Arc::new(client.clone()),
            collection,
        }
    }
}
