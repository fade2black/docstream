use docstream::config::loader::AppConfig;
//use docstream::core::DocumentJob;
use docstream::pipeline::PipelineBuilder;
use tracing::info;
//use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .unwrap();

    let app_config = AppConfig::from_env()?;

    info!("provider: {}", app_config.embedder.provider);
    info!("endpoint: {}", app_config.embedder.endpoint);
    info!("model: {}", app_config.embedder.model);
    info!("qdrant: {:?}", app_config.qdrant);

    let pipeline = PipelineBuilder::new(app_config).build().await?;
    //let (doc_dispatcher, embed_dispatcher) = pipeline.spawn_workers().await?;

    //////////
    // let job = DocumentJob {
    //     doc_id: Uuid::new_v4(),
    //     doc_ref: String::from("data/sample.txt"),
    // };
    // pipeline.push(job).await?;

    // let job = DocumentJob {
    //     doc_id: Uuid::new_v4(),
    //     doc_ref: String::from("data/sample2.txt"),
    // };
    // pipeline.push(job).await?;

    let search_result = pipeline.search("My strange dream", 7).await?;
    for result in search_result {
        info!("chunk_id: {}, score: {}", result.chunk_id, result.score);
        println!("   doc_id: {:?}", result.metadata.document_id);
        println!("   text: {:?}", result.metadata.text);
    }
    // match tokio::try_join!(doc_dispatcher, embed_dispatcher) {
    //     Ok(_) => info!("Pipeline completed successfully."),
    //     Err(e) => error!("Error: {}", e),
    // }

    Ok(())
}
