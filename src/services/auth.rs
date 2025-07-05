use crate::db::mongo::{MongoService, ODM};
use crate::models::users::User;
use crate::utils::errors::CustomError;
use crate::utils::jwt::{JWTMethods, JWTValidator};
use bcrypt::{hash, verify};
use mongodb::{bson::doc, Client, Collection};
use crate::services::board::create_board;

pub async fn signup(
    username: String,
    email: String,
    password: String,
    db: &Client,
    secret: &str,
) -> Result<String, CustomError> {
    let user_service = ODM::<User>::build(db).await;
    let hashed = hash(&password, 4).unwrap();
    let user = User::create(username.clone(), email.clone(), hashed);

    if user_service.fetch_one(&user).await?.is_some() {
        return Err("Username/email already in use.".into());
    }

    user_service.save_one(&user).await?;
    create_board(username, db).await?;
    Ok(JWTValidator::create_jwt(&email, secret))
}

pub async fn login(
    user_or_email: String,
    password: String,
    db: &Client,
    secret: &str,
) -> Result<String, String> {
    let users: Collection<User> = db.database("general").collection("users");

    let user_opt = users
        .find_one(doc! { "$or": [{ "username": &user_or_email }, { "email": &user_or_email }]})
        .await
        .map_err(|_| "Error fetching user info.".to_string())?;

    if let Some(user) = user_opt {
        if verify(password, &user.password_hash).unwrap() {
            return Ok(JWTValidator::create_jwt(&user.email, secret));
        }
    }

    Err("Invalid credentials".into())
}
