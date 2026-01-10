use mongodb::{bson::doc, Client};

use crate::{
    models::{cards::Board, teams::Team, users::User},
    utils::errors::CustomError,
};

pub async fn get_user_by_email(db: &Client, email: &str) -> Result<User, CustomError> {
    let users = db.database("general").collection::<User>("users");

    users
        .find_one(doc! { "email": email })
        .await
        .map_err(|e| CustomError::Database(format!("Failed to find user: {}", e)))?
        .ok_or_else(|| CustomError::NotFound("User not found".to_string()))
}

pub async fn get_team_by_name(db: &Client, team_name: &str) -> Result<Team, CustomError> {
    let teams = db.database("general").collection::<Team>("teams");

    teams
        .find_one(doc! { "name": team_name })
        .await
        .map_err(|e| CustomError::Database(format!("Failed to find team: {}", e)))?
        .ok_or_else(|| CustomError::NotFound("Team not found".to_string()))
}

pub async fn check_team_membership(
    db: &Client,
    user_email: &str,
    team_name: &str,
) -> Result<bool, CustomError> {
    let user = get_user_by_email(db, user_email).await?;
    let team = get_team_by_name(db, team_name).await?;

    let user_id = user
        .id
        .ok_or_else(|| CustomError::Server("User ID not found".to_string()))?;

    Ok(team.is_member(&user_id))
}

pub async fn require_team_membership(
    db: &Client,
    user_email: &str,
    team_name: &str,
) -> Result<(), CustomError> {
    if !check_team_membership(db, user_email, team_name).await? {
        return Err(CustomError::Forbidden(format!(
            "You are not a member of team '{}'",
            team_name
        )));
    }
    Ok(())
}

pub async fn check_team_permission(
    db: &Client,
    user_email: &str,
    team_name: &str,
    permission: &str,
) -> Result<bool, CustomError> {
    let user = get_user_by_email(db, user_email).await?;
    let team = get_team_by_name(db, team_name).await?;

    let user_id = user
        .id
        .ok_or_else(|| CustomError::Server("User ID not found".to_string()))?;

    let member = team.members.iter().find(|m| m.user_id == user_id);

    match member {
        Some(m) => Ok(m.permissions.contains(&permission.to_string())),
        None => Ok(false),
    }
}

pub async fn require_team_permission(
    db: &Client,
    user_email: &str,
    team_name: &str,
    permission: &str,
) -> Result<(), CustomError> {
    if !check_team_permission(db, user_email, team_name, permission).await? {
        return Err(CustomError::Forbidden(format!(
            "You don't have '{}' permission for team '{}'",
            permission, team_name
        )));
    }
    Ok(())
}

pub async fn check_board_access(
    db: &Client,
    user_email: &str,
    board_id: &str,
) -> Result<Board, CustomError> {
    let boards = db.database("general").collection::<Board>("boards");

    let board_oid = mongodb::bson::oid::ObjectId::parse_str(board_id)
        .map_err(|_| CustomError::NotFound("Invalid board ID".to_string()))?;

    let board = boards
        .find_one(doc! { "_id": board_oid })
        .await
        .map_err(|e| CustomError::Database(format!("Failed to find board: {}", e)))?
        .ok_or_else(|| CustomError::NotFound("Board not found".to_string()))?;

    require_team_membership(db, user_email, &board.team).await?;

    Ok(board)
}

pub async fn require_board_write_permission(
    db: &Client,
    user_email: &str,
    board_id: &str,
) -> Result<Board, CustomError> {
    let board = check_board_access(db, user_email, board_id).await?;

    require_team_permission(db, user_email, &board.team, "write").await?;

    Ok(board)
}

pub async fn check_is_team_leader(
    db: &Client,
    user_email: &str,
    team_name: &str,
) -> Result<bool, CustomError> {
    let user = get_user_by_email(db, user_email).await?;
    let team = get_team_by_name(db, team_name).await?;

    let user_id = user
        .id
        .ok_or_else(|| CustomError::Server("User ID not found".to_string()))?;

    Ok(team.is_leader(&user_id))
}

pub async fn require_team_leader(
    db: &Client,
    user_email: &str,
    team_name: &str,
) -> Result<(), CustomError> {
    if !check_is_team_leader(db, user_email, team_name).await? {
        return Err(CustomError::Forbidden(
            "Only team leader can perform this action".to_string(),
        ));
    }
    Ok(())
}
