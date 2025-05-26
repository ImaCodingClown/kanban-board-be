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
        .find_one(doc! { "$or": [{"username": &username}, {"email": &email}] })
        .await
        .map_err(|_| "Error fetching username/email.".to_string())?
        .is_some()
    {
        return Err("Username/email already in use.".into());
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
    user_or_email: String,
    password: String,
    db: &Client,
    secret: &str,
) -> Result<String, String> {
    let users: Collection<User> = db.database("general").collection("users");

    let user_opt = users
        .find_one(doc! { "$or": [{ "username": &user_or_email }, { "email": &user_or_email }]})
        .await
        .map_err(|_| "Error fetching user info.".to_string())?;

    if let Some(user) = user_opt {
        if verify(password, &user.password_hash).unwrap() {
            return Ok(create_jwt(&user.email, secret));
        }
    }

    Err("Invalid credentials".into())
}
