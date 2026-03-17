use crate::{
    config::AppState,
    db::mongo::{MongoService, ODM},
    models::cards::{AddCardPayload, Board, Card, DeleteCardPayload, EditCardPayload},
    models::slack::SlackNotificationPayload,
    models::teams::Team,
    services::{
        board::get_board_by_id,
        slack::{SlackNotifier, SlackWebhookNotifier},
        user_info::get_user_by_username,
    },
    utils::errors::CustomError,
};
use mongodb::bson::{doc, oid::ObjectId};
use mongodb::Client;

pub async fn add_card(payload: AddCardPayload, app_state: &AppState) -> Result<Card, CustomError> {
    let mut board = get_board_by_id(payload.board_id.clone(), &app_state.db).await?;

    let teams = app_state.db.database("general").collection::<Team>("teams");

    let team = teams
        .find_one(doc! {"name": &board.team})
        .await
        .map_err(CustomError::MongoError)?
        .ok_or_else(|| CustomError::NotFound("Team not found".to_string()))?;

    let card_prefix = if let Some(prefix) = team.card_prefix {
        prefix
    } else {
        // generate prefix if existing team doesn't have one
        let prefix = board
            .team
            .chars()
            .filter(|c| c.is_alphanumeric())
            .take(4)
            .collect::<String>()
            .to_uppercase();

        teams
            .update_one(
                doc! {"name": &board.team},
                doc! {"$set": {"card_prefix": &prefix, "next_card_number": 1}},
            )
            .await
            .map_err(CustomError::MongoError)?;

        prefix
    };

    let updated_team = teams
        .find_one_and_update(
            doc! {"name": &board.team},
            doc! {"$inc": {"next_card_number": 1}},
        )
        .await
        .map_err(CustomError::MongoError)?
        .ok_or_else(|| CustomError::NotFound("Team not found".to_string()))?;

    let card_number = updated_team.next_card_number.unwrap_or(1);
    let card_id = format!("{}-{}", card_prefix, card_number);

    let col = board
        .columns
        .iter_mut()
        .find(|c| c.title == payload.column_name)
        .ok_or_else(|| CustomError::NotFound("Column not found".to_string()))?;

    let card = Card {
        id: Some(ObjectId::new()),
        card_id: Some(card_id),
        card_number: Some(card_number),
        board_id: payload.board_id.clone(),
        title: payload.title.clone(),
        description: payload.description.clone(),
        assignee: payload.assignee.clone(),
        story_point: payload.story_point,
        priority: payload.priority.clone(),
    };

    col.cards.push(card.clone());

    let board_service = ODM::<Board>::build(&app_state.db).await;
    let board_id = board
        .id
        .as_ref()
        .ok_or_else(|| CustomError::NotFound("Missing board ID".to_string()))?;
    board_service.replace_one(&board, board_id).await?;

    // Send Slack notification if team has webhook and assignee is set
    if let Some(assignee) = &payload.assignee {
        let teams = app_state.db.database("general").collection::<Team>("teams");
        if let Ok(Some(team)) = teams
            .find_one(mongodb::bson::doc! {"name": &board.team })
            .await
        {
            if let Some(webhook_url) = team.slack_webhook_url {
                if let Ok(user) = get_user_by_username(assignee, &app_state.db).await {
                    if let Some(slack_user_id) = user.slack_user_id {
                        let payload = SlackNotificationPayload {
                            slack_user_id,
                            card_title: &payload.title,
                            card_description: payload.description.as_deref(),
                            priority: payload.priority.as_deref(),
                        };
                        let notifier = SlackWebhookNotifier;
                        let _ = notifier.send(&webhook_url, &payload).await;
                    }
                }
            }
        }
    }

    Ok(card)
}

pub async fn get_columns(board_id: &str, db: &Client) -> Result<Vec<String>, CustomError> {
    let board = get_board_by_id(board_id.to_string(), db).await?;
    let column_titles = board.columns.iter().map(|c| c.title.clone()).collect();
    Ok(column_titles)
}

pub async fn delete_card(payload: DeleteCardPayload, db: &Client) -> Result<(), CustomError> {
    let mut board = get_board_by_id(payload.board_id.clone(), db).await?;

    let card_oid = ObjectId::parse_str(&payload.card_id)
        .map_err(|_| CustomError::NotFound("Invalid card ID".to_string()))?;

    let col = board
        .columns
        .iter_mut()
        .find(|c| c.title == payload.column_name)
        .ok_or_else(|| CustomError::NotFound("Column not found".to_string()))?;

    col.cards.retain(|card| card.id != Some(card_oid));

    let board_service = ODM::<Board>::build(db).await;
    let board_id = board
        .id
        .as_ref()
        .ok_or_else(|| CustomError::NotFound("Missing board ID".to_string()))?;
    board_service.replace_one(&board, board_id).await?;

    Ok(())
}

pub async fn edit_card(
    payload: EditCardPayload,
    app_state: &AppState,
) -> Result<Card, CustomError> {
    let mut board = get_board_by_id(payload.board_id.clone(), &app_state.db).await?;

    let card_oid = ObjectId::parse_str(&payload.card_id)?;

    let mut card = board
        .columns
        .iter_mut()
        .find(|col| col.title == payload.column_name)
        .and_then(|col| {
            Some(
                col.cards
                    .remove(col.cards.iter().position(|c| c.id == Some(card_oid))?),
            )
        })
        .ok_or_else(|| CustomError::NotFound("Card not found".to_string()))?;

    let old_assignee = card.assignee;

    card.title = payload.title;
    card.description = payload.description;
    card.assignee = payload.assignee;
    card.story_point = payload.story_point;
    card.priority = payload.priority;

    let column_name = if let Some(new_column_name) = &payload.new_column_name {
        new_column_name
    } else {
        &payload.column_name
    };

    if let Some(col) = board
        .columns
        .iter_mut()
        .find(|col| &col.title == column_name)
    {
        col.cards.push(card)
    }

    let board_service = ODM::<Board>::build(&app_state.db).await;
    let board_id = board
        .id
        .as_ref()
        .ok_or_else(|| CustomError::NotFound("Missing board ID".to_string()))?;
    board_service.replace_one(&board, board_id).await?;

    let edited_card = board
        .columns
        .into_iter()
        .find(|col| &col.title == column_name)
        .and_then(|col| col.cards.into_iter().find(|c| c.id == Some(card_oid)))
        .ok_or_else(|| CustomError::NotFound("Updated card not found".to_string()))?;

    if old_assignee != edited_card.assignee {
        if let Some(assignee) = &edited_card.assignee {
            let teams = app_state.db.database("general").collection::<Team>("teams");
            if let Ok(Some(team)) = teams
                .find_one(mongodb::bson::doc! {"name": &board.team })
                .await
            {
                if let Some(webhook_url) = team.slack_webhook_url {
                    if let Ok(user) = get_user_by_username(assignee, &app_state.db).await {
                        if let Some(slack_user_id) = user.slack_user_id {
                            let payload = SlackNotificationPayload {
                                slack_user_id,
                                card_title: &edited_card.title,
                                card_description: edited_card.description.as_deref(),
                                priority: edited_card.priority.as_deref(),
                            };
                            let notifier = SlackWebhookNotifier;
                            let _ = notifier.send(&webhook_url, &payload).await;
                        }
                    }
                }
            }
        }
    }

    Ok(edited_card)
}
