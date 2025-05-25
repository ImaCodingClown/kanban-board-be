use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Card {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub assignee: Option<String>,
    pub story_point: Option<u8>,
    pub priority: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Column {
    pub id: Uuid,
    pub title: String,
    pub cards: Vec<Card>,
}
