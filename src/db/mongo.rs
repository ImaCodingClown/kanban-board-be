use mongodb::Client;
use std::sync::Arc;

pub async fn db_client(uri: &str) -> Arc<Client> {
    let client = Client::with_uri_str(uri)
        .await
        .expect("Failed to connect to MongoDB");
    Arc::new(client)
}

