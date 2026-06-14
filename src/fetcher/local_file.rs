use crate::core::{Document, Result};
use crate::fetcher::Fetcher;
use async_trait::async_trait;
use bytes::Bytes;
use uuid::Uuid;

pub struct LocalFileFetcher;

#[async_trait]
impl Fetcher for LocalFileFetcher {
    async fn fetch(&self, doc_ref: &str) -> Result<Document> {
        let data = tokio::fs::read(doc_ref).await?;

        let content = Bytes::from(data);

        let content_type = if doc_ref.ends_with(".pdf") {
            "application/pdf"
        } else {
            "text/plain"
        };

        Ok(Document {
            id: Uuid::new_v4(),
            source: doc_ref.to_string(),
            content,
            content_type: content_type.to_string(),
        })
    }
}
