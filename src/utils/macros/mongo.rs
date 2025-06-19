/// Implements `MongoModel` and `MongoService` for a given model type.
///
/// # Arguments
/// - `$model`: The struct type to implement the traits for (e.g., `User`)
/// - `$collection`: The MongoDB collection name as a string literal
/// - `$database`: The MongoDB database name as a string literal
///
/// # Example
/// ```ignore
/// impl_mongo!(
///     Card,
///     "users",
///     "general"
/// );
/// ```
#[macro_export]
macro_rules! impl_mongo {
    (
     $model:ty,
     $collection:expr,
     $database:expr) => {
        #[::async_trait::async_trait]
        impl MongoService<$model> for ODM<$model> {
            const COLLECTION: &'static str = $collection;
            const DATABASE: &'static str = $database;

            async fn build(client: &::mongodb::Client) -> Self {
                let collection: ::mongodb::Collection<$model> =
                    client.database(Self::DATABASE).collection(Self::COLLECTION);
                ODM::<$model> {
                    client: std::sync::Arc::new(client.clone()),
                    collection,
                }
            }
        }
    };
}
