use async_trait::async_trait;
use futures::TryStreamExt;
use mongodb::{
    bson::{self, doc, oid::ObjectId, Document},
    results::InsertOneResult,
    Client, Collection, Cursor,
};
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
    fn query(&self) -> Result<Document, CustomError> {
        bson::to_document(&self).map_err(|err| CustomError::CustomError(err.to_string()))
    }
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
    pub async fn fetch_many(&self, model: &T) -> Result<Vec<T>, CustomError> {
        self.collection
            .find(model.query()?)
            .await
            .map_err(CustomError::MongoError)?
            .try_collect()
            .await
            .map_err(CustomError::MongoError)
    }
    pub async fn fetch_by_id(&self, id: ObjectId) -> Result<Option<T>, CustomError> {
        self.collection
            .find_one(doc! { "_id": id })
            .await
            .map_err(CustomError::MongoError)
    }

    pub async fn fetch_many_by_ids(&self, ids: Vec<ObjectId>) -> Result<Cursor<T>, CustomError> {
        self.collection
            .find(doc! { "_id": {"$in": ids} })
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
