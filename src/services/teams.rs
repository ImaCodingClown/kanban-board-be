use crate::models::users::User;
use mongodb::{bson::doc, Client};

pub async fn update_user_teams(db: &Client, email: &str, teams: Vec<String>) -> Result<(), String> {
    let users = db.database("general").collection::<User>("users");

    let result = users
        .update_one(doc! { "email": email }, doc! { "$set": { "teams": teams } })
        .await
        .map_err(|e| format!("Failed to update user teams: {e}"))?;

    if result.modified_count == 0 {
        return Err("User not found or no changes made".to_string());
    }

    Ok(())
}

pub async fn create_team(db: &Client, email: &str, team_name: &str) -> Result<Vec<String>, String> {
    println!("create_team called for email: {}, team: {}", email, team_name);
    
    let users = db.database("general").collection::<User>("users");

    let user = users
        .find_one(doc! { "email": email })
        .await
        .map_err(|e| format!("Failed to find user: {e}"))?
        .ok_or_else(|| "User not found".to_string())?;

    println!("Found user: {}, current teams: {:?}", user.username, user.teams);

    let mut teams = user.teams;

    if teams.contains(&team_name.to_string()) {
        println!("User already in team: {}", team_name);
        return Err("Already a member of this team".to_string());
    }

    teams.push(team_name.to_string());
    println!("Updated teams list: {:?}", teams);

    let result = users
        .update_one(
            doc! { "email": email },
            doc! { "$set": { "teams": &teams } },
        )
        .await
        .map_err(|e| format!("Failed to update teams: {e}"))?;

    println!("Database update result: modified_count = {}", result.modified_count);

    if result.modified_count == 0 {
        return Err("Failed to update user teams in database".to_string());
    }

    println!("Successfully updated user teams in database");
    Ok(teams)
}

pub async fn delete_team(db: &Client, email: &str, team_name: &str) -> Result<Vec<String>, String> {
    let users = db.database("general").collection::<User>("users");

    let user = users
        .find_one(doc! { "email": email })
        .await
        .map_err(|e| format!("Failed to find user: {e}"))?
        .ok_or_else(|| "User not found".to_string())?;

    let mut teams = user.teams;

    if !teams.contains(&team_name.to_string()) {
        return Err("Not a member of this team".to_string());
    }

    teams.retain(|t| t != team_name);

    let result = users
        .update_one(
            doc! { "email": email },
            doc! { "$set": { "teams": &teams } },
        )
        .await
        .map_err(|e| format!("Failed to update teams: {e}"))?;

    if result.modified_count == 0 {
        return Err("Failed to update user teams in database".to_string());
    }

    Ok(teams)
}
