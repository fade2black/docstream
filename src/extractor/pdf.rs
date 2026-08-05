use crate::core::Document;
use crate::extractor::{Extractor, ExtractorError};
use pdfium_render::prelude::*;
use std::sync::Arc;

use async_trait::async_trait;

pub struct PdfExtractor {
    pdfium: Arc<Pdfium>,
}

impl PdfExtractor {
    pub fn new() -> Result<Self, ExtractorError> {
        let pdfium = Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path(
            "./vendor/pdfium",
        ))?;

        let pdfium = Arc::new(Pdfium::new(pdfium));

        Ok(Self { pdfium })
    }
}

#[async_trait]
impl Extractor for PdfExtractor {
    async fn extract(&self, document: &Document) -> Result<String, ExtractorError> {
        let pdfium = self.pdfium.clone();
        let content = document.content.clone();

        tokio::task::spawn_blocking(move || {
            let pdf = pdfium.load_pdf_from_byte_slice(content.as_ref(), None)?;
            let mut text = String::new();

            for page in pdf.pages().iter() {
                text.push_str(&page.text()?.all());
                text.push_str("\n\n");
            }

            Ok::<_, ExtractorError>(text)
        })
        .await
        .map_err(|e| ExtractorError::JoinError(e.to_string()))?
    }
}
