use crate::core::Document;
use crate::core::Result;
use crate::fetcher::Fetcher;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::time::{Duration, sleep};
use tracing::info;

pub struct RetryFetcher<F> {
    fetcher: Arc<F>,
    max_retries: usize,
    base_delay_ms: u64,
}

impl<F> RetryFetcher<F> {
    pub fn new(fetcher: Arc<F>, max_retries: usize, base_delay_ms: u64) -> Self {
        Self {
            fetcher,
            max_retries,
            base_delay_ms,
        }
    }
}

#[async_trait]
impl<F> Fetcher for RetryFetcher<F>
where
    F: Fetcher + Send + Sync,
{
    async fn fetch(&self, doc_ref: &str) -> Result<Document> {
        let mut attempt = 0;

        loop {
            match self.fetcher.fetch(doc_ref).await {
                Ok(doc) => return Ok(doc),
                Err(err) => {
                    if attempt >= self.max_retries {
                        return Err(err);
                    }

                    let delay = self.base_delay_ms * 2u64.pow(attempt as u32);
                    sleep(Duration::from_millis(delay)).await;

                    attempt += 1;
                    info!("Retrying fetch: attempt={}, delay={}ms", attempt, delay);
                }
            }
        }
    }
}
