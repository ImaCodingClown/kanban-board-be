#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::slack::{SlackNotification, create_slack_message};

    #[test]
    fn test_create_slack_message_with_all_fields() {
        let notification = SlackNotification {
            slack_user_id: "U01ABC2DEF3".to_string(),
            card_title: "Test Card".to_string(),
            card_description: Some("Test description".to_string()),
            priority: Some("High".to_string()),
        };
        
        let message = create_slack_message(notification);
        
        assert!(message.contains_key("text"));
        assert!(message.contains_key("blocks"));
        
        let blocks = message.get("blocks").unwrap().as_array().unwrap();
        assert_eq!(blocks.len(), 2);
        
        let first_block = &blocks[0];
        assert_eq!(first_block["type"], "section");
        
        let text_content = first_block["text"]["text"].as_str().unwrap();
        assert!(text_content.contains("U01ABC2DEF3"));
        assert!(text_content.contains("Test Card"));
    }

    #[test]
    fn test_create_slack_message_with_minimal_fields() {
        let notification = SlackNotification {
            slack_user_id: "U01XYZ789".to_string(),
            card_title: "Minimal Card".to_string(),
            card_description: None,
            priority: None,
        };
        
        let message = create_slack_message(notification);
        
        assert!(message.contains_key("text"));
        assert!(message.contains_key("blocks"));
        
        let blocks = message.get("blocks").unwrap().as_array().unwrap();
        assert_eq!(blocks.len(), 2);
        
        let second_block = &blocks[1];
        let fields = second_block["fields"].as_array().unwrap();
        
        let description_field = &fields[0];
        assert!(description_field["text"].as_str().unwrap().contains("No description"));
        
        let priority_field = &fields[1];
        assert!(priority_field["text"].as_str().unwrap().contains("Not set"));
    }
}
