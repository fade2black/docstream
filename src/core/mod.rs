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

#[derive(Debug, Serialize, Deserialize)]
pub struct Embedding {
    /// The chunk this embedding was generated from
    pub chunk_id: Uuid,

    /// The vector representation (dimension depends on the model)
    pub vector: Vec<f32>,

    /// The model that produced this embedding, e.g. "text-embedding-3-small"
    pub model_name: String,
}

/// Semantic chunk produced by chunker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    pub id: Uuid,
    pub doc_id: Uuid,
    pub text: String,
}
