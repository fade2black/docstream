use crate::core::Chunk;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub mod qdrant;

#[derive(Debug, serde::Deserialize)]
pub struct QdrantConfig {
    pub url: String,
    pub vector_size: u32,
    pub collection_name: String,
    pub api_key: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChunkMetadata {
    pub text: String,
    pub document_id: String,
}

pub struct SearchResult {
    /// The chunk ID of the matched embedding
    pub chunk_id: uuid::Uuid,

    /// Similarity score (higher is better)
    pub score: f32,

    /// Metadata attached to this chunk, if any
    pub metadata: ChunkMetadata,
}

#[async_trait]
pub trait VectorStore: Send + Sync {
    /// Store an embedding. The store decides which chunk fields to persist as metadata.
    async fn insert(&self, vec: &[f32], chunk: &Chunk) -> Result<(), VectorStoreError>;
    /// Search for the closest embeddings to the given query vector.
    async fn search(
        &self,
        query: &[f32],
        top_k: usize,
    ) -> Result<Vec<SearchResult>, VectorStoreError>;
}

#[derive(Debug, thiserror::Error)]
pub enum VectorStoreError {
    #[error("failed to store embedding: {0}")]
    StoreError(String),

    #[error("failed to search: {0}")]
    SearchError(String),

    #[error("store not available: {0}")]
    Unavailable(String),

    #[error("invalid id: {0}")]
    InvalidId(String),

    #[error("Missing chunk id")]
    MissingId,
}
