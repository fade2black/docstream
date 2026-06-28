//use docstream::chunker;
//use docstream::pipeline::Pipeline;
//use docstream::core::Chunk;
use docstream::chunker::simple::SimpleChunker;
use docstream::config::loader::AppConfig;
use docstream::core::DocumentJob;
use docstream::embedder::local_embedder::LocalEmbedder;
use docstream::extractor::text::TextExtractor;
use docstream::fetcher::{local_file::LocalFileFetcher, retry::RetryFetcher};
use docstream::pipeline::Pipeline;
use std::sync::Arc;
use tracing::{error, info};
use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let app_config = AppConfig::from_file("config.toml").unwrap();

    info!("provider: {}", app_config.embedder.provider);
    info!("endpoint: {}", app_config.embedder.endpoint);
    info!("model: {}", app_config.embedder.model);
    info!("api_key: {:?}", app_config.embedder.api_key);

    let embedder = Arc::new(LocalEmbedder::new(app_config.embedder));
    let fetcher = Arc::new(RetryFetcher::new(Arc::new(LocalFileFetcher), 5, 750));
    let extractor = Arc::new(TextExtractor::new());
    let chunker = Arc::new(SimpleChunker::new());
    let max_docs = 50;
    let max_embeds = 100;

    let pipeline = Pipeline::new(fetcher, extractor, chunker, embedder, max_docs, max_embeds);
    let (doc_dispatcher, embed_dispatcher) = pipeline.run().await?;

    let job = DocumentJob {
        doc_id: Uuid::new_v4(),
        doc_ref: String::from("data/sample.txt"),
    };
    pipeline.push(job).await?;

    let job = DocumentJob {
        doc_id: Uuid::new_v4(),
        doc_ref: String::from("data/sample2.txt"),
    };
    pipeline.push(job).await?;

    match tokio::try_join!(doc_dispatcher, embed_dispatcher) {
        Ok(_) => info!("Pipeline completed successfully."),
        Err(e) => error!("Error: {}", e),
    }

    Ok(())
}
