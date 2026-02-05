use crate::{
    models::{
        company::{
            Company, CompanyMemberWithUsername, CompanyRole, CompanyWithUsernames,
            CreateCompanyPayload, UpdateCompanyPayload,
        },
        users::User,
    },
    utils::errors::CustomError,
};
use futures::TryStreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId},
    Client,
};

pub async fn create_company(
    db: &Client,
    email: &str,
    payload: CreateCompanyPayload,
) -> Result<Company, CustomError> {
    let users = db.database("general").collection::<User>("users");
    let companies = db.database("general").collection::<Company>("companies");

    let user = users
        .find_one(doc! { "email": email })
        .await
        .map_err(|e| CustomError::Database(format!("Failed to find user: {}", e)))?;

    let user = user.ok_or_else(|| CustomError::NotFound("User not found".to_string()))?;

    // Check if user is Tony (owner)
    if user.username.to_lowercase() != "tony" {
        return Err(CustomError::Authentication(
            "Only Tony can create companies".to_string(),
        ));
    }

    let existing_company = companies
        .find_one(doc! { "name": &payload.name })
        .await
        .map_err(|e| CustomError::Database(format!("Failed to check company existence: {}", e)))?;

    if existing_company.is_some() {
        return Err(CustomError::Conflict(
            "Company name already exists".to_string(),
        ));
    }

    let company_name = payload.name.clone();
    let company = Company::new(payload.name, payload.description, user.id.unwrap());

    let result = companies
        .insert_one(&company)
        .await
        .map_err(|e| CustomError::Database(format!("Failed to create company: {}", e)))?;

    let mut created_company = company;
    created_company.id = Some(result.inserted_id.as_object_id().unwrap());

    // Add company to user's group
    let mut user_groups = user.group;
    user_groups.push(company_name);

    let _update_result = users
        .update_one(
            doc! { "email": email },
            doc! { "$set": { "group": &user_groups } },
        )
        .await
        .map_err(|e| CustomError::Database(format!("Failed to update user groups: {}", e)))?;

    Ok(created_company)
}

pub async fn get_company(
    db: &Client,
    company_id: &str,
) -> Result<Option<Company>, CustomError> {
    let companies = db.database("general").collection::<Company>("companies");

    let company_oid = ObjectId::parse_str(company_id)
        .map_err(|_| CustomError::NotFound("Invalid company ID".to_string()))?;

    let company = companies
        .find_one(doc! { "_id": company_oid })
        .await
        .map_err(|e| CustomError::Database(format!("Failed to get company: {}", e)))?;

    Ok(company)
}

pub async fn get_user_companies(db: &Client, email: &str) -> Result<Vec<Company>, CustomError> {
    let users = db.database("general").collection::<User>("users");
    let companies = db.database("general").collection::<Company>("companies");

    let user = users
        .find_one(doc! { "email": email })
        .await
        .map_err(|e| CustomError::Database(format!("Failed to find user: {}", e)))?;

    let user = user.ok_or_else(|| CustomError::NotFound("User not found".to_string()))?;

    let mut user_companies = Vec::new();

    // Get companies from user's group field
    for company_name in &user.group {
        let company = companies
            .find_one(doc! { "name": company_name })
            .await
            .map_err(|e| {
                CustomError::Database(format!(
                    "Failed to find company {}: {}",
                    company_name, e
                ))
            })?;

        if let Some(company) = company {
            user_companies.push(company);
        }
    }

    // Also check if user is a member of any companies
    let all_companies_cursor = companies
        .find(doc! {})
        .await
        .map_err(|e| CustomError::Database(format!("Failed to query companies: {}", e)))?;

    let all_companies: Vec<Company> = all_companies_cursor
        .try_collect()
        .await
        .map_err(|e| CustomError::Database(format!("Failed to collect companies: {}", e)))?;

    for company in all_companies {
        if company.is_member(&user.id.unwrap())
            && !user_companies.iter().any(|c| c.id == company.id)
        {
            user_companies.push(company);
        }
    }

    Ok(user_companies)
}

