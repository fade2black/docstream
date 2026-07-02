// I define here how it's wired

use crate::chunker::simple::SimpleChunker;
use crate::config::loader::AppConfig;
use crate::core::Chunk;
use crate::core::DocumentJob;
use crate::embedder::local_embedder::LocalEmbedder;
use crate::extractor::text::TextExtractor;
use crate::fetcher::{local_file::LocalFileFetcher, retry::RetryFetcher};
use crate::store::VectorStoreError;
use crate::store::qdrant::QdrantStore;

use crate::prelude::*;
use std::sync::Arc;
use tokio::sync::{Mutex, Semaphore, mpsc};
use tokio::task::JoinHandle;
use tracing::{error, info};

pub struct Pipeline {
    // Components
    fetcher: Arc<dyn Fetcher>,
    extractor: Arc<dyn Extractor>,
    chunker: Arc<dyn Chunker>,
    embedder: Arc<dyn Embedder>,
    store: Arc<dyn VectorStore>,

    // Concurrency limits
    doc_semaphore: Arc<Semaphore>,
    embed_semaphore: Arc<Semaphore>,

    // Internal channels
    doc_tx: mpsc::Sender<DocumentJob>,
    doc_rx: Arc<Mutex<mpsc::Receiver<DocumentJob>>>,

    chunk_tx: mpsc::Sender<Chunk>,
    chunk_rx: Arc<Mutex<mpsc::Receiver<Chunk>>>,
}

#[derive(Debug, thiserror::Error)]
pub enum PipelineError {
    #[error("pipeline internal error: {0}")]
    Internal(String),
    #[error("fetch error: {0}")]
    VectorStoreError(#[from] VectorStoreError),
}

pub struct PipelineBuilder {
    config: AppConfig,
}

impl Pipeline {
    pub fn new(
        store: Arc<dyn VectorStore>,
        fetcher: Arc<dyn Fetcher>,
        extractor: Arc<dyn Extractor>,
        chunker: Arc<dyn Chunker>,
        embedder: Arc<dyn Embedder>,
        max_docs: usize,
        max_embeds: usize,
    ) -> Self {
        let (doc_tx, doc_rx) = mpsc::channel::<DocumentJob>(max_docs * 2);
        let (chunk_tx, chunk_rx) = mpsc::channel::<Chunk>(max_embeds * 4);
        Self {
            store,
            fetcher,
            extractor,
            chunker,
            embedder,

            doc_semaphore: Arc::new(Semaphore::new(max_docs)),
            embed_semaphore: Arc::new(Semaphore::new(max_embeds)),

            doc_tx,
            doc_rx: Arc::new(Mutex::new(doc_rx)),

            chunk_tx,
            chunk_rx: Arc::new(Mutex::new(chunk_rx)),
        }
    }

    pub async fn push(&self, job: DocumentJob) -> Result<(), PipelineError> {
        let doc_id = job.doc_id;
        self.doc_tx
            .send(job)
            .await
            .map_err(|e| PipelineError::Internal(format!("doc_tx closed: {}", e)))?;

        info!("Doc worker: doc_id={} started", doc_id);
        Ok(())
    }

    pub async fn spawn_workers(&self) -> Result<(JoinHandle<()>, JoinHandle<()>), PipelineError> {
        Ok((self.spawn_doc_dispatcher(), self.spawn_embed_dispatcher()))
    }

    fn spawn_doc_dispatcher(&self) -> JoinHandle<()> {
        let doc_rx = self.doc_rx.clone();
        let sem = self.doc_semaphore.clone();

        let fetcher = self.fetcher.clone();
        let extractor = self.extractor.clone();
        let chunker = self.chunker.clone();
        let chunk_tx = self.chunk_tx.clone();

        tokio::spawn(async move {
            loop {
                let job = {
                    let mut rx = doc_rx.lock().await;
                    match rx.recv().await {
                        Some(job) => job,
                        None => {
                            info!("No more jobs to process");
                            break;
                        }
                    }
                };

                let permit = match sem.clone().acquire_owned().await {
                    Ok(p) => p,
                    Err(e) => {
                        error!("Unable to acquire permit: {}", e);
                        break;
                    }
                };

                // Clone the necessary components for the worker body
                let fetcher = fetcher.clone();
                let extractor = extractor.clone();
                let chunker = chunker.clone();
                let chunk_tx = chunk_tx.clone();

                info!("Spawning doc worker: doc_id={}", job.doc_id);
                tokio::spawn(async move {
                    let worker = DocWorker::new(fetcher, extractor, chunker);

                    if let Err(e) = worker.process(&job, chunk_tx).await {
                        error!("Doc worker doc_id={} failed: {}", job.doc_id, e);
                    }
                    drop(permit);

                    info!("Doc worker doc_id={} completed", job.doc_id);
                });
            }
        })
    }

    fn spawn_embed_dispatcher(&self) -> JoinHandle<()> {
        let chunk_rx = self.chunk_rx.clone();
        let sem = self.embed_semaphore.clone();
        let embedder = self.embedder.clone();
        let store = self.store.clone();

        tokio::spawn(async move {
            loop {
                let chunk = {
                    let mut rx = chunk_rx.lock().await;
                    match rx.recv().await {
                        Some(chunk) => chunk,
                        None => break,
                    }
                };

                let permit = match sem.clone().acquire_owned().await {
                    Ok(permit) => permit,
                    Err(_) => break,
                };

                let embedder = embedder.clone();
                let store = store.clone();

                info!("Spawning embedder worker: chunk_id={}", chunk.id);
                tokio::spawn(async move {
                    let vec = match embedder.embed(&chunk).await {
                        Ok(vec) => vec,
                        Err(e) => {
                            error!("Failed to embed chunk: {:?}", e);
                            return;
                        }
                    };

                    info!(
                        "Storing embedding: vec=[{},...], size={}",
                        vec[0],
                        vec.len()
                    );

                    if let Err(e) = store.insert(&vec, &chunk).await {
                        info!("Failed to store embedding: {}", e);
                    }

                    drop(permit);
                    info!("Embedder worker chunk_id={} completed", chunk.id);
                });
            }
        })
    }
}

impl PipelineBuilder {
    pub fn new(config: AppConfig) -> Self {
        Self { config }
    }

    pub async fn build(self) -> Result<Pipeline, PipelineError> {
        let embedder = Arc::new(LocalEmbedder::new(self.config.embedder));

        let fetcher = Arc::new(RetryFetcher::new(Arc::new(LocalFileFetcher), 5, 750));

        let extractor = Arc::new(TextExtractor::new());

        let chunker = Arc::new(SimpleChunker::new());

        let store = Arc::new(QdrantStore::new(self.config.qdrant).await?);

        Ok(Pipeline::new(
            store, fetcher, extractor, chunker, embedder, 50, 100,
        ))
    }
}
