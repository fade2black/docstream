use crate::core::DocumentJob;
use crate::fetcher::{FetchedData, Fetcher, FetcherError};
use async_trait::async_trait;
use bytes::Bytes;

pub struct LocalFileFetcher;

#[async_trait]
impl Fetcher for LocalFileFetcher {
    async fn fetch(&self, job: &DocumentJob) -> Result<FetchedData, FetcherError> {
        let data = tokio::fs::read(&job.doc_ref)
            .await
            .map_err(|e| FetcherError::ReadError(e.to_string()))?;

        let content = Bytes::from(data);

        let content_type = if job.doc_ref.ends_with(".pdf") {
            "application/pdf"
        } else {
            "text/plain"
        };

        Ok(FetchedData {
            source: job.doc_ref.clone(),
            content,
            content_type: content_type.to_string(),
        })
    }
}
