use bytes::Bytes;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type Result<T> = anyhow::Result<T>;

/// Represents a fetched document (raw input to pipeline)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: Uuid,
    pub source: String,
    pub content: Bytes,
    pub content_type: String,
}

///// Semantic chunk produced by chunker
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct Chunk {
//     pub id: Uuid,
//     pub document_id: Uuid,
//     pub source: String,
//     pub text: String,
//     pub position: usize,
// }
