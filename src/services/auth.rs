use crate::db::mongo::{MongoService, ODM};
use crate::models::auth::{AuthResponse, RefreshToken};
use crate::models::users::User;
use crate::services::board::get_board_by_team;
use crate::services::teams::add_user_to_ljy_team;
use crate::utils::errors::CustomError;
use crate::utils::jwt::{JWTMethods, JWTValidator};
use bcrypt::{hash, verify};
use chrono::Utc;
use mongodb::{bson::doc, Client, Collection, IndexModel};
use tokio;

use futures::StreamExt;

const MAX_SESSIONS_PER_USER: usize = 3;

pub async fn signup(
    username: String,
    email: String,
    password: String,
    db: &Client,
    secret: &str,
) -> Result<AuthResponse, CustomError> {
    let user_service = ODM::<User>::build(db).await;
    let hashed = hash(&password, 4)
        .map_err(|e| CustomError::Server(format!("Password hashing failed: {}", e)))?;
    let teams = vec!["LJY Members".to_string()];
    let user = User::create(username.clone(), email.clone(), hashed, teams.clone());

    if user_service.fetch_one(&user).await?.is_some() {
        return Err(CustomError::Conflict(
            "Username/email already in use".to_string(),
        ));
    }

    let save_result = user_service
        .save_one(&user)
        .await
        .map_err(|e| CustomError::Database(format!("Failed to save user: {}", e)))?;

    let user_id = save_result
        .inserted_id
        .as_object_id()
        .ok_or_else(|| CustomError::Server("Failed to get user ID after save".to_string()))?;

    let _ = get_board_by_team("LJY Members".to_string(), db).await;

    if let Err(_) = add_user_to_ljy_team(db, user_id, &email).await {
        // Silently continue if team addition fails
    }

    let (access_token, refresh_token) = JWTValidator::create_jwt(&email, secret);
    let _ = save_refresh_token(&email, &refresh_token, db)
        .await
        .map_err(|e| CustomError::Database(format!("Failed to save refresh token: {}", e)))?;

    Ok(AuthResponse {
        access_token,
        refresh_token,
        expires_in: 12 * 60 * 60, // 12 hours in seconds
    })
}

pub async fn login(
    user_or_email: String,
    password: String,
    db: &Client,
    secret: &str,
) -> Result<AuthResponse, CustomError> {
    let users: Collection<User> = db.database("general").collection("users");

    let user_opt = users
        .find_one(doc! { "$or": [{ "username": &user_or_email }, { "email": &user_or_email }]})
        .await
        .map_err(|e| CustomError::Database(format!("Error fetching user info: {}", e)))?;

    if let Some(user) = user_opt {
        if verify(password, &user.password_hash)
            .map_err(|e| CustomError::Server(format!("Password verification failed: {}", e)))?
        {
            let (access_token, refresh_token) = JWTValidator::create_jwt(&user.email, secret);

            let response = AuthResponse {
                access_token,
                refresh_token: refresh_token.clone(),
                expires_in: 12 * 60 * 60,
            };

            let user_email = user.email.clone();
            let db_clone = db.clone();
            tokio::spawn(async move {
                let _ =
                    manage_sessions_and_save_token(&user_email, &refresh_token, &db_clone).await;
            });

            return Ok(response);
        }
    }

    Err(CustomError::Authentication(
        "Invalid credentials".to_string(),
    ))
}

pub async fn refresh_access_token(
    refresh_token: String,
    db: &Client,
    secret: &str,
) -> Result<AuthResponse, CustomError> {
    let refresh_tokens: Collection<RefreshToken> =
        db.database("general").collection("refresh_tokens");

    let token_hash = JWTValidator::hash_refresh_token(&refresh_token);
    let now = Utc::now().timestamp();

    let stored_token = refresh_tokens
        .find_one(doc! {
            "token_hash": &token_hash,
            "expires_at": { "$gt": now }
        })
        .await
        .map_err(|e| CustomError::Database(format!("Error fetching refresh token: {}", e)))?;

    let stored_token = stored_token.ok_or_else(|| {
        CustomError::Authentication("Invalid or expired refresh token".to_string())
    })?;

    let new_access_token = JWTValidator::create_access_token(
        &stored_token.user_email,
        &stored_token.id.to_hex(),
        secret,
    );

    Ok(AuthResponse {
        access_token: new_access_token,
        refresh_token: refresh_token,
        expires_in: 12 * 60 * 60,
    })
}

