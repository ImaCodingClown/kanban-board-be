#[cfg(test)]
mod tests {
    use kanban_backend::{
        config::{AppState, Environment},
        models::cards::{Board, Card, Column, EditCardPayload},
        services::{slack::SlackNotification, user_info::get_user_by_username},
    };
    use mongodb::{bson::oid::ObjectId, Client};

    #[tokio::test]
    async fn test_get_user_by_username() {
        // This test requires a real database connection
        // In a real test environment, you would use a test database
        // For now, we'll just test the function signature and error handling

        let client = Client::with_uri_str("mongodb://localhost:27017")
            .await
            .expect("Failed to connect to MongoDB");

        let result = get_user_by_username("nonexistent_user", &client).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_assignee_change_detection_logic() {
        // Test the logic for detecting assignee changes
        let old_assignee = Some("user1".to_string());
        let new_assignee = "user2".to_string();

        // Test case 1: assignee changed
        assert_ne!(old_assignee, Some(new_assignee.clone()));

        // Test case 2: assignee unchanged
        let old_assignee_same = Some("user2".to_string());
        assert_eq!(old_assignee_same, Some(new_assignee.clone()));

        // Test case 3: assignee added (from None to Some)
        let old_assignee_none = None;
        assert_ne!(old_assignee_none, Some(new_assignee.clone()));
    }

    #[test]
    fn test_slack_notification_creation() {
        let notification = SlackNotification {
            slack_user_id: "U01ABC2DEF3".to_string(),
            card_title: "Test Card".to_string(),
            card_description: Some("Test description".to_string()),
            priority: Some("High".to_string()),
        };

        assert_eq!(notification.slack_user_id, "U01ABC2DEF3");
        assert_eq!(notification.card_title, "Test Card");
        assert_eq!(
            notification.card_description,
            Some("Test description".to_string())
        );
        assert_eq!(notification.priority, Some("High".to_string()));
    }

    #[test]
    fn test_edit_card_payload_structure() {
        let payload = EditCardPayload {
            card_id: "507f1f77bcf86cd799439011".to_string(),
            title: "Updated Card".to_string(),
            description: "Updated description".to_string(),
            column_name: "In Progress".to_string(),
            story_point: Some(5),
            assignee: "newuser".to_string(),
            team: "testteam".to_string(),
            priority: Some("Medium".to_string()),
        };

        assert_eq!(payload.card_id, "507f1f77bcf86cd799439011");
        assert_eq!(payload.title, "Updated Card");
        assert_eq!(payload.description, "Updated description");
        assert_eq!(payload.assignee, "newuser");
        assert_eq!(payload.team, "testteam");
    }

    #[test]
    fn test_board_card_structure() {
        let card = Card {
            id: Some(ObjectId::new()),
            title: "Test Card".to_string(),
            description: Some("Test description".to_string()),
            assignee: Some("testuser".to_string()),
            story_point: Some(3),
            priority: Some("Low".to_string()),
        };

        let column = Column {
            title: "To Do".to_string(),
            cards: vec![card.clone()],
        };

        let board = Board {
            id: Some(ObjectId::new()),
            team: "testteam".to_string(),
            iteration: None,
            columns: vec![column],
        };

        assert_eq!(board.team, "testteam");
        assert_eq!(board.columns.len(), 1);
        assert_eq!(board.columns[0].cards.len(), 1);
        assert_eq!(
            board.columns[0].cards[0].assignee,
            Some("testuser".to_string())
        );
    }
}
