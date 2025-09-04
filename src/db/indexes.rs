use crate::models::auth::RefreshToken;
use crate::models::users::User;
use crate::utils::errors::CustomError;
use mongodb::{bson::doc, Client, Collection, IndexModel};

pub async fn create_performance_indexes(db: &Client) -> Result<(), CustomError> {
    let users: Collection<User> = db.database("general").collection("users");
    let refresh_tokens: Collection<RefreshToken> =
        db.database("general").collection("refresh_tokens");

    let users_indexes = users
        .list_indexes()
        .await
        .map_err(|e| CustomError::Database(format!("Failed to list user indexes: {}", e)))?;
    let users_index_names: Vec<String> = users_indexes
        .map(|idx| idx.name)
        .collect()
        .await
        .map_err(|e| CustomError::Database(format!("Failed to collect user index names: {}", e)))?;

    let refresh_tokens_indexes = refresh_tokens.list_indexes().await.map_err(|e| {
        CustomError::Database(format!("Failed to list refresh token indexes: {}", e))
    })?;
    let refresh_tokens_index_names: Vec<String> = refresh_tokens_indexes
        .map(|idx| idx.name)
        .collect()
        .await
        .map_err(|e| {
            CustomError::Database(format!(
                "Failed to collect refresh token index names: {}",
                e
            ))
        })?;

    if !users_index_names.contains(&"username_1".to_string()) {
        users
            .create_index(IndexModel::builder().keys(doc! { "username": 1 }).build())
            .await
            .map_err(|e| {
                CustomError::Database(format!("Failed to create username index: {}", e))
            })?;
    }

    if !users_index_names.contains(&"email_1".to_string()) {
        users
            .create_index(IndexModel::builder().keys(doc! { "email": 1 }).build())
            .await
            .map_err(|e| CustomError::Database(format!("Failed to create email index: {}", e)))?;
    }

    if !users_index_names.contains(&"username_1_email_1".to_string()) {
        users
            .create_index(
                IndexModel::builder()
                    .keys(doc! { "username": 1, "email": 1 })
                    .build(),
            )
            .await
            .map_err(|e| {
                CustomError::Database(format!("Failed to create compound index: {}", e))
            })?;
    }

    if !refresh_tokens_index_names.contains(&"user_email_1_expires_at_1".to_string()) {
        refresh_tokens
            .create_index(
                IndexModel::builder()
                    .keys(doc! { "user_email": 1, "expires_at": 1 })
                    .build(),
            )
            .await
            .map_err(|e| {
                CustomError::Database(format!("Failed to create user_email_expires index: {}", e))
            })?;
    }

    if !refresh_tokens_index_names.contains(&"token_hash_1".to_string()) {
        refresh_tokens
            .create_index(IndexModel::builder().keys(doc! { "token_hash": 1 }).build())
            .await
            .map_err(|e| {
                CustomError::Database(format!("Failed to create token_hash index: {}", e))
            })?;
    }

    Ok(())
}
