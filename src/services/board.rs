use crate::{
    db::mongo::{MongoService, ODM},
    models::cards::{Board, Column},
    utils::errors::CustomError,
};
use mongodb::Client;

pub async fn get_board(team_name: String, db: &Client) -> Result<Vec<Board>, CustomError> {
    let board_service = ODM::<Board>::build(db).await;
    let board = Board::new(team_name);
    board_service.fetch_many(&board).await
}

pub async fn create_board(team_name: String, db: &Client) -> Result<Board, CustomError> {
    let mut board = Board::new(team_name);
    let board_service = ODM::<Board>::build(db).await;

    board.columns = vec![
        Column {
            title: "To Do".into(),
            cards: vec![],
        },
        Column {
            title: "In Progress".into(),
            cards: vec![],
        },
        Column {
            title: "Done".into(),
            cards: vec![],
        },
    ];

    board_service.save_one(&board).await?;
    Ok(board)
}