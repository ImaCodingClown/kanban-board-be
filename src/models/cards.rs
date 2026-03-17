use mongodb::bson::{doc, oid::ObjectId, Document};
use serde::{Deserialize, Serialize};

use crate::{
    db::mongo::{MongoModel, MongoService, ODM},
    impl_mongo,
};

#[serde_with::skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Board {
    #[serde(
        rename = "_id",
        serialize_with = "crate::utils::mongo_serializers::opt_oid_to_str"
    )]
    pub id: Option<ObjectId>,
    pub team: String,
    pub board_name: String,
    pub iteration: Option<String>,
    pub columns: Vec<Column>,
}

impl Board {
    pub fn create_default(team: String, board_name: String) -> Self {
        Self {
            id: None,
            team,
            board_name: board_name.clone(),
            iteration: Some(board_name),
            columns: vec![
                Column {
                    title: "To Do".to_string(),
                    cards: vec![],
                },
                Column {
                    title: "In Progress".to_string(),
                    cards: vec![],
                },
                Column {
                    title: "Done".to_string(),
                    cards: vec![],
                },
            ],
        }
    }
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Card {
    #[serde(
        rename = "_id",
        serialize_with = "crate::utils::mongo_serializers::opt_oid_to_str"
    )]
    pub id: Option<ObjectId>,
    pub card_id: Option<String>,
    pub card_number: Option<u32>,
    pub board_id: String,
    pub title: String,
    pub description: Option<String>,
    pub assignee: Option<String>,
    pub story_point: Option<u8>,
    pub priority: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct AddCardPayload {
    pub title: String,
    pub description: Option<String>,
    pub column_name: String,
    pub story_point: Option<u8>,
    pub assignee: Option<String>,
    pub board_id: String,
    pub priority: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct DeleteCardPayload {
    pub card_id: String,
    pub column_name: String,
    pub board_id: String,
}

#[derive(Deserialize, Debug)]
pub struct EditCardPayload {
    pub card_id: String,
    pub title: String,
    pub description: Option<String>,
    pub column_name: String,
    pub new_column_name: Option<String>,
    pub story_point: Option<u8>,
    pub assignee: Option<String>,
    pub board_id: String,
    pub priority: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Column {
    pub title: String,
    pub cards: Vec<Card>,
}

#[derive(Deserialize)]
pub struct TeamQuery {
    pub team: String,
}

#[derive(Deserialize)]
pub struct BoardIdQuery {
    pub board_id: String,
}

impl_mongo!(Board, "boards", "general");

impl MongoModel for Board {
    fn unique_query(&self) -> Document {
        doc! { "_id": &self.id }
    }
}
