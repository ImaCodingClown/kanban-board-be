use crate::create_index;
use crate::models::auth::RefreshToken;
use crate::models::users::User;
use crate::utils::errors::CustomError;
use mongodb::{bson::doc, Client, Collection};

pub async fn create_performance_indexes(db: &Client) -> Result<(), CustomError> {
    let users: Collection<User> = db.database("general").collection("users");
    let refresh_tokens: Collection<RefreshToken> =
        db.database("general").collection("refresh_tokens");

    // Users collection indexes
    create_index!(
        users,
        doc! { "username": 1 },
        "Failed to create username index: {}"
    );
    create_index!(
        users,
        doc! { "email": 1 },
        "Failed to create email index: {}"
    );
    create_index!(
        users,
        doc! { "username": 1, "email": 1 },
        "Failed to create compound index: {}"
    );

    // Refresh tokens collection indexes
    create_index!(
        refresh_tokens,
        doc! { "user_email": 1, "expires_at": 1 },
        "Failed to create user_email_expires index: {}"
    );
    create_index!(
        refresh_tokens,
        doc! { "token_hash": 1 },
        "Failed to create token_hash index: {}"
    );

    Ok(())
}
