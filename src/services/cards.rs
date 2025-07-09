use crate::{
    db::mongo::{MongoService, ODM},
    models::cards::{AddCardPayload, Board, Card},
    utils::errors::CustomError,
};
use mongodb::{bson::oid::ObjectId, Client};

pub async fn add_card(payload: AddCardPayload, db: &Client) -> Result<Card, CustomError> {
    println!("Add card hit");
    let board_service = ODM::<Board>::build(db).await;
    let mut boards = board_service
        .fetch_many_by_team(&payload.team.clone())
        .await?;

    let board = boards
        .get_mut(0)
        .ok_or_else(|| CustomError::CustomError("Board not found".to_string()))?;

    // Find the column
    let col = board
        .columns
        .iter_mut()
        .find(|c| c.title == payload.column_name)
        .ok_or_else(|| CustomError::CustomError("Column not found".to_string()))?;

    // Create and insert the card
    let card = Card {
        id: Some(ObjectId::new()),
        title: payload.title,
        description: payload.description,
        assignee: None,
        story_point: None,
        priority: None,
    };

    col.cards.push(card.clone());
    
    board_service.save_one(board).await?;

    Ok(card)
}
