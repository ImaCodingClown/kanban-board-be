use crate::{
    db::mongo::{MongoService, ODM},
    models::cards::Board,
    utils::errors::CustomError,
};
use mongodb::{bson::{doc, oid::ObjectId}, Client};

pub async fn get_board_by_id(board_id: String, db: &Client) -> Result<Board, CustomError> {
    let board_service = ODM::<Board>::build(db).await;
    let object_id = ObjectId::parse_str(&board_id)
        .map_err(|_| CustomError::NotFound("Invalid board ID".to_string()))?;
    
    match board_service.fetch_by_id(object_id).await? {
        Some(board) => Ok(board),
        None => Err(CustomError::NotFound("Board not found".to_string())),
    }
}

pub async fn get_boards_by_team(team_name: String, db: &Client) -> Result<Vec<Board>, CustomError> {
    let board_service = ODM::<Board>::build(db).await;
    board_service.fetch_many_by_team(&team_name).await
}

pub async fn get_board_by_team(team_name: String, db: &Client) -> Result<Board, CustomError> {
    let boards = get_boards_by_team(team_name.clone(), db).await?;
    
    if let Some(board) = boards.first() {
        Ok(board.clone())
    } else {
        // Create a default board if none exists
        let new_board = Board::create_default(team_name.clone(), team_name.clone() + "'s Board");
        let board_service = ODM::<Board>::build(db).await;
        board_service.save_one(&new_board).await?;
        Ok(new_board)
    }
}

pub async fn create_board(
    team_name: String,
    board_name: String,
    db: &Client,
) -> Result<Board, CustomError> {
    let board_service = ODM::<Board>::build(db).await;

    // Check if board with same name already exists in the team
    let existing_boards = board_service.fetch_many_by_team(&team_name).await?;
    for board in &existing_boards {
        if board.board_name == board_name {
            return Err(CustomError::Conflict(format!(
                "Board '{}' already exists in team '{}'",
                board_name, team_name
            )));
        }
    }

    let board = Board::create_default(team_name.clone(), board_name.clone());
    board_service.save_one(&board).await?;
    Ok(board)
}

pub async fn update_board(board: Board, db: &Client) -> Result<Board, CustomError> {
    let board_service = ODM::<Board>::build(db).await;
    let board_id = board.id.as_ref()
        .ok_or_else(|| CustomError::NotFound("Missing board ID".to_string()))?;
    
    board_service.replace_one(&board, board_id).await?;
    Ok(board)
}

pub async fn delete_board(board_id: String, db: &Client) -> Result<(), CustomError> {
    let board_service = ODM::<Board>::build(db).await;
    let object_id = ObjectId::parse_str(&board_id)
        .map_err(|_| CustomError::NotFound("Invalid board ID".to_string()))?;
    
    board_service.collection
        .delete_one(doc! { "_id": object_id })
        .await
        .map_err(CustomError::MongoError)?;
    Ok(())
}
