use crate::core::Document;
use async_trait::async_trait;

pub mod local_file;
pub mod retry;
pub mod s3;

#[async_trait]
pub trait Fetcher: Send + Sync {
    async fn fetch(&self, doc_ref: &str) -> Result<Document, FetcherError>;
}

#[derive(Debug, thiserror::Error)]
pub enum FetcherError {
    #[error("Invalid S3 doc_ref format")]
    InvalidS3DocRefFormat,

    #[error("failed to read document: {0}")]
    ReadError(String),
}
