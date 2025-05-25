use crate::models::cards::{Card, Column};
use uuid::Uuid;

pub async fn get_board() -> Result<Vec<Column>, String> {
    let board = vec![
        Column {
            id: Uuid::new_v4(),
            title: "To Do".to_string(),
            cards: vec![
                Card {
                    id: Uuid::new_v4(),
                    title: "Learn Rust".to_string(),
                    description: Some("Complete Rust book chapters 1-3".to_string()),
                    assignee: Some("Alice".to_string()),
                    story_point: Some(3),
                    priority: Some("High".to_string()),
                },
                Card {
                    id: Uuid::new_v4(),
                    title: "Build a Kanban app".to_string(),
                    description: Some(
                        "Prototype Kanban layout and basic functionality".to_string(),
                    ),
                    assignee: Some("Bob".to_string()),
                    story_point: Some(5),
                    priority: Some("Medium".to_string()),
                },
            ],
        },
        Column {
            id: Uuid::new_v4(),
            title: "In Progress".to_string(),
            cards: vec![],
        },
        Column {
            id: Uuid::new_v4(),
            title: "Done".to_string(),
            cards: vec![],
        },
    ];

    Ok(board)
}
