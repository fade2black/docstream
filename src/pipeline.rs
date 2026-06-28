// I define here how it's wired

//use crate::chunker::simple::SimpleChunker;
use crate::core::Chunk;
use crate::core::DocumentJob;
//use crate::embedder::EmbedderConfig;
// use crate::extractor::text::TextExtractor;
// use crate::fetcher::local_file::LocalFileFetcher;
// use crate::fetcher::retry::RetryFetcher;
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
}

impl Pipeline {
    pub fn new(
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

    pub async fn run(&self) -> Result<(JoinHandle<()>, JoinHandle<()>), PipelineError> {
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
                        None => break,
                    }
                };

                let permit = match sem.clone().acquire_owned().await {
                    Ok(p) => p,
                    Err(_) => break,
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

                info!("Spawning embedder worker: chunk_id={}", chunk.id);
                tokio::spawn(async move {
                    let vec = match embedder.embed(&chunk).await {
                        Ok(vec) => vec,
                        Err(e) => {
                            error!("Failed to embed chunk: {:?}", e);
                            return;
                        }
                    };
                    // Implement later: store embedding in database
                    info!("vec=[{}, {}, ..., {}]", vec[0], vec[1], vec[vec.len() - 1]);

                    drop(permit);
                    info!("Embedder worker chunk_id={} completed", chunk.id);
                });
            }
        })
    }
}

// let doc_rx = self.doc_rx.clone();
// let doc_semaphore = self.doc_semaphore.clone();
// let fetcher = self.fetcher.clone();
// let extractor = self.extractor.clone();
// let chunker = self.chunker.clone();
// let chunk_tx = self.chunk_tx.clone();

// tokio::spawn(async move {
//     loop {
//         let mut rx = doc_rx.lock().await;

//         let job = match rx.recv().await {
//             Some(job) => job,
//             None => break,
//         };

//         let permit = match doc_semaphore.clone().acquire_owned().await {
//             Ok(p) => p,
//             Err(_) => break,
//         };

//         let fetcher = fetcher.clone();
//         let extractor = extractor.clone();
//         let chunker = chunker.clone();
//         let chunk_tx = chunk_tx.clone();

//         tokio::spawn(async move {
//             // doc worker will go here
//             drop(permit);
//         });
//     }
// });

// use crate::chunker::simple::SimpleChunker;
// use crate::core::Chunk;
// use crate::extractor::text::TextExtractor;
// use crate::fetcher::local_file::LocalFileFetcher;
// use crate::fetcher::retry::RetryFetcher;
// use crate::prelude::*;

// pub struct Pipeline {
//     doc_ref: String,
// }

// impl Pipeline {
//     pub fn new(doc_ref: impl Into<String>) -> Self {
//         Self {
//             doc_ref: doc_ref.into(),
//         }
//     }

//     pub async fn run(self) -> anyhow::Result<()> {
//         let (tx, mut rx) = tokio::sync::mpsc::channel::<Chunk>(100);

//         tokio::spawn(async move {
//             while let Some(chunk) = rx.recv().await {
//                 tracing::info!(
//                     "chunk: doc={}, id={}, text={}",
//                     chunk.doc_id,
//                     chunk.id,
//                     chunk.text
//                 )
//             }
//         });

//         let fetcher = RetryFetcher::new(std::sync::Arc::new(LocalFileFetcher), 3, 200);

//         let extractor = TextExtractor::new();
//         let chunker = SimpleChunker::new();

//         let worker = DocWorker::new(fetcher, extractor, chunker);

//         worker.process(&self.doc_ref, tx).await?;

//         Ok(())
//     }
// }

// use crate::prelude::*;
// use std::sync::Arc;
// use tokio::sync::mpsc;
// use tracing::info;

// use crate::chunker::simple::SimpleChunker;
// use crate::extractor::text::TextExtractor;
// use crate::fetcher::local_file::LocalFileFetcher;
// use crate::fetcher::retry::RetryFetcher;

// pub struct Pipeline;

// impl Pipeline {
//     pub fn new() -> Self {
//         Self
//     }

//     pub async fn run(&self) -> anyhow::Result<()> {
//         // 1. Channel (pipeline boundary: chunk → next stage)
//         let (tx, mut rx) = mpsc::channel::<Chunk>(100);

//         // 2. Receiver (temporary: logs chunks)
//         tokio::spawn(async move {
//             info!("[RECEIVER] started");

//             while let Some(chunk) = rx.recv().await {
//                 info!(
//                     "RECEIVER: doc_id={}, chunk_id={}, text={}",
//                     chunk.doc_id, chunk.id, chunk.text
//                 );
//             }
//         });

//         // 3. Components
//         let local_fetcher = LocalFileFetcher;

//         let retry_fetcher = RetryFetcher::new(Arc::new(local_fetcher), 3, 200);

//         let extractor = TextExtractor::new();
//         let chunker = SimpleChunker::new();

//         // 4. Worker
//         let worker = DocWorker::new(retry_fetcher, extractor, chunker);

//         // 5. Run pipeline
//         worker.process("data/sample.txt", tx).await?;

//         // 6. keep process alive for debug visibility (temporary)
//         tokio::time::sleep(std::time::Duration::from_secs(2)).await;

//         Ok(())
//     }
// }
