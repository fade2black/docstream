use bytes::Bytes;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a job to process a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentJob {
    pub doc_id: Uuid,
    pub doc_ref: String,
}

/// Represents a fetched document (raw input to pipeline)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: Uuid,
    pub source: String,
    pub content: Bytes,
    pub content_type: String,
}

/// Semantic chunk produced by chunker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    pub id: Uuid,
    pub doc_id: Uuid,
    pub text: String,
}
