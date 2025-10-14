use crate::{
    config::AppState,
    db::mongo::{MongoService, ODM},
    models::cards::{AddCardPayload, Board, Card, DeleteCardPayload, EditCardPayload},
    services::{slack::SlackNotification, user_info::get_user_by_username},
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

    // Send Slack notification if assignee is set
    if let Some(webhook_url) = &app_state.slack_webhook_url {
        if let Some(assignee) = &payload.assignee {
            if let Ok(user) = get_user_by_username(assignee, &app_state.db).await {
                if let Some(slack_user_id) = user.slack_user_id {
                    let notification = SlackNotification {
                        slack_user_id,
                        card_title: payload.title,
                        card_description: payload.description,
                        priority: payload.priority,
                    };

                    if let Err(e) = crate::services::slack::send_assignee_notification(
                        webhook_url,
                        notification,
                    )
                    .await
                    {
                        eprintln!("Failed to send Slack notification: {}", e);
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

    // Send Slack notification if assignee changed
    if let Some(webhook_url) = &app_state.slack_webhook_url {
        if old_assignee != Some(payload.assignee.clone()) {
            if let Ok(user) = get_user_by_username(&payload.assignee, &app_state.db).await {
                if let Some(slack_user_id) = user.slack_user_id {
                    let notification = SlackNotification {
                        slack_user_id,
                        card_title: payload.title.clone(),
                        card_description: Some(payload.description.clone()),
                        priority: payload.priority.clone(),
                    };

                    if let Err(e) = crate::services::slack::send_assignee_notification(
                        webhook_url,
                        notification,
                    )
                    .await
                    {
                        eprintln!("Failed to send Slack notification: {}", e);
                    }
                }
            }
        }
    }

    Ok(edited_card.clone())
}
