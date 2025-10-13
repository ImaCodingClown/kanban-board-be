#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::users::{User, UpdateSlackIdPayload};

    #[test]
    fn test_user_create_with_slack_id_field() {
        let user = User::create(
            "testuser".to_string(),
            "test@example.com".to_string(),
            "hashed_password".to_string(),
            vec!["team1".to_string()],
        );
        
        assert_eq!(user.username, "testuser");
        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.password_hash, "hashed_password");
        assert_eq!(user.teams, vec!["team1"]);
        assert_eq!(user.slack_user_id, None);
        assert_eq!(user.group.len(), 0);
        assert_eq!(user.permissions.len(), 0);
    }

    #[test]
    fn test_update_slack_id_payload_creation() {
        let payload = UpdateSlackIdPayload {
            slack_user_id: "U01ABC2DEF3".to_string(),
        };
        
        assert_eq!(payload.slack_user_id, "U01ABC2DEF3");
    }

    #[test]
    fn test_user_serialization_with_slack_id() {
        let mut user = User::create(
            "testuser".to_string(),
            "test@example.com".to_string(),
            "hashed_password".to_string(),
            vec!["team1".to_string()],
        );
        user.slack_user_id = Some("U01ABC2DEF3".to_string());
        
        let json = serde_json::to_string(&user).unwrap();
        assert!(json.contains("U01ABC2DEF3"));
        assert!(json.contains("slack_user_id"));
    }

    #[test]
    fn test_user_deserialization_with_slack_id() {
        let json = r#"{
            "username": "testuser",
            "email": "test@example.com",
            "password_hash": "hashed_password",
            "group": [],
            "permissions": [],
            "teams": ["team1"],
            "slack_user_id": "U01ABC2DEF3"
        }"#;
        
        let user: User = serde_json::from_str(json).unwrap();
        assert_eq!(user.username, "testuser");
        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.slack_user_id, Some("U01ABC2DEF3".to_string()));
    }
}
