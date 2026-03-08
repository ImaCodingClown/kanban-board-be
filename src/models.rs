pub mod auth;
pub mod cards;
pub mod slack;
pub mod teams;
pub mod users;

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum CompanyRole {
    Owner,
    Member,
    Leader,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CompanyMember {
    pub user_id: ObjectId,
    pub role: CompanyRole,
    pub joined_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Company {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub description: Option<String>,
    pub leader_id: ObjectId,
    pub members: Vec<CompanyMember>,
    pub created_at: String,
    pub updated_at: String,
    pub is_active: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slack_webhook_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_card_number: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub card_prefix: Option<String>,
}

impl Company {
    pub fn new(name: String, description: Option<String>, leader_id: ObjectId) -> Self {
        let owner_member = CompanyMember {
            user_id: leader_id,
            role: CompanyRole::Owner,
            joined_at: chrono::Utc::now().to_rfc3339(),
        };
        Company {
            id: None,
            name,
            description,
            leader_id,
            members: vec![owner_member],
            created_at: chrono::Utc::now().to_rfc3339(),  
            updated_at: chrono::Utc::now().to_rfc3339(),
            is_active: true,
            slack_webhook_url: None,  
            next_card_number: None,   
            card_prefix: None   
        }
    }

    pub fn add_member(&mut self, user_id: ObjectId) {
        let member = CompanyMember {
            user_id,
            role: CompanyRole::Member,
            joined_at: chrono::Utc::now().to_rfc3339(),
        };
        if !self.members.iter().any(|m| m.user_id == user_id) {
            self.members.push(member);
        }
    }

    pub fn remove_member(&mut self, user_id: &ObjectId) {
        self.members.retain(|member| &member.user_id != user_id);
    }

    pub fn is_owner(&self, user_id: &ObjectId) -> bool {
        self.leader_id == *user_id
    }

    pub fn is_member(&self, user_id: &ObjectId) -> bool {
        self.members.iter().any(|member| &member.user_id == user_id)
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateCompanyPayload {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCompanyPayload {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddMemberPayload {
    pub user_id: String,
}

#[derive(Debug, Deserialize)]
pub struct RemoveMemberPayload {
    pub user_id: String,
}

#[derive(Debug, Serialize)]
pub struct CompanyResponse {
    pub success: bool,
    pub company: Option<Company>,
    pub message: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CompaniesResponse {
    pub success: bool,
    pub companies: Vec<Company>,
    pub message: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CompanyMemberWithUsername {
    pub user_id: String,
    pub username: String,
    pub email: String,
    pub role: String,
    pub joined_at: String,
}

#[derive(Debug, Serialize)]
pub struct CompanyWithUsernames {
    pub _id: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub owner_id: String,
    pub owner_username: String,
    pub members: Vec<CompanyMemberWithUsername>,
    pub created_at: String,
    pub updated_at: String,
    pub is_active: bool,
}

#[derive(Debug, Serialize)]
pub struct CompanyWithUsernamesResponse {
    pub success: bool,
    pub company: Option<CompanyWithUsernames>,
    pub message: Option<String>,
}