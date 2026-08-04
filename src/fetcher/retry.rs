use crate::core::DocumentJob;
use crate::fetcher::{FetchedData, Fetcher, FetcherError};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::time::{Duration, sleep};
use tracing::{error, info};

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

    fn backoff_delay_ms(base_delay_ms: u64, attempt: u32) -> u64 {
        base_delay_ms * 2u64.pow(attempt)
    }
}

#[async_trait]
impl<F> Fetcher for RetryFetcher<F>
where
    F: Fetcher + Send + Sync,
{
    async fn fetch(&self, job: &DocumentJob) -> Result<FetchedData, FetcherError> {
        let mut attempt = 0;

        loop {
            match self.fetcher.fetch(job).await {
                Ok(doc) => return Ok(doc),
                Err(err) => {
                    if attempt >= self.max_retries {
                        error!(
                            "Fetch failed: doc_id={} after {} attempts",
                            job.doc_id, attempt
                        );
                        return Err(err);
                    }

                    let delay = Self::backoff_delay_ms(self.base_delay_ms, attempt as u32);
                    sleep(Duration::from_millis(delay)).await;

                    attempt += 1;
                    info!("Retrying fetch: attempt={}, delay={}ms", attempt, delay);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backoff_delay_grows_exponentially() {
        let base_delay_ms = 100;

        assert_eq!(RetryFetcher::<()>::backoff_delay_ms(base_delay_ms, 0), 100);
        assert_eq!(RetryFetcher::<()>::backoff_delay_ms(base_delay_ms, 1), 200);
        assert_eq!(RetryFetcher::<()>::backoff_delay_ms(base_delay_ms, 2), 400);
        assert_eq!(RetryFetcher::<()>::backoff_delay_ms(base_delay_ms, 3), 800);
    }
}
