use std::env;
use dotenvy::dotenv;
use mongodb::{bson::doc, options::ClientOptions, Client};
use kanban_backend::models::users::User;
use kanban_backend::services::auth::signup;

#[tokio::test]
async fn test_username() {
    dotenv().ok();
    let mongo_uri = env::var("MONGO_URI").unwrap_or("mongodb://localhost:27017".to_string());

    let client_options = ClientOptions::parse(&mongo_uri).await.unwrap();
    let client = Client::with_options(client_options).unwrap();
    let db = client.database("general");
    let users = db.collection::<User>("users");
    users.drop().await.unwrap(); // clear collection before test

    let test_username = "test";
    let test_email = "test@gmail.com";

    let result = signup(
        test_username.to_string(),
        test_email.to_string(),
        "testpw123".to_string(),
        &client,
        "test_secret",
    )
    .await;

    assert!(result.is_ok(), "signup failed: {:?}", result);

    // check username
    let inserted_user = users
        .find_one(doc! { "username": test_username })
        .await
        .unwrap();

    assert!(inserted_user.is_some(), "Failed: {:?}", inserted_user);
    assert_eq!(inserted_user.unwrap().email, test_email);
}
