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
        payload: &SlackNotificationPayload,
    ) -> Result<(), &CustomError>;
}

pub struct SlackWebhookNotifier;

#[async_trait::async_trait]
impl SlackNotifier for SlackWebhookNotifier {
    fn build_message(
        &self,
        payload: &SlackNotificationPayload,
    ) -> HashMap<String, serde_json::Value> {
        create_slack_message(payload.clone())
    }

    async fn send(
        &self,
        webhook_url: &str,
        payload: &SlackNotificationPayload,
    ) -> Result<(), &CustomError> {
        let message = self.build_message(payload);
        send_notification_with_message(webhook_url, message)
            .await
            .map_err(|e| CustomError::Server(e.to_string()))?;
        Ok(())
    }
}

pub async fn send_notification_with_message(
    webhook_url: &str,
    message: HashMap<String, serde_json::Value>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::new();

    let response = client.post(webhook_url).json(&message).send().await?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("Slack webhook failed: {} - {}", status, body).into());
    }

    Ok(())
}

pub fn create_slack_message(
    notification: SlackNotificationPayload,
) -> HashMap<String, serde_json::Value> {
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
                "text": format!("<@{}> has been assigned to a card!", notification.slack_user_id)
            }
        }),
        json!({
            "type": "divider"
        }),
        json!({
            "type": "section",
            "text": {
                "type": "mrkdwn",
                "text": format!("*{}*", notification.card_title)
            }
        }),
        json!({
            "type": "section",
            "fields": [
                {
                    "type": "mrkdwn",
                    "text": format!("*Description:*\n{}",
                        notification.card_description.unwrap_or("No description"))
                },
                {
                    "type": "mrkdwn",
                    "text": format!("*Priority:*\n{}",
                        notification.priority.unwrap_or("Not set"))
                }
            ]
        }),
    ];

    message.insert("blocks".to_string(), json!(blocks));

    message
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_message_has_blocks() {
        let payload = SlackNotificationPayload {
            slack_user_id: "U01ABC2DEF3".to_string(),
            card_title: "Test Card",
            card_description: Some("Test description"),
            priority: Some("High"),
        };

        let message = create_slack_message(payload);

        assert!(message.contains_key("text"));
        assert!(message.contains_key("blocks"));

        let blocks = message.get("blocks").unwrap().as_array().unwrap();
        assert_eq!(blocks.len(), 4);
    }
}
