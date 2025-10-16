use crate::{
    config::AppState,
    db::mongo::{MongoService, ODM},
    models::cards::{AddCardPayload, Board, Card, DeleteCardPayload, EditCardPayload},
    models::slack::SlackNotificationPayload,
    models::teams::Team,
    services::{
        slack::{SlackNotifier, SlackWebhookNotifier},
        user_info::get_user_by_username,
    },
    utils::errors::CustomError,
};
use mongodb::{bson::oid::ObjectId, Client};

pub async fn add_card(payload: AddCardPayload, app_state: &AppState) -> Result<Card, CustomError> {
    let board_service = ODM::<Board>::build(&app_state.db).await;
    let mut boards = board_service.fetch_many_by_team(&payload.team).await?;

    let board = boards
        .get_mut(0)
        .ok_or_else(|| CustomError::NotFound("Board not found".to_string()))?;

    let col = board
        .columns
        .iter_mut()
        .find(|c| c.title == payload.column_name)
        .ok_or_else(|| CustomError::NotFound("Column not found".to_string()))?;

    let card = Card {
        id: Some(ObjectId::new()),
        title: payload.title.clone(),
        description: payload.description.clone(),
        assignee: payload.assignee.clone(),
        story_point: payload.story_point,
        priority: payload.priority.clone(),
    };

    col.cards.push(card.clone());

    board_service
        .replace_one(board, board.id.as_ref().unwrap())
        .await?;

    // Send Slack notification if team has webhook and assignee is set
    if let Some(assignee) = &payload.assignee {
        let teams = app_state.db.database("general").collection::<Team>("teams");
        if let Ok(Some(team)) = teams
            .find_one(mongodb::bson::doc! {"name": &payload.team })
            .await
        {
            if let Some(webhook_url) = team.slack_webhook_url {
                if let Ok(user) = get_user_by_username(assignee, &app_state.db).await {
                    if let Some(slack_user_id) = user.slack_user_id {
                        let payload = SlackNotificationPayload {
                            slack_user_id,
                            card_title: payload.title,
                            card_description: payload.description,
                            priority: payload.priority,
                        };
                        let notifier = SlackWebhookNotifier;
                        let _ = notifier.send(&webhook_url, payload).await;
                    }
                }
            }
        }
    }

    Ok(card)
}

pub async fn get_columns(team: &str, db: &Client) -> Result<Vec<String>, CustomError> {
    let board_service = ODM::<Board>::build(db).await;
    let boards = board_service.fetch_many_by_team(team).await?;

    let board = boards
        .first()
        .ok_or_else(|| CustomError::NotFound("Board not found".to_string()))?;

    let column_titles = board.columns.iter().map(|c| c.title.clone()).collect();

    Ok(column_titles)
}

pub async fn delete_card(payload: DeleteCardPayload, db: &Client) -> Result<(), CustomError> {
    let board_service = ODM::<Board>::build(db).await;
    let mut boards = board_service.fetch_many_by_team(&payload.team).await?;

    let board = boards
        .get_mut(0)
        .ok_or_else(|| CustomError::NotFound("Board not found".to_string()))?;

    let card_oid = ObjectId::parse_str(&payload.card_id)
        .map_err(|_| CustomError::NotFound("Invalid card ID".to_string()))?;

    let col = board
        .columns
        .iter_mut()
        .find(|c| c.title == payload.column_name)
        .ok_or_else(|| CustomError::NotFound("Column not found".to_string()))?;

    col.cards.retain(|card| card.id != Some(card_oid));

    let board_id = board
        .id
        .as_ref()
        .ok_or_else(|| CustomError::NotFound("Missing board ID".to_string()))?;
    board_service.replace_one(board, board_id).await?;

    Ok(())
}

pub async fn edit_card(
    payload: EditCardPayload,
    app_state: &AppState,
) -> Result<Card, CustomError> {
    let board_service = ODM::<Board>::build(&app_state.db).await;
    let mut boards = board_service.fetch_many_by_team(&payload.team).await?;

    let board = boards
        .get_mut(0)
        .ok_or_else(|| CustomError::NotFound("Board not found".to_string()))?;

    let card_oid = ObjectId::parse_str(&payload.card_id)
        .map_err(|_| CustomError::NotFound("Invalid card ID".to_string()))?;

    let old_assignee = {
        let column = board
            .columns
            .iter()
            .find(|col| col.title == payload.column_name)
            .ok_or_else(|| CustomError::NotFound("Column not found".to_string()))?;

        let card = column
            .cards
            .iter()
            .find(|card| card.id == Some(card_oid))
            .ok_or_else(|| CustomError::NotFound("Card not found".to_string()))?;

        card.assignee.clone()
    };

    {
        let column = board
            .columns
            .iter_mut()
            .find(|col| col.title == payload.column_name)
            .ok_or_else(|| CustomError::NotFound("Column not found".to_string()))?;

        let card = column
            .cards
            .iter_mut()
            .find(|card| card.id == Some(card_oid))
            .ok_or_else(|| CustomError::NotFound("Card not found".to_string()))?;

        card.title = payload.title.clone();
        card.description = Some(payload.description.clone());
        card.assignee = Some(payload.assignee.clone());
        card.story_point = payload.story_point;
        card.priority = payload.priority.clone();
    }

    let board_id = board.id.unwrap();
    board_service.replace_one(board, &board_id).await?;

    let edited_card = board
        .columns
        .iter()
        .find(|col| col.title == payload.column_name)
        .and_then(|col| col.cards.iter().find(|c| c.id == Some(card_oid)))
        .ok_or_else(|| CustomError::NotFound("Updated card not found".to_string()))?;

    if old_assignee != Some(payload.assignee.clone()) {
        let teams = app_state.db.database("general").collection::<Team>("teams");
        if let Ok(Some(team)) = teams
            .find_one(mongodb::bson::doc! {"name": &payload.team })
            .await
        {
            if let Some(webhook_url) = team.slack_webhook_url {
                if let Ok(user) = get_user_by_username(&payload.assignee, &app_state.db).await {
                    if let Some(slack_user_id) = user.slack_user_id {
                        let payload = SlackNotificationPayload {
                            slack_user_id,
                            card_title: payload.title.clone(),
                            card_description: Some(payload.description.clone()),
                            priority: payload.priority.clone(),
                        };
                        let notifier = SlackWebhookNotifier;
                        let _ = notifier.send(&webhook_url, payload).await;
                    }
                }
            }
        }
    }

    Ok(edited_card.clone())
}
