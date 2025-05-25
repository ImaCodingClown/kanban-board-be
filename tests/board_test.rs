use kanban_backend::services::board::get_board;

#[tokio::test]
async fn test_get_board() {
    let result = get_board().await;

    assert!(result.is_ok(), "get_board failed: {:?}", result);

    let columns = result.unwrap();
    assert!(!columns.is_empty(), "Board should not be empty");

    let first_column = &columns[0];
    assert_eq!(
        first_column.title, "To Do",
        "First column title should be 'To Do'"
    );
    assert!(
        !first_column.cards.is_empty(),
        "First column should have cards"
    );
    let second_card = &first_column.cards[1];
    assert_eq!(
        second_card.story_point,
        Some(5),
        "Second card should have story point 5"
    );

    let second_column = &columns[1];
    assert_eq!(
        second_column.title, "In Progress",
        "Second column title should be 'In Progress'"
    );
    let third_column = &columns[2];
    assert_eq!(
        third_column.title, "Done",
        "Third column title should be 'Done'"
    );
}
