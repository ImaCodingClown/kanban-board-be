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
