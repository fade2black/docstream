use crate::core::DocumentJob;
use async_trait::async_trait;
use bytes::Bytes;

pub mod local_file;
pub mod retry;
pub mod s3;

pub struct FetchedData {
    pub source: String,
    pub content: Bytes,
    pub content_type: String,
}

#[async_trait]
pub trait Fetcher: Send + Sync {
    async fn fetch(&self, doc: &DocumentJob) -> Result<FetchedData, FetcherError>;
}

#[derive(Debug, thiserror::Error)]
pub enum FetcherError {
    #[error("Invalid S3 doc_ref format")]
    InvalidS3DocRefFormat,

    #[error("failed to read document: {0}")]
    ReadError(String),
}
