use crate::extractor::Document;
use crate::extractor::ExtractorError;
use crate::extractor::pdf::PdfExtractor;
use crate::extractor::text::TextExtractor;
use crate::prelude::Extractor;
use async_trait::async_trait;

pub struct ExtractorRouter {
    text: TextExtractor,
    pdf: PdfExtractor,
}

impl ExtractorRouter {
    pub fn new() -> Result<Self, ExtractorError> {
        Ok(ExtractorRouter {
            text: TextExtractor::new(),
            pdf: PdfExtractor::new()?,
        })
    }
}

#[async_trait]
impl Extractor for ExtractorRouter {
    async fn extract(&self, document: &Document) -> Result<String, ExtractorError> {
        match document.content_type.as_str() {
            "application/pdf" => self.pdf.extract(document).await,
            _ => self.text.extract(document).await,
        }
    }
}
