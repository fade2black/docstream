use crate::core::{Document, Result};
use async_trait::async_trait;

pub mod local_file;
pub mod retry;
pub mod s3;

#[async_trait]
pub trait Fetcher: Send + Sync {
    async fn fetch(&self, doc_ref: &str) -> Result<Document>;
}
