use crate::core::Chunk;
use async_trait::async_trait;

pub mod qdrant;

#[derive(Debug, serde::Deserialize)]
pub struct QdrantConfig {
    pub url: String,
    pub vector_size: u32,
    pub collection_name: String,
    pub api_key: Option<String>,
}

// #[async_trait]
// pub trait Fetcher: Send + Sync {
//     async fn fetch(&self, doc: &DocumentJob) -> Result<FetchedData, FetcherError>;
// }

#[async_trait]
pub trait VectorStore: Send + Sync {
    /// Store an embedding. The store decides which chunk fields to persist as metadata.
    async fn insert(&self, vec: &[f32], chunk: &Chunk) -> Result<(), VectorStoreError>;
}

#[derive(Debug, thiserror::Error)]
pub enum VectorStoreError {
    #[error("failed to store embedding: {0}")]
    StoreError(String),

    #[error("store not available: {0}")]
    Unavailable(String),
}
