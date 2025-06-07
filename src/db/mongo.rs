use async_trait::async_trait;
use mongodb::{bson::Document, results::InsertOneResult, Client, Collection};
use serde::{de::DeserializeOwned, Serialize};
use std::sync::Arc;

use crate::utils::errors::CustomError;

pub async fn db_client(uri: &str) -> Arc<Client> {
    let client = Client::with_uri_str(uri)
        .await
        .expect("Failed to connect to MongoDB");
    Arc::new(client)
}

pub trait MongoModel: Send + Sync + DeserializeOwned + Serialize {
    fn unique_query(&self) -> Document;
}

#[async_trait]
pub trait MongoService<T>
where
    T: MongoModel,
{
    const COLLECTION: &str;
    const DATABASE: &str;

    async fn build(client: &Client) -> Self;
}

#[allow(clippy::upper_case_acronyms)]
pub struct ODM<T>
where
    T: MongoModel,
{
    #[allow(dead_code)]
    pub client: Arc<Client>,
    pub collection: Collection<T>,
}

impl<T> ODM<T>
where
    T: MongoModel,
{
    pub async fn fetch_one(&self, model: &T) -> Result<Option<T>, CustomError> {
        self.collection
            .find_one(model.unique_query())
            .await
            .map_err(CustomError::MongoError)
    }
    pub async fn save_one(&self, model: &T) -> Result<InsertOneResult, CustomError> {
        self.collection
            .insert_one(model)
            .await
            .map_err(CustomError::MongoError)
    }
}
