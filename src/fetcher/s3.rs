use crate::core::DocumentJob;
use crate::fetcher::{FetchedData, Fetcher, FetcherError};

use async_trait::async_trait;
use aws_sdk_s3::Client;

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
    async fn fetch(&self, job: &DocumentJob) -> Result<FetchedData, FetcherError> {
        let (bucket, key) = job
            .doc_ref
            .split_once(':')
            .ok_or(FetcherError::InvalidS3DocRefFormat)?;

        let response = self
            .client
            .get_object()
            .bucket(bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| FetcherError::ReadError(e.to_string()))?;

        let content_type = response
            .content_type()
            .unwrap_or("application/octet-stream")
            .to_string();

        let bytes = response
            .body
            .collect()
            .await
            .map_err(|e| FetcherError::ReadError(e.to_string()))?;

        let content = bytes.into_bytes();

        Ok(FetchedData {
            source: format!("s3://{}/{}", bucket, key),
            content,
            content_type,
        })
    }
}
