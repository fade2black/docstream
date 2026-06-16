use crate::core::{Document, Result};
use crate::extractor::Extractor;

use anyhow::anyhow;
use async_trait::async_trait;

pub struct TextExtractor;

impl TextExtractor {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Extractor for TextExtractor {
    async fn extract(&self, document: &Document) -> Result<String> {
        // Take raw bytes and decode as UTF-8 text.
        let text = std::str::from_utf8(document.content.as_ref())
            .map_err(|e| anyhow!("Failed to decode UTF-8 text: {}", e))?;

        Ok(text.to_owned())
    }
}
