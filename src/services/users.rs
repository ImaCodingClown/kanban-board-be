use crate::models::users::User;
use mongodb::{
    bson::{doc, oid::ObjectId},
    Database,
};

pub async fn get_user_by_id(db: &Database, user_id: &str) -> Result<User, String> {
    let users = db.collection::<User>("users");
    let user_id = ObjectId::parse_str(user_id).map_err(|_| "Invalid ID format".to_string())?;
    users
        .find_one(doc! { "_id": user_id })
        .await
        .map_err(|e| format!("DB error: {}", e))?
        .ok_or("User not found".to_string())
}
