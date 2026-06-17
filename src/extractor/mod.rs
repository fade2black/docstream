pub mod pdf;
pub mod text;

use crate::core::Document;
use async_trait::async_trait;
use pdfium_render::prelude::PdfiumError;

#[async_trait]
pub trait Extractor: Send + Sync {
    //Send + Sync: allow extractors to live inside workers and be shared safely.
    async fn extract(&self, document: &Document) -> Result<String, ExtractorError>;
}

#[derive(Debug, thiserror::Error)]
pub enum ExtractorError {
    #[error("pdfium bind error")]
    PdfiumBindError(#[from] PdfiumError),
    #[error("utf-8 decode error")]
    Utf8Error(#[from] std::str::Utf8Error),
}
