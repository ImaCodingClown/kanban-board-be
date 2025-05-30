use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use dotenvy::dotenv;
use http_body_util::BodyExt;
use kanban_backend::{
    config::{AppState, Environment},
    models::users::User,
    services::auth::signup,
    routes::auth,
};
use mongodb::{bson::doc, options::ClientOptions, Client};
use std::env;
use tower::ServiceExt;

#[tokio::test]
async fn test_username() {
    dotenv().ok();
    let mongo_uri = env::var("MONGO_URI").unwrap_or("mongodb://localhost:27017".to_string());

    let client_options = ClientOptions::parse(&mongo_uri).await.unwrap();
    let client = Client::with_options(client_options).unwrap();
    let db = client.database("general");
    let users = db.collection::<User>("users");
    users.drop().await.unwrap(); // Clear before test

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

    let inserted_user = users
        .find_one(doc! { "username": test_username })
        .await
        .unwrap();

    assert!(inserted_user.is_some(), "Failed: {:?}", inserted_user);
    assert_eq!(inserted_user.unwrap().email, test_email);
}

#[derive(Debug, serde::Deserialize)]
struct UserResponse {
    username: String,
    email: String,
}

#[tokio::test]
async fn test_me_endpoint() {
    dotenv().ok();
    let mongo_uri = env::var("MONGO_URI").unwrap_or("mongodb://localhost:27017".to_string());
    let jwt_secret = "rhdfyd";

    let client_options = ClientOptions::parse(&mongo_uri).await.unwrap();
    let client = Client::with_options(client_options).unwrap();
    let db = client.database("general");
    let users = db.collection::<User>("users");
    users.drop().await.unwrap(); // Clear before test

    let test_username = "testuser";
    let test_email = "testuser@gmail.com";

    let _created_user = signup(
        test_username.to_string(),
        test_email.to_string(),
        "testpw123".to_string(),
        &client,
        jwt_secret,
    )
    .await
    .expect("Failed to create test user");

    // Use JWT from separate function, as signup returns String not struct with token
    use kanban_backend::utils::jwt::create_jwt;
    let user_in_db = users
        .find_one(doc! { "username": test_username })
        .await
        .unwrap()
        .expect("User not found");

    let token = create_jwt(
        &user_in_db.id.expect("User missing ID").to_hex(),
        jwt_secret,
    );

    let state = AppState {
        environment: Environment::Test,
        db: client.into(),
        jwt_secret: jwt_secret.to_string(),
    };

    let app = Router::new()
    .nest("/api/auth", auth::routes())
    .with_state(state);


    let request = Request::builder()
        .uri("/api/auth/me")
        .header("Authorization", format!("Bearer {}", token))
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let user: UserResponse =
        serde_json::from_slice(&body).expect("Response deserialization failed");

    assert_eq!(user.username, test_username);
    assert_eq!(user.email, test_email);
}
