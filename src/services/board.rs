// File: src/services/board.rs
use crate::{
    db::mongo::{MongoService, ODM},
    models::cards::Board,
    utils::errors::CustomError,
};
use mongodb::Client;

pub async fn get_board_by_team(team_name: String, db: &Client) -> Result<Board, CustomError> {
    let board_service = ODM::<Board>::build(db).await;
    //TODO: Replace LJY Members
    let mut boards = board_service.fetch_many_by_team("LJY Members").await?;

    if let Some(board) = boards.pop() {
        Ok(board)
    } else {
        Ok(Board::create_default("LJY Members".to_string()))
    }
}

pub async fn create_board(team_name: String, db: &Client) -> Result<Board, CustomError> {
    //TODO: Replace LJY Members
    let board = Board::create_default("LJY Members".to_string());
    let board_service = ODM::<Board>::build(db).await;
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
