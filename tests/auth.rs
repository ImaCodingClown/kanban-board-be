use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use dotenvy::dotenv;
use http_body_util::BodyExt;
use kanban_backend::config::{AppState, Environment};
use kanban_backend::models::users::User;
use kanban_backend::routes::auth;
use kanban_backend::services::auth::signup;
use kanban_backend::utils::jwt::create_jwt;
use mongodb::{bson::doc, options::ClientOptions, Client};
use std::{env, sync::Arc};
use tower::ServiceExt;

#[tokio::test]
async fn test_username() {
    dotenv().ok();
    let mongo_uri =
        env::var("MONGO_URI").unwrap_or_else(|_| "mongodb://localhost:27017".to_string());

    let client_options = ClientOptions::parse(&mongo_uri).await.unwrap();
    let client = Client::with_options(client_options).unwrap();
    let db = client.database("general");
    let users = db.collection::<User>("users");
    users.drop().await.unwrap();

    let test_username = "test";
    let test_email = "test@gmail.com";

    let result = signup(
        test_username.to_string(),
        test_email.to_string(),
        "testpw123".to_string(),
        &client,
        "rhdfyd",
    )
    .await;

    assert!(result.is_ok());
    let inserted_user = users
        .find_one(doc! { "username": test_username })
        .await
        .unwrap();
    assert!(inserted_user.is_some());
    assert_eq!(inserted_user.unwrap().email, test_email);
}

#[tokio::test]
async fn test_me_endpoint() {
    dotenv().ok();

    let mongo_uri = env::var("MONGO_URI").expect("MONGO_URI must be set");
    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");

    let client_options = ClientOptions::parse(&mongo_uri).await.unwrap();
    let client = Client::with_options(client_options).unwrap();
    let db = client.database("general");
    let users = db.collection::<User>("users");
    users.drop().await.unwrap();

    let test_username = "testuser";
    let test_email = "testuser@gmail.com";

    // First insert user
    let _ = signup(
        test_username.to_string(),
        test_email.to_string(),
        "testpw123".to_string(),
        &client,
        &jwt_secret,
    )
    .await;

    // Manually get the user's ObjectId to generate token
    let user = users
        .find_one(doc! { "username": test_username })
        .await
        .unwrap()
        .expect("User not found in DB");

    let user_id_str = user.id.unwrap().to_hex();
    let token = create_jwt(&user_id_str, &jwt_secret);

    let state = AppState {
        environment: Environment::Dev,
        db: Arc::new(client),
        jwt_secret,
    };

    let app = Router::new().merge(auth::routes()).with_state(state);

    let request = Request::builder()
        .uri("/me")
        .header("Authorization", format!("Bearer {}", token))
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    println!("Response body: {:?}", json);

    let email = json
        .get("email")
        .expect("Missing email field")
        .as_str()
        .unwrap();
    assert_eq!(email, test_email);
}
