use crate::models::{teams::{Team, CreateTeamPayload, UpdateTeamPayload, AddMemberPayload, RemoveMemberPayload, TeamRole}, users::User, cards::Board};
use crate::utils::errors::CustomError;
use mongodb::{bson::{doc, oid::ObjectId}, Client};

pub async fn create_team(db: &Client, email: &str, payload: CreateTeamPayload) -> Result<Team, CustomError> {
    let users = db.database("general").collection::<User>("users");
    let teams = db.database("general").collection::<Team>("teams");

    let user = users
        .find_one(doc! { "email": email })
        .await
        .map_err(|e| CustomError::Database(format!("Failed to find user: {}", e)))?;

    let user = user.ok_or_else(|| CustomError::NotFound("User not found".to_string()))?;

    let existing_team = teams
        .find_one(doc! { "name": &payload.name })
        .await
        .map_err(|e| CustomError::Database(format!("Failed to check team existence: {}", e)))?;

    if existing_team.is_some() {
        return Err(CustomError::Conflict("Team name already exists".to_string()));
    }

    let team_name = payload.name.clone();
    let team = Team::new(payload.name, payload.description, user.id.unwrap());

    let result = teams
        .insert_one(&team)
        .await
        .map_err(|e| CustomError::Database(format!("Failed to create team: {}", e)))?;

    let mut created_team = team;
    created_team.id = Some(result.inserted_id.as_object_id().unwrap());

    let mut user_teams = user.teams;
    user_teams.push(team_name);

    let _update_result = users
        .update_one(
            doc! { "email": email },
            doc! { "$set": { "teams": &user_teams } },
        )
        .await
        .map_err(|e| CustomError::Database(format!("Failed to update user teams: {}", e)))?;

    Ok(created_team)
}

pub async fn add_user_to_team(db: &Client, team_name: &str, user_id: ObjectId, role: TeamRole) -> Result<(), CustomError> {
    let teams = db.database("general").collection::<Team>("teams");
    
    let mut team = teams
        .find_one(doc! { "name": team_name })
        .await
        .map_err(|e| CustomError::Database(format!("Failed to find team {}: {}", team_name, e)))?
        .ok_or_else(|| CustomError::NotFound(format!("Team {} not found", team_name)))?;

    team.add_member(user_id, role);

    let _result = teams
        .replace_one(doc! { "name": team_name }, &team)
        .await
        .map_err(|e| CustomError::Database(format!("Failed to update team {}: {}", team_name, e)))?;

    Ok(())
}

pub async fn add_user_to_ljy_team(db: &Client, user_id: ObjectId, _email: &str) -> Result<(), CustomError> {
    add_user_to_team(db, "LJY Members", user_id, TeamRole::Collaborator).await
}

pub async fn get_team(db: &Client, team_name: &str) -> Result<Option<Team>, CustomError> {
    let teams = db.database("general").collection::<Team>("teams");
    
    let team = teams
        .find_one(doc! { "name": team_name })
        .await
        .map_err(|e| CustomError::Database(format!("Failed to get team: {}", e)))?;

    Ok(team)
}

pub async fn get_user_teams(db: &Client, email: &str) -> Result<Vec<Team>, CustomError> {
    let users = db.database("general").collection::<User>("users");
    let teams = db.database("general").collection::<Team>("teams");

    let user = users
        .find_one(doc! { "email": email })
        .await
        .map_err(|e| CustomError::Database(format!("Failed to find user: {}", e)))?;

    let user = user.ok_or_else(|| CustomError::NotFound("User not found".to_string()))?;

    let mut user_teams = Vec::new();

    for team_name in &user.teams {
        let team = teams
            .find_one(doc! { "name": team_name })
            .await
            .map_err(|e| CustomError::Database(format!("Failed to find team {}: {}", team_name, e)))?;

        if let Some(team) = team {
            user_teams.push(team);
        }
    }

    Ok(user_teams)
}

