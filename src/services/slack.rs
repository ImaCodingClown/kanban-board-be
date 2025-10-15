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

pub struct SlackWebhookNotifier;

impl SlackWebhookNotifier {
    fn create_message(payload: &SlackNotificationPayload) -> HashMap<String, serde_json::Value> {
        let mut message = HashMap::new();

        message.insert(
            "text".to_string(),
            json!(format!(
                "Hey <@{}>, you've been assigned to a card!",
                payload.slack_user_id
            )),
        );

        let mut blocks = vec![
            json!({
                "type": "section",
                "text": {
                    "type": "mrkdwn",
                    "text": format!("*<@{}> has been assigned to a card!*", payload.slack_user_id)
                }
            }),
            json!({ "type": "divider" }),
            json!({
                "type": "section",
                "fields": [
                    { "type": "mrkdwn", "text": format!("*Card:*\n{}", payload.card_title) },
                    { "type": "mrkdwn", "text": format!("*Priority:*\n{}", payload.priority.as_deref().unwrap_or("N/A")) }
                ]
            }),
        ];

        if let Some(description) = &payload.card_description {
            blocks.push(json!({
                "type": "section",
                "text": { "type": "mrkdwn", "text": format!("*Description:*\n{}", description) }
            }));
        }

        message.insert("blocks".to_string(), json!(blocks));
        message
    }
}

#[async_trait::async_trait]
impl SlackNotifier for SlackWebhookNotifier {
    fn build_message(
        &self,
        payload: &SlackNotificationPayload,
    ) -> HashMap<String, serde_json::Value> {
        Self::create_message(payload)
    }

    async fn send(
        &self,
        webhook_url: &str,
        payload: SlackNotificationPayload,
    ) -> Result<(), CustomError> {
        let client = reqwest::Client::new();
        let message = Self::create_message(&payload);
        let response = client
            .post(webhook_url)
            .json(&message)
            .send()
            .await
            .map_err(|e| CustomError::Server(format!("Slack request error: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(CustomError::Server(format!(
                "Slack webhook failed: {}",
                status
            )));
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

        let msg = SlackWebhookNotifier::create_message(&payload);
        assert!(msg.contains_key("text"));
        assert!(msg.contains_key("blocks"));
    }
}
