use crate::models::users::User;
use futures::TryStreamExt;
use mongodb::{bson::doc, Client, Database};

pub async fn get_user_by_email(db: &Database, email: &str) -> Result<User, String> {
    let users = db.collection::<User>("users");
    users
        .find_one(doc! { "email": email })
        .await
        .map_err(|e| format!("DB error: {e}"))?
        .ok_or_else(|| "User not found".to_string())
}

pub async fn get_all_users(db: &Client) -> Result<Vec<User>, String> {
    let users = db.database("general").collection::<User>("users");

    let cursor = users
        .find(doc! {})
        .await
        .map_err(|e| format!("Failed to query users: {e}"))?;

    let users: Vec<User> = cursor
        .try_collect()
        .await
        .map_err(|e| format!("Failed to collect users: {e}"))?;

    Ok(users)
}
