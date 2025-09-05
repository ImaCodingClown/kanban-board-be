/// Creates a MongoDB index with error handling.
///
/// # Arguments
/// - `$collection`: The MongoDB collection to create the index on
/// - `$keys`: The index keys as a BSON document (e.g., `doc! { "field": 1 }`)
/// - `$error_msg`: Error message format string for database errors
///
/// # Example
/// ```ignore
/// create_index!(users, doc! { "username": 1 }, "Failed to create username index: {}");
/// ```
#[macro_export]
macro_rules! create_index {
    ($collection:expr, $keys:expr, $error_msg:literal) => {
        $collection
            .create_index(mongodb::IndexModel::builder().keys($keys).build())
            .await
            .map_err(|e| crate::utils::errors::CustomError::Database(format!($error_msg, e)))?;
    };
}
