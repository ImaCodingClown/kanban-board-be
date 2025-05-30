use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use dotenvy::dotenv;
use kanban_backend::{
    config::{AppState, Environment},
    models::users::User,
    routes::auth::routes as auth_routes,
    services::auth::signup,
    utils::jwt::create_jwt,
};
use mongodb::{bson::doc, options::ClientOptions, Client};
use std::env;
use std::sync::Arc;
use tower::ServiceExt;

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

#[tokio::test]
async fn test_me_endpoint() {
    dotenv().ok();
    let mongo_uri = env::var("MONGO_URI").unwrap_or("mongodb://localhost:27017".to_string());
    let jwt_secret = "rhdfyd".to_string();

    let client_options = ClientOptions::parse(&mongo_uri).await.unwrap();
    let client = Arc::new(Client::with_options(client_options).unwrap());
    let _db = client.database("general");

    let user_id = "68339cdbb0ea56d83356e547"; // Example ObjectId, replace with a valid one from your test database
    let token = create_jwt(user_id, &jwt_secret);

    let state = AppState {
        environment: Environment::Dev,
        db: client.clone(),
        jwt_secret,
    };

    let app = auth_routes().with_state(state);

    let request = Request::builder()
        .uri("/me")
        .method("GET")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}
