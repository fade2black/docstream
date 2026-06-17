mod core;
mod extractor;
mod fetcher;

use aws_sdk_s3::Client;

use crate::extractor::Extractor;
use crate::fetcher::Fetcher;
use crate::fetcher::local_file::LocalFileFetcher;
use crate::fetcher::retry::RetryFetcher;
use crate::fetcher::s3::S3Fetcher;

use std::sync::Arc;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let local_fetcher = LocalFileFetcher;
    //let extractor = extractor::text::TextExtractor::new();
    let extractor = extractor::pdf::PdfExtractor::new()?;

    let retry_fetcher = RetryFetcher::new(
        Arc::new(local_fetcher),
        3,   // max retries
        200, // base delay ms
    );

    let doc = retry_fetcher.fetch("data/sample.pdf").await?;
    let text = extractor.extract(&doc).await?;

    info!(
        "Fetched document: id={}, source={}, bytes={}",
        doc.id,
        doc.source,
        doc.content.len()
    );

    println!("Extracted text: {}", text);

    info!("--------------------------------------------------------------------------");
    // Read from AWS S3
    let config = aws_config::load_from_env().await;
    let client = Client::new(&config);

    let s3_fetcher = S3Fetcher::new(client);

    let retry_fetcher = RetryFetcher::new(
        Arc::new(s3_fetcher),
        5,   // max retries
        200, // base delay ms
    );

    let doc = retry_fetcher.fetch("samples10dj37he:sample.pdf").await?;
    let text = extractor.extract(&doc).await?;

    info!(
        "Fetched document: id={}, source={}, bytes={}",
        doc.id,
        doc.source,
        doc.content.len()
    );
    println!("Extracted text: {}", text);

    Ok(())
}
