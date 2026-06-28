use crate::core::Chunk;
use async_trait::async_trait;
use tokio::sync::mpsc;
use uuid::Uuid;

pub mod simple;

#[async_trait]
pub trait Chunker: Send + Sync {
    async fn chunk(
        &self,
        content: &str,
        source_id: Uuid,
        tx: mpsc::Sender<Chunk>,
    ) -> Result<(), ChunkerError>;
}

#[derive(Debug, thiserror::Error)]
pub enum ChunkerError {
    #[error("content is empty")]
    EmptyContent,

    #[error("chunking failed: {0}")]
    ChunkingFailed(String),
}
