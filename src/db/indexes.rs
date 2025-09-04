use crate::utils::errors::CustomError;
use mongodb::{bson::doc, Client, Collection, IndexModel};
use crate::models::users::User;
use crate::models::auth::RefreshToken;

pub async fn create_performance_indexes(db: &Client) -> Result<(), CustomError> {
    let users: Collection<User> = db.database("general").collection("users");
    let refresh_tokens: Collection<RefreshToken> = db.database("general").collection("refresh_tokens");
    
    users.create_index(
        IndexModel::builder().keys(doc! { "username": 1 }).build()
    ).await.map_err(|e| CustomError::Database(format!("Failed to create username index: {}", e)))?;
    
    users.create_index(
        IndexModel::builder().keys(doc! { "email": 1 }).build()
    ).await.map_err(|e| CustomError::Database(format!("Failed to create email index: {}", e)))?;
    
    users.create_index(
        IndexModel::builder().keys(doc! { "username": 1, "email": 1 }).build()
    ).await.map_err(|e| CustomError::Database(format!("Failed to create compound index: {}", e)))?;
    
    refresh_tokens.create_index(
        IndexModel::builder().keys(doc! { "user_email": 1, "expires_at": 1 }).build()
    ).await.map_err(|e| CustomError::Database(format!("Failed to create user_email_expires index: {}", e)))?;
    
    refresh_tokens.create_index(
        IndexModel::builder().keys(doc! { "token_hash": 1 }).build()
    ).await.map_err(|e| CustomError::Database(format!("Failed to create token_hash index: {}", e)))?;
    
    Ok(())
}