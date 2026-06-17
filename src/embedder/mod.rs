use crate::core::Embedding;

pub mod fastembed;

#[async_trait::async_trait]
pub trait Embedder: Send + Sync {
    /// Generate an embedding vector for the given text.
    /// `chunk_id` is the ID of the chunk being embedded.
    ///
    /// # Contract
    /// Implementations **must** return a normalized (unit length) vector.
    /// Normalization is the embedder's responsibility — the pipeline and
    /// vector store rely on dot product as cosine similarity, which is only
    /// correct when vectors have magnitude 1.0.
    async fn embed(&mut self, chunk_id: uuid::Uuid, text: &str)
    -> Result<Embedding, EmbedderError>;
}

#[derive(Debug, thiserror::Error)]
pub enum EmbedderError {
    #[error("embedding API error: {0}")]
    ApiError(String),

    #[error("empty text provided")]
    EmptyText,

    #[error("model not available: {0}")]
    ModelUnavailable(String),
}