pub async fn update_team(db: &Client, email: &str, team_name: &str, payload: UpdateTeamPayload) -> Result<Team, CustomError> {
    let users = db.database("general").collection::<User>("users");
    let teams = db.database("general").collection::<Team>("teams");

    let user = users
        .find_one(doc! { "email": email })
        .await
        .map_err(|e| CustomError::Database(format!("Failed to find user: {}", e)))?;

    let user = user.ok_or_else(|| CustomError::NotFound("User not found".to_string()))?;

    let mut team = teams
        .find_one(doc! { "name": team_name })
        .await
        .map_err(|e| CustomError::Database(format!("Failed to find team: {}", e)))?
        .ok_or_else(|| CustomError::NotFound("Team not found".to_string()))?;

    if !team.is_leader(&user.id.unwrap()) {
        return Err(CustomError::Authentication("Only team leader can update team".to_string()));
    }

    if let Some(new_name) = &payload.name {
        if new_name != team_name {
            let existing_team = teams
                .find_one(doc! { "name": new_name })
                .await
                .map_err(|e| CustomError::Database(format!("Failed to check team existence: {}", e)))?;

            if existing_team.is_some() {
                return Err(CustomError::Conflict("Team name already exists".to_string()));
            }
        }
        team.name = new_name.clone();
    }

    if let Some(description) = &payload.description {
        team.description = Some(description.clone());
    }

    let _update_result = teams
        .replace_one(doc! { "name": team_name }, &team)
        .await
        .map_err(|e| CustomError::Database(format!("Failed to update team: {}", e)))?;

    Ok(team)
}

pub async fn add_member(db: &Client, email: &str, team_name: &str, payload: AddMemberPayload) -> Result<Team, CustomError> {
    let users = db.database("general").collection::<User>("users");
    let teams = db.database("general").collection::<Team>("teams");

    let leader = users
        .find_one(doc! { "email": email })
        .await
        .map_err(|e| CustomError::Database(format!("Failed to find leader: {}", e)))?;

    let leader = leader.ok_or_else(|| CustomError::NotFound("Leader not found".to_string()))?;

    let new_member = users
        .find_one(doc! { "email": &payload.user_email })
        .await
        .map_err(|e| CustomError::Database(format!("Failed to find new member: {}", e)))?;

    let new_member = new_member.ok_or_else(|| CustomError::NotFound("New member not found".to_string()))?;

    let mut team = teams
        .find_one(doc! { "name": team_name })
        .await
        .map_err(|e| CustomError::Database(format!("Failed to find team: {}", e)))?
        .ok_or_else(|| CustomError::NotFound("Team not found".to_string()))?;

    if !team.is_leader(&leader.id.unwrap()) {
        return Err(CustomError::Authentication("Only team leader can add members".to_string()));
    }

    if team.is_member(&new_member.id.unwrap()) {
        return Err(CustomError::Conflict("User is already a member of this team".to_string()));
    }

    team.add_member(new_member.id.unwrap(), payload.role.clone());

    let _update_result = teams
        .replace_one(doc! { "name": team_name }, &team)
        .await
        .map_err(|e| CustomError::Database(format!("Failed to update team: {}", e)))?;

    let mut member_teams = new_member.teams;
    if !member_teams.contains(&team_name.to_string()) {
        member_teams.push(team_name.to_string());
        
        let _user_update_result = users
            .update_one(
                doc! { "email": &payload.user_email },
                doc! { "$set": { "teams": &member_teams } },
            )
            .await
            .map_err(|e| CustomError::Database(format!("Failed to update member teams: {}", e)))?;
    }

    Ok(team)
}