pub async fn logout(
    _user_email: &str,
    refresh_token: &str,
    db: &Client,
) -> Result<(), CustomError> {
    let refresh_tokens: Collection<RefreshToken> =
        db.database("general").collection("refresh_tokens");

    let token_hash = JWTValidator::hash_refresh_token(refresh_token);
    refresh_tokens
        .delete_one(doc! { "token_hash": &token_hash })
        .await
        .map_err(|e| CustomError::Database(format!("Failed to delete refresh token: {}", e)))?;

    Ok(())
}

async fn save_refresh_token(
    user_email: &str,
    refresh_token: &str,
    db: &Client,
) -> Result<(), String> {
    let refresh_tokens: Collection<RefreshToken> =
        db.database("general").collection("refresh_tokens");

    let token_hash = JWTValidator::hash_refresh_token(refresh_token);
    let now = Utc::now().timestamp();
    let expires_at = now + (30 * 24 * 60 * 60); // 30 days from now

    let refresh_token_doc = RefreshToken {
        id: mongodb::bson::oid::ObjectId::new(),
        user_email: user_email.to_string(),
        token_hash,
        expires_at,
    };

    refresh_tokens
        .insert_one(&refresh_token_doc)
        .await
        .map_err(|_| "Failed to save refresh token".to_string())?;

    Ok(())
}

async fn count_active_sessions(user_email: &str, db: &Client) -> Result<usize, CustomError> {
    let refresh_tokens: Collection<RefreshToken> =
        db.database("general").collection("refresh_tokens");
    let now = Utc::now().timestamp();

    let pipeline = vec![
        doc! { "$match": {
            "user_email": user_email,
            "expires_at": { "$gt": now }
        }},
        doc! { "$count": "active_sessions" },
    ];

    let mut cursor = refresh_tokens
        .aggregate(pipeline)
        .await
        .map_err(|e| CustomError::Database(format!("Error counting sessions: {}", e)))?;

    if let Some(result) = cursor.next().await {
        let count_doc: mongodb::bson::Document =
            result.map_err(|e| CustomError::Database(format!("Error parsing count: {}", e)))?;
        Ok(count_doc.get_i32("active_sessions").unwrap_or(0) as usize)
    } else {
        Ok(0)
    }
}

async fn remove_oldest_session(user_email: &str, db: &Client) -> Result<(), CustomError> {
    let refresh_tokens: Collection<RefreshToken> =
        db.database("general").collection("refresh_tokens");
    let now = Utc::now().timestamp();

    let pipeline = vec![
        doc! { "$match": {
            "user_email": user_email,
            "expires_at": { "$gt": now }
        }},
        doc! { "$sort": { "_id": 1 } },
        doc! { "$limit": 1 },
        doc! { "$project": { "_id": 1 } },
    ];

    let mut cursor = refresh_tokens
        .aggregate(pipeline)
        .await
        .map_err(|e| CustomError::Database(format!("Error finding oldest session: {}", e)))?;

    if let Some(result) = cursor.next().await {
        let doc: mongodb::bson::Document =
            result.map_err(|e| CustomError::Database(format!("Error parsing document: {}", e)))?;
        if let Ok(id) = doc.get_object_id("_id") {
            refresh_tokens
                .delete_one(doc! { "_id": id })
                .await
                .map_err(|e| {
                    CustomError::Database(format!("Error deleting oldest session: {}", e))
                })?;
        }
    }

    Ok(())
}
async fn manage_sessions_and_save_token(
    user_email: &str,
    refresh_token: &str,
    db: &Client,
) -> Result<(), CustomError> {
    let active_sessions = count_active_sessions(user_email, db).await?;

    if active_sessions >= MAX_SESSIONS_PER_USER {
        remove_oldest_session(user_email, db).await?;
    }

    save_refresh_token(user_email, refresh_token, db)
        .await
        .map_err(|e| CustomError::Database(format!("Failed to save refresh token: {}", e)))?;

    Ok(())
}
