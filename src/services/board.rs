use crate::{
    db::mongo::{MongoService, ODM},
    models::cards::Board,
    utils::errors::CustomError,
};
use mongodb::Client;

pub async fn get_board(team_name: String, db: &Client) -> Result<Vec<Board>, CustomError> {
    let board_service = ODM::<Board>::build(db).await;
    let board = Board::new(team_name);
    board_service.fetch_many(&board).await
}

pub async fn create_board(team_name: String, db: &Client) -> Result<Board, CustomError> {
    let board = Board::create_default(team_name);
    let board_service = ODM::<Board>::build(db).await;
    board_service.save_one(&board).await?;
    Ok(board)
}