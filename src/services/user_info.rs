use crate::{
    models::users::{UpdateSlackIdPayload, User},
    utils::errors::CustomError,
};
use futures::TryStreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId},
    Client, Database,
};

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

pub async fn update_user_slack_id(
    user_id: &str,
    payload: UpdateSlackIdPayload,
    db: &Client,
) -> Result<User, CustomError> {
    let user_oid = ObjectId::parse_str(user_id)
        .map_err(|_| CustomError::NotFound(format!("Invalid user ID format: '{}'. Expected MongoDB ObjectId format.", user_id)))?;

    let users = db.database("general").collection::<User>("users");

    let filter = doc! { "_id": user_oid };
    let update = doc! {
        "$set": {
            "slack_user_id": payload.slack_user_id
        }
    };

    let result = users
        .find_one_and_update(filter, update)
        .await
        .map_err(|e| CustomError::Server(format!("Database error: {}", e)))?;

    result.ok_or_else(|| CustomError::NotFound("User not found".to_string()))
}

pub async fn get_user_by_username(username: &str, db: &Client) -> Result<User, CustomError> {
    let users = db.database("general").collection::<User>("users");

    let filter = doc! { "username": username };

    let user = users
        .find_one(filter)
        .await
        .map_err(|e| CustomError::Server(format!("Database error: {}", e)))?
        .ok_or_else(|| CustomError::NotFound("User not found".to_string()))?;

    Ok(user)
}
