use serde_json::json;
use std::collections::HashMap;

#[derive(Debug)]
pub struct SlackNotification {
    pub slack_user_id: String,
    pub card_title: String,
    pub card_description: Option<String>,
    pub priority: Option<String>,
}

pub async fn send_assignee_notification(
    webhook_url: &str,
    notification: SlackNotification,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::new();

    let message = create_slack_message(notification);

    let response = match client.post(webhook_url).json(&message).send().await {
        Ok(res) => res,
        Err(e) => {
            eprintln!("Error sending request to Slack: {}", e);
            return Err(Box::new(e));
        }
    };

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        eprintln!("Slack webhook failed: {} - {}", status, body);
        return Err(format!("Slack webhook failed: {} - {}", status, body).into());
    }

    Ok(())
}

pub fn create_slack_message(notification: SlackNotification) -> HashMap<String, serde_json::Value> {
    let mut message = HashMap::new();

    message.insert(
        "text".to_string(),
        json!(format!(
            "Hey <@{}>, you've been assigned to a card!",
            notification.slack_user_id
        )),
    );

    let mut blocks = vec![
        json!({
            "type": "section",
            "text": {
                "type": "mrkdwn",
                "text": format!("*<@{}> has been assigned to a card!*", notification.slack_user_id)
            }
        }),
        json!({
            "type": "divider"
        }),
        json!({
            "type": "section",
            "fields": [
                {
                    "type": "mrkdwn",
                    "text": format!("*Card:*\n{}", notification.card_title)
                },
                {
                    "type": "mrkdwn",
                    "text": format!("*Priority:*\n{}",
                        notification.priority.as_deref().unwrap_or("N/A"))
                }
            ]
        }),
    ];

    if let Some(description) = &notification.card_description {
        blocks.push(json!({
            "type": "section",
            "text": {
                "type": "mrkdwn",
                "text": format!("*Description:*\n{}", description)
            }
        }));
    }

    message.insert("blocks".to_string(), json!(blocks));
    // Remove channel field to send to default webhook channel
    // message.insert("channel".to_string(), json!(notification.slack_user_id));

    message
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_slack_message() {
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
    }
}
