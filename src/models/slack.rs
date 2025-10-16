use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct SlackNotificationPayload {
    pub slack_user_id: String,
    pub card_title: String,
    pub card_description: Option<String>,
    pub priority: Option<String>,
}
