use crate::core::Document;
use crate::extractor::{Extractor, ExtractorError};
use pdfium_render::prelude::*;

use async_trait::async_trait;

pub struct PdfExtractor {
    pdfium: Pdfium,
}

impl PdfExtractor {
    pub fn new() -> Result<Self, ExtractorError> {
        let pdfium = Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path(
            "./vendor/pdfium",
        ))?;

        let pdfium = Pdfium::new(pdfium);

        Ok(Self { pdfium })
    }
}

#[async_trait]
impl Extractor for PdfExtractor {
    async fn extract(&self, document: &Document) -> Result<String, ExtractorError> {
        let pdf = self
            .pdfium
            .load_pdf_from_byte_slice(document.content.as_ref(), None)?;

        let mut text = String::new();

        for page in pdf.pages().iter() {
            let page_text = page.text()?.all();

            text.push_str(&page_text);
            text.push_str("\n\n");
        }

        Ok(text)
    }
}
