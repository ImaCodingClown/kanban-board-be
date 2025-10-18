use crate::{
    db::mongo::{MongoService, ODM},
    models::cards::Board,
    utils::errors::CustomError,
};
use mongodb::Client;

pub async fn get_board_by_team(team_name: String, db: &Client) -> Result<Board, CustomError> {
    let board_service = ODM::<Board>::build(db).await;
    let mut boards = board_service.fetch_many_by_team(&team_name).await?;

    if let Some(board) = boards.pop() {
        Ok(board)
    } else {
        // Create a default board if none exists
        let new_board = Board::create_default(team_name.clone(), team_name.clone() + "'s Board".to_string());
        board_service.save_one(&new_board).await?;
        Ok(new_board)   
    }
}

pub async fn create_board(team_name: String, board_name: String, db: &Client) -> Result<Board, CustomError> {
    let board_service = ODM::<Board>::build(db).await;

    let existing_boards = board_service.fetch_many_by_team(&team_name).await?;
    if !existing_boards.is_empty() {
        return Ok(existing_boards.into_iter().next().unwrap());
    }

    let board = Board::create_default(team_name.clone(), board_name.clone());
    board_service.save_one(&board).await?;
    Ok(board)
}

pub async fn update_board(board: Board, db: &Client) -> Result<Board, CustomError> {
    let board_service = ODM::<Board>::build(db).await;
    board_service
        .replace_one(&board, &board.id.unwrap())
        .await?;
    Ok(board)
}
