// How it runs

use crate::chunker::Chunker;
use crate::core::Chunk;
use crate::core::Document;
use crate::core::DocumentJob;
use crate::extractor::Extractor;
use crate::fetcher::Fetcher;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::info;

pub struct DocWorker {
    fetcher: Arc<dyn Fetcher>,
    extractor: Arc<dyn Extractor>,
    chunker: Arc<dyn Chunker>,
}

// TODO:
// replace `Result<(), String>` with `Result<(), PipelineError>`
// so we can:
// - retry fetch failures
// - ignore empty docs
// - classify extractor errors
impl DocWorker {
    pub fn new(
        fetcher: Arc<dyn Fetcher>,
        extractor: Arc<dyn Extractor>,
        chunker: Arc<dyn Chunker>,
    ) -> Self {
        Self {
            fetcher,
            extractor,
            chunker,
        }
    }

    pub async fn process(
        &self,
        job: &DocumentJob,
        sender: mpsc::Sender<Chunk>,
    ) -> anyhow::Result<()> {
        info!(doc_id=%job.doc_id, doc_ref=%job.doc_ref, "processing document");

        // 1. Fetch document
        let fetched_data = self.fetcher.fetch(job).await?;
        let document = Document {
            id: job.doc_id,
            source: fetched_data.source,
            content: fetched_data.content,
            content_type: fetched_data.content_type,
        };
        info!("Fetched document: {}", job.doc_id);

        // 2. Extract text
        let text = self.extractor.extract(&document).await?;
        info!("Extracted document: {}", job.doc_id);

        info!("Chunking document: {}", job.doc_id);
        // 3. Chunk + stream output
        self.chunker.chunk(&text, job.doc_id, sender).await?;

        Ok(())
    }
}
