use crate::{
    db::mongo::{MongoService, ODM},
    models::cards::{AddCardPayload, Board, Card, DeleteCardPayload, EditCardPayload},
    utils::errors::CustomError,
};
use mongodb::{bson::oid::ObjectId, Client};

pub async fn add_card(payload: AddCardPayload, db: &Client) -> Result<Card, CustomError> {
    let board_service = ODM::<Board>::build(db).await;
    let mut boards = board_service.fetch_many_by_team(&payload.team).await?;

    let board = boards
        .get_mut(0)
        .ok_or_else(|| CustomError::NotFound("Board not found".to_string()))?;

    let col = board
        .columns
        .iter_mut()
        .find(|c| c.title == payload.column_name)
        .ok_or_else(|| CustomError::NotFound("Column not found".to_string()))?;

    let card = Card {
        id: Some(ObjectId::new()),
        title: payload.title,
        description: payload.description,
        assignee: payload.assignee,
        story_point: payload.story_point,
        priority: payload.priority,
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

    let board = boards
        .first()
        .ok_or_else(|| CustomError::NotFound("Board not found".to_string()))?;

    let column_titles = board.columns.iter().map(|c| c.title.clone()).collect();

    Ok(column_titles)
}

pub async fn delete_card(payload: DeleteCardPayload, db: &Client) -> Result<(), CustomError> {
    let board_service = ODM::<Board>::build(db).await;
    let mut boards = board_service.fetch_many_by_team(&payload.team).await?;

    let board = boards
        .get_mut(0)
        .ok_or_else(|| CustomError::NotFound("Board not found".to_string()))?;

    let card_oid = ObjectId::parse_str(&payload.card_id)
        .map_err(|_| CustomError::NotFound("Invalid card ID".to_string()))?;

    let col = board
        .columns
        .iter_mut()
        .find(|c| c.title == payload.column_name)
        .ok_or_else(|| CustomError::NotFound("Column not found".to_string()))?;

    col.cards.retain(|card| card.id != Some(card_oid));

    let board_id = board
        .id
        .clone()
        .ok_or_else(|| CustomError::NotFound("Missing board ID".to_string()))?;

    board_service.replace_one(board, &board_id).await?;

    Ok(())
}

pub async fn edit_card(payload: EditCardPayload, db: &Client) -> Result<Card, CustomError> {
    let board_service = ODM::<Board>::build(db).await;
    let mut boards = board_service.fetch_many_by_team(&payload.team).await?;

    let board = boards
        .get_mut(0)
        .ok_or_else(|| CustomError::NotFound("Board not found".to_string()))?;

    let card_oid = ObjectId::parse_str(&payload.card_id)
        .map_err(|_| CustomError::NotFound("Invalid card ID".to_string()))?;

    {
        let column = board
            .columns
            .iter_mut()
            .find(|col| col.title == payload.column_name)
            .ok_or_else(|| CustomError::NotFound("Column not found".to_string()))?;

        let card = column
            .cards
            .iter_mut()
            .find(|card| card.id == Some(card_oid))
            .ok_or_else(|| CustomError::NotFound("Card not found".to_string()))?;

        card.title = payload.title.clone();
        card.description = Some(payload.description.clone());
        card.assignee = Some(payload.assignee.clone());
        card.story_point = payload.story_point.clone();
        card.priority = payload.priority.clone();
    }

    let board_id = board.id.clone().unwrap();
    board_service.replace_one(board, &board_id).await?;

    let edited_card = board
        .columns
        .iter()
        .find(|col| col.title == payload.column_name)
        .and_then(|col| col.cards.iter().find(|c| c.id == Some(card_oid)))
        .ok_or_else(|| CustomError::NotFound("Updated card not found".to_string()))?;

    Ok(edited_card.clone())
}
