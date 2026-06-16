pub mod pdf;
pub mod text;

use crate::core::{Document, Result};
use async_trait::async_trait;

#[async_trait]
pub trait Extractor: Send + Sync {
    //Send + Sync: allow extractors to live inside workers and be shared safely.
    async fn extract(&self, document: &Document) -> Result<String>;
}
