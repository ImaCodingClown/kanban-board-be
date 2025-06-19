use mongodb::bson::{doc, oid::ObjectId, Document};
use serde::{Deserialize, Serialize};

use crate::{
    db::mongo::{MongoModel, MongoService, ODM},
    impl_mongo,
};

#[serde_with::skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct Board {
    #[serde(rename = "_id")]
    pub id: Option<ObjectId>,
    pub team: String,
    pub iteration: Option<String>,
}

impl Board {
    pub fn new(team: String) -> Self {
        Self {
            id: None,
            team,
            iteration: None,
        }
    }
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct Card {
    #[serde(rename = "_id")]
    pub id: Option<ObjectId>,
    pub title: String,
    pub description: Option<String>,
    pub assignee: Option<String>,
    pub story_point: Option<u8>,
    pub priority: Option<String>,
    pub parent_column: Option<ObjectId>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Column {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub title: String,
    pub parent_board: Option<ObjectId>,
}

impl_mongo!(Card, "cards", "general");
impl_mongo!(Board, "boards", "general");
impl_mongo!(Column, "columns", "general");
impl MongoModel for Card {
    fn unique_query(&self) -> Document {
        doc! { "id": &self.id }
    }
}

impl MongoModel for Column {
    fn unique_query(&self) -> Document {
        doc! { "id": &self.id }
    }
}

impl MongoModel for Board {
    fn unique_query(&self) -> Document {
        doc! { "id": &self.id }
    }
}