pub async fn update_company(
    db: &Client,
    email: &str,
    company_id: &str,
    payload: UpdateCompanyPayload,
) -> Result<Company, CustomError> {
    let users = db.database("general").collection::<User>("users");
    let companies = db.database("general").collection::<Company>("companies");

    let user = users
        .find_one(doc! { "email": email })
        .await
        .map_err(|e| CustomError::Database(format!("Failed to find user: {}", e)))?;

    let user = user.ok_or_else(|| CustomError::NotFound("User not found".to_string()))?;

    let company_oid = ObjectId::parse_str(company_id)
        .map_err(|_| CustomError::NotFound("Invalid company ID".to_string()))?;

    let mut company = companies
        .find_one(doc! { "_id": company_oid })
        .await
        .map_err(|e| CustomError::Database(format!("Failed to find company: {}", e)))?
        .ok_or_else(|| CustomError::NotFound("Company not found".to_string()))?;

    // Only owner (Tony) can update
    if !company.is_owner(&user.id.unwrap()) && user.username.to_lowercase() != "tony" {
        return Err(CustomError::Authentication(
            "Only company owner can update company".to_string(),
        ));
    }

    if let Some(new_name) = &payload.name {
        if new_name != &company.name {
            let existing_company = companies
                .find_one(doc! { "name": new_name })
                .await
                .map_err(|e| {
                    CustomError::Database(format!("Failed to check company existence: {}", e))
                })?;

            if existing_company.is_some() {
                return Err(CustomError::Conflict(
                    "Company name already exists".to_string(),
                ));
            }
        }
        company.name = new_name.clone();
    }

    if let Some(description) = &payload.description {
        company.description = Some(description.clone());
    }

    company.updated_at = chrono::Utc::now();

    let _update_result = companies
        .replace_one(doc! { "_id": company_oid }, &company)
        .await
        .map_err(|e| CustomError::Database(format!("Failed to update company: {}", e)))?;

    Ok(company)
}

pub async fn get_company_with_usernames(
    db: &Client,
    company_id: &str,
) -> Result<CompanyWithUsernames, CustomError> {
    let companies = db.database("general").collection::<Company>("companies");
    let users = db.database("general").collection::<User>("users");

    let company_oid = ObjectId::parse_str(company_id)
        .map_err(|_| CustomError::NotFound("Invalid company ID".to_string()))?;

    let company = companies
        .find_one(doc! { "_id": company_oid })
        .await
        .map_err(|e| CustomError::Database(format!("Failed to get company: {}", e)))?
        .ok_or_else(|| {
            return CustomError::NotFound("Company not found".to_string());
        })?;

    let owner = users
        .find_one(doc! { "_id": company.owner_id })
        .await
        .map_err(|e| CustomError::Database(format!("Failed to find owner: {}", e)))?
        .ok_or_else(|| CustomError::NotFound("Owner not found".to_string()))?;

    let mut members_with_usernames = Vec::new();

    for member in &company.members {
        if let Ok(Some(member_user)) = users
            .find_one(doc! { "_id": member.user_id })
            .await
        {
            members_with_usernames.push(CompanyMemberWithUsername {
                user_id: member.user_id.to_hex(),
                username: member_user.username.clone(),
                email: member_user.email.clone(),
                role: match member.role {
                    CompanyRole::Owner => "Owner".to_string(),
                    CompanyRole::Member => "Member".to_string(),
                },
                joined_at: member.joined_at.to_rfc3339(),
            });
        }
    }

    Ok(CompanyWithUsernames {
        _id: company.id.map(|id| id.to_hex()),
        name: company.name.clone(),
        description: company.description.clone(),
        owner_id: company.owner_id.to_hex(),
        owner_username: owner.username.clone(),
        members: members_with_usernames,
        created_at: company.created_at.to_rfc3339(),
        updated_at: company.updated_at.to_rfc3339(),
        is_active: company.is_active,
    })
}

pub async fn add_member(
    db: &Client,
    email: &str,
    company_id: &str,
    user_id: &str,
) -> Result<Company, CustomError> {
    let users = db.database("general").collection::<User>("users");
    let companies = db.database("general").collection::<Company>("companies");

    let requester = users
        .find_one(doc! { "email": email })
        .await
        .map_err(|e| CustomError::Database(format!("Failed to find requester: {}", e)))?;

    let requester = requester.ok_or_else(|| CustomError::NotFound("Requester not found".to_string()))?;

    let company_oid = ObjectId::parse_str(company_id)
        .map_err(|_| CustomError::NotFound("Invalid company ID".to_string()))?;

    let mut company = companies
        .find_one(doc! { "_id": company_oid })
        .await
        .map_err(|e| CustomError::Database(format!("Failed to find company: {}", e)))?
        .ok_or_else(|| CustomError::NotFound("Company not found".to_string()))?;

    // Only owner (Tony) can add members
    if !company.is_owner(&requester.id.unwrap()) && requester.username.to_lowercase() != "tony" {
        return Err(CustomError::Authentication(
            "Only company owner can add members".to_string(),
        ));
    }

    let new_member_id = ObjectId::parse_str(user_id)
        .map_err(|_| CustomError::NotFound("Invalid user_id format".to_string()))?;

    let new_member = users
        .find_one(doc! { "_id": new_member_id })
        .await
        .map_err(|e| CustomError::Database(format!("Failed to find user: {}", e)))?
        .ok_or_else(|| CustomError::NotFound("User not found".to_string()))?;

    // Check if user is already a member
    if company.is_member(&new_member_id) {
        return Err(CustomError::Conflict(
            "User is already a member of this company".to_string(),
        ));
    }

    // Add member to company
    company.add_member(new_member_id);

    // Add company to user's group if not already there
    let mut user_groups = new_member.group;
    if !user_groups.contains(&company.name) {
        user_groups.push(company.name.clone());
        
        let _update_result = users
            .update_one(
                doc! { "_id": new_member_id },
                doc! { "$set": { "group": &user_groups } },
            )
            .await
            .map_err(|e| CustomError::Database(format!("Failed to update user groups: {}", e)))?;
    }

    // Update company
    company.updated_at = chrono::Utc::now();
    let _update_result = companies
        .replace_one(doc! { "_id": company_oid }, &company)
        .await
        .map_err(|e| CustomError::Database(format!("Failed to update company: {}", e)))?;

    Ok(company)
}

