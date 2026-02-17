use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct SlackNotificationPayload<'a> {
    pub slack_user_id: String,
    pub card_title: &'a str,
    pub card_description: Option<&'a str>,
    pub priority: Option<&'a str>,
}
