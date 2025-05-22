use crate::models::users::User;
use crate::utils::jwt::create_jwt;
use bcrypt::{hash, verify};
use mongodb::{bson::doc, Client, Collection};

pub async fn signup(
    username: String,
    email: String,
    password: String,
    db: &Client,
    secret: &str,
) -> Result<String, String> {
    let users: Collection<User> = db.database("general").collection("users");

    println!("{}", email.as_str());

    if users
        .find_one(doc! { "email": &email })
        .await
        .unwrap()
        .is_some()
    {
        return Err("Email already in use".into());
    }

    if users
        .find_one(doc! { "username": &username })
        .await
        .unwrap()
        .is_some()
    {
        return Err("Username already in use".into());
    }

    let hashed = hash(&password, 4).unwrap();
    let user = User {
        username: username.clone(),
        id: None,
        email: email.clone(),
        password_hash: hashed,
    };

    users.insert_one(user).await.unwrap();
    Ok(create_jwt(&email, secret))
}

pub async fn login(
    username: String,
    email: String,
    password: String,
    db: &Client,
    secret: &str,
) -> Result<String, String> {
    let users: Collection<User> = db.database("general").collection("users");

    let username_opt = users
        .find_one(doc! { "username": &username })
        .await
        .unwrap();

    if let Some(user) = username_opt {
        if user.email == email && verify(password, &user.password_hash).unwrap() {
            return Ok(create_jwt(&user.email, secret));
        }
    }

    Err("Invalid credentials".into())
}

// Test username
#[cfg(test)]
mod tests {
    use super::*;
    use dotenvy::dotenv;
    use mongodb::{options::ClientOptions, Client};
    use std::env;

    #[tokio::test]
    async fn test_username() {
        dotenv().ok();
        let mongo_uri = env::var("MONGO_URI").unwrap_or("mongodb://localhost:27017".to_string());

        let client_options = ClientOptions::parse(&mongo_uri).await.unwrap();
        let client = Client::with_options(client_options).unwrap();
        let db = client.database("general");
        let users = db.collection::<User>("users");
        users.drop().await.unwrap(); // clear collection before test

        let test_username = "test";
        let test_email = "test@gmail.com";

        let result = signup(
            test_username.to_string(),
            test_email.to_string(),
            "testpw123".to_string(),
            &client,
            "test_secret",
        )
        .await;

        assert!(result.is_ok(), "signup failed: {:?}", result);

        // check username
        let inserted_user = users
            .find_one(doc! { "username": test_username })
            .await
            .unwrap();

        assert!(inserted_user.is_some(), "Failed: {:?}", inserted_user);
        assert_eq!(inserted_user.unwrap().email, test_email);
    }
}