pub async fn remove_member(
    db: &Client,
    email: &str,
    company_id: &str,
    user_id: &str,
) -> Result<Company, CustomError> {
    let users = db.database("general").collection::<User>("users");
    let companies = db.database("general").collection::<Company>("companies");

    let requester = users
        .find_one(doc! { "email": email })
        .await
        .map_err(|e| CustomError::Database(format!("Failed to find requester: {}", e)))?;

    let requester = requester.ok_or_else(|| CustomError::NotFound("Requester not found".to_string()))?;

    let company_oid = ObjectId::parse_str(company_id)
        .map_err(|_| CustomError::NotFound("Invalid company ID".to_string()))?;

    let mut company = companies
        .find_one(doc! { "_id": company_oid })
        .await
        .map_err(|e| CustomError::Database(format!("Failed to find company: {}", e)))?
        .ok_or_else(|| CustomError::NotFound("Company not found".to_string()))?;

    // Only owner (Tony) can remove members
    if !company.is_owner(&requester.id.unwrap()) && requester.username.to_lowercase() != "tony" {
        return Err(CustomError::Authentication(
            "Only company owner can remove members".to_string(),
        ));
    }

    let member_id = ObjectId::parse_str(user_id)
        .map_err(|_| CustomError::NotFound("Invalid user_id format".to_string()))?;

    // Cannot remove the owner
    if company.is_owner(&member_id) {
        return Err(CustomError::Authentication(
            "Cannot remove the company owner".to_string(),
        ));
    }

    // Check if user is a member
    if !company.is_member(&member_id) {
        return Err(CustomError::NotFound(
            "User is not a member of this company".to_string(),
        ));
    }

    // Remove member from company
    company.remove_member(&member_id);

    // Remove company from user's group
    let member_user = users
        .find_one(doc! { "_id": member_id })
        .await
        .map_err(|e| CustomError::Database(format!("Failed to find user: {}", e)))?
        .ok_or_else(|| CustomError::NotFound("User not found".to_string()))?;

    let mut user_groups = member_user.group;
    user_groups.retain(|g| g != &company.name);

    let _update_result = users
        .update_one(
            doc! { "_id": member_id },
            doc! { "$set": { "group": &user_groups } },
        )
        .await
        .map_err(|e| CustomError::Database(format!("Failed to update user groups: {}", e)))?;

    // Update company
    company.updated_at = chrono::Utc::now();
    let _update_result = companies
        .replace_one(doc! { "_id": company_oid }, &company)
        .await
        .map_err(|e| CustomError::Database(format!("Failed to update company: {}", e)))?;

    Ok(company)
}

pub async fn delete_company(
    db: &Client,
    email: &str,
    company_id: &str,
) -> Result<(), CustomError> {
    let users = db.database("general").collection::<User>("users");
    let companies = db.database("general").collection::<Company>("companies");

    let user = users
        .find_one(doc! { "email": email })
        .await
        .map_err(|e| CustomError::Database(format!("Failed to find user: {}", e)))?;

    let user = user.ok_or_else(|| CustomError::NotFound("User not found".to_string()))?;

    let company_oid = ObjectId::parse_str(company_id)
        .map_err(|_| CustomError::NotFound("Invalid company ID".to_string()))?;

    let company = companies
        .find_one(doc! { "_id": company_oid })
        .await
        .map_err(|e| CustomError::Database(format!("Failed to find company: {}", e)))?
        .ok_or_else(|| CustomError::NotFound("Company not found".to_string()))?;

    // Only owner (Tony) can delete
    if !company.is_owner(&user.id.unwrap()) && user.username.to_lowercase() != "tony" {
        return Err(CustomError::Authentication(
            "Only company owner can delete company".to_string(),
        ));
    }

    let _result = companies
        .delete_one(doc! { "_id": company_oid })
        .await
        .map_err(|e| CustomError::Database(format!("Failed to delete company: {}", e)))?;

    // Remove company from all users' groups
    let _update_result = users
        .update_many(
            doc! { "group": &company.name },
            doc! { "$pull": { "group": &company.name } },
        )
        .await
        .map_err(|e| {
            CustomError::Database(format!("Failed to remove company from user groups: {}", e))
        })?;

    Ok(())
}

