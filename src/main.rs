use docstream::config::loader::AppConfig;
use docstream::pipeline::PipelineBuilder;
use docstream::rest::router;
use docstream::rest::state::AppState;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .unwrap();

    let app_config = AppConfig::from_env()?;
    let pipeline = Arc::new(PipelineBuilder::new(app_config).build().await?);

    let (doc_worker, embed_worker) = pipeline.spawn_workers();

    let state = AppState {
        pipeline: pipeline.clone(),
    };

    let app = router::router(state);
    let listener = TcpListener::bind("127.0.0.1:3000").await?;

    info!("REST server listening on http://127.0.0.1:3000");

    tokio::select! {
        res = axum::serve(listener, app) => {
            res?;
        }
        _ = doc_worker => {
            panic!("doc worker exited unexpectedly");
        }
        _ = embed_worker => {
            panic!("embed worker exited unexpectedly");
        }
    }

    Ok(())
}

//////////
//use docstream::core::DocumentJob;
//
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

// let search_result = pipeline.search("My strange dream", 7).await?;
// for result in search_result {
//     info!("chunk_id: {}, score: {}", result.chunk_id, result.score);
//     println!("   doc_id: {:?}", result.metadata.document_id);
//     println!("   text: {:?}", result.metadata.text);
//}
// match tokio::try_join!(doc_dispatcher, embed_dispatcher) {
//     Ok(_) => info!("Pipeline completed successfully."),
//     Err(e) => error!("Error: {}", e),
// }
