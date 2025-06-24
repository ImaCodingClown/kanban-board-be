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
    pub columns: Vec<Column>,
}

impl Board {
    pub fn new(team: String) -> Self {
        Self {
            id: None,
            team,
            iteration: None,
            columns: Vec::new(),
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
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Column {
    pub title: String,
    pub cards: Vec<Card>,
}

impl_mongo!(Board, "boards", "general");

impl MongoModel for Board {
    fn unique_query(&self) -> Document {
        doc! { "id": &self.id }
    }
}
