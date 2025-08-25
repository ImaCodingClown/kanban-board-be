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
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_active: bool,
}

impl Team {
    pub fn new(name: String, description: Option<String>, leader_id: ObjectId) -> Self {
        let now = Utc::now();
        let leader_member = TeamMember {
            user_id: leader_id.clone(),
            role: TeamRole::Leader,
            joined_at: now,
            permissions: vec!["read".to_string(), "write".to_string(), "delete".to_string(), "manage_members".to_string()],
        };

        Self {
            id: None,
            name,
            description,
            leader_id,
            members: vec![leader_member],
            created_at: now,
            updated_at: now,
            is_active: true,
        }
    }

    pub fn add_member(&mut self, user_id: ObjectId, role: TeamRole) {
        let member = TeamMember {
            user_id,
            role,
            joined_at: Utc::now(),
            permissions: match role {
                TeamRole::Leader => vec!["read".to_string(), "write".to_string(), "delete".to_string(), "manage_members".to_string()],
                TeamRole::Collaborator => vec!["read".to_string(), "write".to_string()],
            },
        };
        self.members.push(member);
        self.updated_at = Utc::now();
    }

    pub fn remove_member(&mut self, user_id: &ObjectId) -> bool {
        let initial_len = self.members.len();
        self.members.retain(|member| member.user_id != *user_id);
        let removed = self.members.len() < initial_len;
        if removed {
            self.updated_at = Utc::now();
        }
        removed
    }

    pub fn is_leader(&self, user_id: &ObjectId) -> bool {
        self.leader_id == *user_id
    }

    pub fn is_member(&self, user_id: &ObjectId) -> bool {
        self.members.iter().any(|member| member.user_id == *user_id)
    }

    pub fn can_edit(&self, user_id: &ObjectId) -> bool {
        self.is_leader(user_id) || self.members.iter().any(|member| {
            member.user_id == *user_id && member.permissions.contains(&"write".to_string())
        })
    }

    pub fn can_manage_members(&self, user_id: &ObjectId) -> bool {
        self.is_leader(user_id) || self.members.iter().any(|member| {
            member.user_id == *user_id && member.permissions.contains(&"manage_members".to_string())
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTeamPayload {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateTeamPayload {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddMemberPayload {
    pub user_email: String,
    pub role: TeamRole,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RemoveMemberPayload {
    pub user_email: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TeamResponse {
    pub success: bool,
    pub team: Option<Team>,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TeamsResponse {
    pub success: bool,
    pub teams: Vec<Team>,
    pub message: String,
}
