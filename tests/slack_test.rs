#[cfg(test)]
mod tests {
    use kanban_backend::models::slack::SlackNotificationPayload;
    use kanban_backend::services::slack::create_slack_message;

    #[test]
    fn test_create_slack_message_with_all_fields() {
        let notification = SlackNotificationPayload {
            slack_user_id: "U01ABC2DEF3".to_string(),
            card_title: "Test Card",
            card_description: Some("Test description"),
            priority: Some("High"),
        };

        let message = create_slack_message(notification);

        assert!(message.contains_key("text"));
        assert!(message.contains_key("blocks"));

        let blocks = message.get("blocks").unwrap().as_array().unwrap();
        assert_eq!(blocks.len(), 4);

        // First block: user assignment message
        let first_block = &blocks[0];
        assert_eq!(first_block["type"], "section");
        let text_content = first_block["text"]["text"].as_str().unwrap();
        assert!(text_content.contains("U01ABC2DEF3"));
        assert!(text_content.contains("has been assigned to a card"));

        // Second block: divider
        let second_block = &blocks[1];
        assert_eq!(second_block["type"], "divider");

        // Third block: card title
        let third_block = &blocks[2];
        assert_eq!(third_block["type"], "section");
        let title_content = third_block["text"]["text"].as_str().unwrap();
        assert!(title_content.contains("Test Card"));
    }

    #[test]
    fn test_create_slack_message_with_minimal_fields() {
        let notification = SlackNotificationPayload {
            slack_user_id: "U01XYZ789".to_string(),
            card_title: "Minimal Card",
            card_description: None,
            priority: None,
        };

        let message = create_slack_message(notification);

        assert!(message.contains_key("text"));
        assert!(message.contains_key("blocks"));

        let blocks = message.get("blocks").unwrap().as_array().unwrap();
        assert_eq!(blocks.len(), 4);

        // Fourth block: description and priority fields
        let fourth_block = &blocks[3];
        assert_eq!(fourth_block["type"], "section");
        let fields = fourth_block["fields"].as_array().unwrap();

        let description_field = &fields[0];
        assert!(description_field["text"]
            .as_str()
            .unwrap()
            .contains("No description"));

        let priority_field = &fields[1];
        assert!(priority_field["text"].as_str().unwrap().contains("Not set"));
    }
}
