use axum::{
  extract::{Json, Path, State},
  http::StatusCode,
};
use crate::models::cards::{Card, Column, AddCard};
use crate::config::AppState;
use uuid::Uuid;

pub async fn add_card(
  State(_state): State<AppState>,
  Path(column_id): Path<Uuid>,
  Json(payload): Json<AddCard>,
) -> Result<Card, StatusCode> {

  let new_card = Card {
      id: Uuid::new_v4(),
      title: payload.title,
      description: payload.description,
      assignee: payload.assignee,
      story_point: payload.story_point,
      priority: payload.priority,
  };

   let dummy_column = Column {
      id: column_id,
      title: "To do".to_string(),
      cards: vec![new_card.clone()],
  };

  println!("test column with new card: {:#?}", dummy_column);


  Ok(new_card)
}