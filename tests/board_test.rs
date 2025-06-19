use dotenvy::dotenv;
use kanban_backend::{models::cards::Board, services::board::get_board};
use mongodb::{options::ClientOptions, Client};
use std::env;
#[tokio::test]
async fn test_get_board() {
    dotenv().ok();
    let mongo_uri = env::var("MONGO_URI").unwrap_or("mongodb://localhost:27017".to_string());

    let client_options = ClientOptions::parse(&mongo_uri).await.unwrap();
    let client = Client::with_options(client_options).unwrap();
    let db = client.database("general");
    let users = db.collection::<Board>("boards");
    users.drop().await.unwrap(); // clear collection before test

    let result = get_board("".to_string(), &client).await;

    assert!(result.is_ok(), "get_board failed: {:?}", result);

    let columns = result.unwrap();
    assert!(columns.is_none(), "Board should be empty");
}
