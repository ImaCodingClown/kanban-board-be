use crate::models::users::User;
use mongodb::{bson::doc, Database, Client};

pub async fn get_user_by_email(db: &Database, email: &str) -> Result<User, String> {
    let users = db.collection::<User>("users");
    users
        .find_one(doc! { "email": email })
        .await
        .map_err(|e| format!("DB error: {e}"))?
        .ok_or_else(|| "User not found".to_string())
}

pub async fn update_user_teams(
    db: &Client,
    email: &str,
    teams: Vec<String>,
) -> Result<(), String> {
    let users = db.database("general").collection::<User>("users");
    
    users
        .update_one(
            doc! { "email": email },
            doc! { "$set": { "teams": teams } },
        )
        .await
        .map_err(|e| format!("Failed to update user teams: {e}"))?;
    
    Ok(())
}