use crate::{
    db::mongo::{MongoService, ODM},
    models::cards::{Board, Card, Column},
    utils::errors::CustomError,
};
use mongodb::Client;

pub async fn get_board(team_name: String, db: &Client) -> Result<Option<Board>, CustomError> {
    let board_service = ODM::<Board>::build(db).await;
    let board = Board::new(team_name);
    board_service.fetch_one(&board).await
}
