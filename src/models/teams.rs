use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum TeamRole {
    Leader,
    Collaborator,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TeamMember {
    pub user_id: ObjectId,
    pub username: String,
    pub role: TeamRole,
    pub joined_at: DateTime<Utc>,
    pub permissions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Team {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub description: Option<String>,
    pub leader_id: ObjectId,
    pub members: Vec<TeamMember>,
}

impl Team {
    pub fn new(name: String, description: Option<String>, leader_id: ObjectId, username: String) -> Self {
        let leader_member = TeamMember {
            user_id: leader_id,
            username: username,
            role: TeamRole::Leader,
            joined_at: chrono::Utc::now(),
            permissions: vec!["read".to_string(), "write".to_string(), "delete".to_string(), "manage_members".to_string()],
        };

        Team {
            id: None,
            name,
            description,
            leader_id,
            members: vec![leader_member],
        }
    }

    pub fn add_member(&mut self, user_id: ObjectId, username: String, role: TeamRole) {
        let permissions = match role {
            TeamRole::Leader => vec!["read".to_string(), "write".to_string(), "delete".to_string(), "manage_members".to_string()],
            TeamRole::Collaborator => vec!["read".to_string(), "write".to_string()],
        };

        let member = TeamMember {
            user_id,
            username,
            role,
            joined_at: chrono::Utc::now(),
            permissions,
        };

        self.members.push(member);
    }

    pub fn remove_member(&mut self, user_id: &ObjectId) {
        self.members.retain(|member| &member.user_id != user_id);
    }

    pub fn is_leader(&self, user_id: &ObjectId) -> bool {
        self.leader_id == *user_id
    }

    pub fn is_member(&self, user_id: &ObjectId) -> bool {
        self.members.iter().any(|member| &member.user_id == user_id)
    }

    pub fn get_member_role(&self, user_id: &ObjectId) -> Option<TeamRole> {
        self.members
            .iter()
            .find(|member| &member.user_id == user_id)
            .map(|member| member.role)
    }

    pub fn can_edit(&self, user_id: &ObjectId) -> bool {
        if let Some(member) = self.members.iter().find(|m| &m.user_id == user_id) {
            member.permissions.contains(&"write".to_string())
        } else {
            false
        }
    }

    pub fn can_manage_members(&self, user_id: &ObjectId) -> bool {
        if let Some(member) = self.members.iter().find(|m| &m.user_id == user_id) {
            member.permissions.contains(&"manage_members".to_string())
        } else {
            false
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateTeamPayload {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTeamPayload {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddMemberPayload {
    pub username: String,
    pub user_email: String,
    pub role: TeamRole,
}

#[derive(Debug, Deserialize)]
pub struct RemoveMemberPayload {
    pub user_email: String,
}

#[derive(Debug, Serialize)]
pub struct TeamResponse {
    pub success: bool,
    pub team: Option<Team>,
    pub message: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TeamsResponse {
    pub success: bool,
    pub teams: Vec<Team>,
    pub message: Option<String>,
}