use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use dotenvy::dotenv;
use http_body_util::BodyExt;
use kanban_backend::config::{AppState, Environment};
use kanban_backend::models::users::User;
use kanban_backend::routes::auth::routes as auth_routes; // <-- merge all auth routes, including GET /me
use kanban_backend::services::auth::signup;
use kanban_backend::utils::jwt::{JWTMethods, JWTValidator};
use mongodb::{bson::doc, options::ClientOptions, Client};
use std::{env, sync::Arc};
use tower::ServiceExt;

#[tokio::test]
async fn test_username() {
    dotenv().ok();
    let mongo_uri = env::var("MONGO_URI").expect("MONGO_URI must be set");

    let client_options = ClientOptions::parse(&mongo_uri).await.unwrap();
    let client = Client::with_options(client_options).unwrap();
    let users = client.database("general").collection::<User>("users");
    users.drop().await.unwrap();

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

    // Setup Mongo client and clear users
    let client_options = ClientOptions::parse(&mongo_uri).await.unwrap();
    let client = Client::with_options(client_options).unwrap();
    let users = client.database("general").collection::<User>("users");
    users.drop().await.unwrap();

    // Create a new user via the service
    let test_username = "testuser";
    let test_email = "testuser@gmail.com";
    let _ = signup(
        test_username.to_string(),
        test_email.to_string(),
        "testpw123".to_string(),
        &client,
        &jwt_secret,
    )
    .await;

    // Load the inserted user to get their email
    let user = users
        .find_one(doc! { "username": test_username })
        .await
        .unwrap()
        .expect("User not found in DB");

    // Generate a JWT with the user's email in `sub`
    let token = JWTValidator::create_jwt(&user.email, &jwt_secret);

    // Build shared AppState (with database and secret)
    let state = AppState {
        environment: Environment::Dev,
        db: Arc::new(client),
        jwt_secret,
    };

    // Merge your auth routes (including GET /me) with that state
    let app = Router::new().merge(auth_routes()).with_state(state);

    // Perform GET /me with the Bearer token
    let request = Request::builder()
        .uri("/me")
        .header("Authorization", format!("Bearer {token}"))
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Assert we get 200 OK and the correct email back
    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(json.get("email").unwrap().as_str().unwrap(), test_email);
}
