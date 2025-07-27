use crate::{
    db::mongo::{MongoService, ODM},
    models::cards::{AddCardPayload, DeleteCardPayload, Board, Card},
    utils::errors::CustomError,
};
use mongodb::{bson::oid::ObjectId, Client};

pub async fn add_card(payload: AddCardPayload, db: &Client) -> Result<Card, CustomError> {
    let board_service = ODM::<Board>::build(db).await;
    let mut boards = board_service
        //TODO: Replace LJY Members
        .fetch_many_by_team("LJY Members")
        .await?;

    let board = boards
        .get_mut(0)
        .ok_or_else(|| CustomError::CustomError("Board not found".to_string()))?;

    let col = board
        .columns
        .iter_mut()
        .find(|c| c.title == payload.column_name)
        .ok_or_else(|| CustomError::CustomError("Column not found".to_string()))?;

    let card = Card {
        id: Some(ObjectId::new()),
        title: payload.title,
        description: payload.description,
        assignee: None,
        story_point: None,
        priority: None,
    };

    col.cards.push(card.clone());

    board_service
        .replace_one(board, board.id.as_ref().unwrap())
        .await?;

    Ok(card)
}

pub async fn get_columns(team: &str, db: &Client) -> Result<Vec<String>, CustomError> {
    let board_service = ODM::<Board>::build(db).await;
    let boards = board_service.fetch_many_by_team(team).await?;

    let board = boards.first()
        .ok_or_else(|| CustomError::CustomError("Board not found".to_string()))?;

    let column_titles = board.columns.iter().map(|c| c.title.clone()).collect();

    Ok(column_titles)
}

pub async fn delete_card(payload: DeleteCardPayload, db: &Client) -> Result<(), CustomError> {
    let board_service = ODM::<Board>::build(db).await;

    // TODO: replace LJY Members
    let mut boards = board_service
        .fetch_many_by_team("LJY Members")
        .await?;

    let board = boards
        .get_mut(0)
        .ok_or_else(|| CustomError::CustomError("Board not found".to_string()))?;

    let card_oid = ObjectId::parse_str(&payload.card_id)
        .map_err(|_| CustomError::CustomError("Invalid card ID".into()))?;

    let col = board
        .columns
        .iter_mut()
        .find(|c| c.title == payload.column_name)
        .ok_or_else(|| CustomError::CustomError("Column not found".to_string()))?;

    col.cards.retain(|card| card.id != Some(card_oid));

    let board_id = board
        .id
        .clone()
        .ok_or_else(|| CustomError::CustomError("Missing board ID".into()))?;

    board_service.replace_one(board, &board_id).await?;

    Ok(())
}