use crate::core::{Document, Result};
use crate::fetcher::Fetcher;

use async_trait::async_trait;
use aws_sdk_s3::Client;
use bytes::Bytes;
use uuid::Uuid;

pub struct S3Fetcher {
    client: Client,
}

impl S3Fetcher {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl Fetcher for S3Fetcher {
    async fn fetch(&self, doc_ref: &str) -> Result<Document> {
        let (bucket, key) = doc_ref
            .split_once(':')
            .ok_or_else(|| anyhow::anyhow!("Invalid S3 doc_ref format"))?;

        let response = self
            .client
            .get_object()
            .bucket(bucket)
            .key(key)
            .send()
            .await?;

        let content_type = response
            .content_type()
            .unwrap_or("application/octet-stream")
            .to_string();

        let bytes = response.body.collect().await?;

        let content = Bytes::from(bytes.into_bytes());

        Ok(Document {
            id: Uuid::new_v4(),
            source: format!("s3://{}/{}", bucket, key),
            content,
            content_type,
        })
    }
}