pub async fn remove_member(db: &Client, email: &str, team_name: &str, payload: RemoveMemberPayload) -> Result<Team, CustomError> {
    let users = db.database("general").collection::<User>("users");
    let teams = db.database("general").collection::<Team>("teams");

    let leader = users
        .find_one(doc! { "email": email })
        .await
        .map_err(|e| CustomError::Database(format!("Failed to find leader: {}", e)))?;

    let leader = leader.ok_or_else(|| CustomError::NotFound("Leader not found".to_string()))?;

    let member_to_remove = users
        .find_one(doc! { "email": &payload.user_email })
        .await
        .map_err(|e| CustomError::Database(format!("Failed to find member: {}", e)))?;

    let member_to_remove = member_to_remove.ok_or_else(|| CustomError::NotFound("Member not found".to_string()))?;

    let mut team = teams
        .find_one(doc! { "name": team_name })
        .await
        .map_err(|e| CustomError::Database(format!("Failed to find team: {}", e)))?
        .ok_or_else(|| CustomError::NotFound("Team not found".to_string()))?;

    if !team.is_leader(&leader.id.unwrap()) {
        return Err(CustomError::Authentication("Only team leader can remove members".to_string()));
    }

    if !team.is_member(&member_to_remove.id.unwrap()) {
        return Err(CustomError::NotFound("User is not a member of this team".to_string()));
    }

    if team.is_leader(&member_to_remove.id.unwrap()) {
        return Err(CustomError::Conflict("Cannot remove team leader".to_string()));
    }

    team.remove_member(&member_to_remove.id.unwrap());

    let _update_result = teams
        .replace_one(doc! { "name": team_name }, &team)
        .await
        .map_err(|e| CustomError::Database(format!("Failed to update team: {}", e)))?;

    let mut member_teams = member_to_remove.teams;
    member_teams.retain(|t| t != team_name);

    let _user_update_result = users
        .update_one(
            doc! { "email": &payload.user_email },
            doc! { "$set": { "teams": &member_teams } },
        )
        .await
        .map_err(|e| CustomError::Database(format!("Failed to update member teams: {}", e)))?;
    Ok(team)
}

pub async fn leave_team(db: &Client, email: &str, team_name: &str) -> Result<(), CustomError> {
    let users = db.database("general").collection::<User>("users");
    let teams = db.database("general").collection::<Team>("teams");

    let user = users
        .find_one(doc! { "email": email })
        .await
        .map_err(|e| CustomError::Database(format!("Failed to find user: {}", e)))?;

    let user = user.ok_or_else(|| CustomError::NotFound("User not found".to_string()))?;

    let mut team = teams
        .find_one(doc! { "name": team_name })
        .await
        .map_err(|e| CustomError::Database(format!("Failed to find team: {}", e)))?
        .ok_or_else(|| CustomError::NotFound("Team not found".to_string()))?;

    if !team.is_member(&user.id.unwrap()) {
        return Err(CustomError::NotFound("User is not a member of this team".to_string()));
    }

    if team.is_leader(&user.id.unwrap()) {
        return Err(CustomError::Conflict("Team leader cannot leave team. Transfer leadership or delete team instead".to_string()));
    }

    team.remove_member(&user.id.unwrap());

    let _update_result = teams
        .replace_one(doc! { "name": team_name }, &team)
        .await
        .map_err(|e| CustomError::Database(format!("Failed to update team: {}", e)))?;

    let mut user_teams = user.teams;
    user_teams.retain(|t| t != team_name);

    let _user_update_result = users
        .update_one(
            doc! { "email": email },
            doc! { "$set": { "teams": &user_teams } },
        )
        .await
        .map_err(|e| CustomError::Database(format!("Failed to update user teams: {}", e)))?;
    Ok(())
}
pub async fn delete_team(db: &Client, email: &str, team_name: &str) -> Result<(), CustomError> {
    let users = db.database("general").collection::<User>("users");
    let teams = db.database("general").collection::<Team>("teams");
    let boards = db.database("general").collection::<Board>("boards");

    let user = users
        .find_one(doc! { "email": email })
        .await
        .map_err(|e| CustomError::Database(format!("Failed to find user: {}", e)))?;

    let user = user.ok_or_else(|| CustomError::NotFound("User not found".to_string()))?;

    let team = teams
        .find_one(doc! { "name": team_name })
        .await
        .map_err(|e| CustomError::Database(format!("Failed to find team: {}", e)))?
        .ok_or_else(|| CustomError::NotFound("Team not found".to_string()))?;

    if !team.is_leader(&user.id.unwrap()) {
        return Err(CustomError::Authentication("Only team leader can delete team".to_string()));
    }

    let _board_result = boards
        .delete_many(doc! { "team": team_name })
        .await
        .map_err(|e| CustomError::Database(format!("Failed to delete team boards: {}", e)))?;

    let _team_result = teams
        .delete_one(doc! { "name": team_name })
        .await
        .map_err(|e| CustomError::Database(format!("Failed to delete team: {}", e)))?;

    let _update_result = users
        .update_many(
            doc! { "teams": team_name },
            doc! { "$pull": { "teams": team_name } },
        )
        .await
        .map_err(|e| CustomError::Database(format!("Failed to remove team from users: {}", e)))?;

    Ok(())
}