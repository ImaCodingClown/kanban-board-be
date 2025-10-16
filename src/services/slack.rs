use crate::models::slack::SlackNotificationPayload;
use crate::utils::errors::CustomError;
use serde_json::json;
use std::collections::HashMap;

#[async_trait::async_trait]
pub trait SlackNotifier: Send + Sync {
    fn build_message(
        &self,
        payload: &SlackNotificationPayload,
    ) -> HashMap<String, serde_json::Value>;
    async fn send(
        &self,
        webhook_url: &str,
        payload: SlackNotificationPayload,
    ) -> Result<(), CustomError>;
}

pub async fn send_assignee_notification(
    webhook_url: &str,
    notification: SlackNotification,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::new();

    let message = create_slack_message(notification);

    let response = client.post(webhook_url).json(&message).send().await?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("Slack webhook failed: {} - {}", status, body).into());
    }

    Ok(())
}

fn create_slack_message(notification: SlackNotification) -> HashMap<String, serde_json::Value> {
    let mut message = HashMap::new();

    message.insert(
        "text".to_string(),
        json!("You have been assigned to a card"),
    );

    let blocks = vec![
        json!({
            "type": "section",
            "text": {
                "type": "mrkdwn",
                "text": format!("*Card Assigned*\n<@{}> You have been assigned to: *{}*",
                    notification.slack_user_id, notification.card_title)
            }
        }),
        json!({
            "type": "section",
            "fields": [
                {
                    "type": "mrkdwn",
                    "text": format!("*Description:*\n{}",
                        notification.card_description.as_deref().unwrap_or("No description"))
                },
                {
                    "type": "mrkdwn",
                    "text": format!("*Priority:*\n{}",
                        notification.priority.as_deref().unwrap_or("Not set"))
                }
            ]
        }),
    ];

    message.insert("blocks".to_string(), json!(blocks));

    message
}

pub async fn send_notification_async(webhook_url: String, notification: SlackNotification) {
    tokio::spawn(async move {
        if let Err(e) = send_assignee_notification(&webhook_url, notification).await {
            eprintln!("Failed to send Slack notification: {}", e);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_message_has_blocks() {
        let payload = SlackNotificationPayload {
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
