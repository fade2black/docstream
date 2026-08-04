use docstream::config::loader::AppConfig;
use docstream::pipeline::PipelineBuilder;
use docstream::rest::router;
use docstream::rest::state::AppState;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::{error, info};

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
            error!("doc worker exited unexpectedly");
            anyhow::bail!("doc worker exited unexpectedly");
        }
        _ = embed_worker => {
            error!("embed worker exited unexpectedly");
            anyhow::bail!("embed worker exited unexpectedly");
        }
    }

    Ok(())
}
