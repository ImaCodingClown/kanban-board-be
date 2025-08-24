use crate::models::users::User;
use mongodb::{bson::doc, Database};

pub async fn get_user_by_email(db: &Database, email: &str) -> Result<User, String> {
    let users = db.collection::<User>("users");
    users
        .find_one(doc! { "email": email })
        .await
        .map_err(|e| format!("DB error: {e}"))?
        .ok_or_else(|| "User not found".to_string())
}
