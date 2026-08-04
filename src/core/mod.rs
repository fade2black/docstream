use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Represents a job to process a document
/// If, in the future, clients need idempotency (e.g., safely retrying the same request),
/// we could allow them to provide an ID or an idempotency key.
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

#[derive(Debug, Clone)]
pub struct Metadata {
    /// Arbitrary key-value pairs attached to a chunk
    pub fields: HashMap<String, serde_json::Value>,
}
