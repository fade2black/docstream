# docstream — Refactor Plan

Derived from `REVIEW.md`. Sequenced by what should logically be fixed first: crash risk → missing safety nets → structural debt → hygiene. Nothing here has been implemented yet.

1. **Fix the process-killing panic paths** — replace `panic!()` in `main.rs:35,38` with graceful handling of a resolved dispatcher `JoinHandle` (log + restart the loop, or a controlled shutdown) instead of taking down the whole server.

2. **Stand up a test foundation** — add `[dev-dependencies]`, then write unit tests for the pure/deterministic logic first: `SimpleChunker::chunk`, `RetryFetcher` backoff math, `QdrantStore::parse_chunk_id`/`parse_metadata`, `utils::dot`. Highest-leverage, low-effort win; de-risks every refactor after it.

3. **Resolve dead/duplicate code** — delete or wire up `S3Fetcher` and `PdfExtractor` (decide: content-type-based dispatch in `PipelineBuilder::build()`, or remove if out of scope); delete the unused `store::mod::QdrantConfig` and `ConfigError`; delete unused `core::Metadata`.

4. **Build a real REST error type** — implement `rest/error.rs` with a proper `IntoResponse` error enum, replace the ad-hoc per-handler status-code mapping in `handlers.rs`, and return structured JSON error bodies (400 vs 500 distinguished) instead of flat 500s.

5. **Stop discarding typed errors in `DocWorker::process`** — change its return type from `anyhow::Result<()>` to a typed error (or propagate `FetcherError`/`ExtractorError`/`ChunkerError` directly), so `pipeline.rs` can classify and retry instead of just logging a string. Resolves the TODO already in the code.

6. **Make worker/chunk task failures observable** — join the discarded `JoinHandle`s from `tokio::spawn` in `pipeline.rs:149,189` (or install a panic hook) and log failures via `tracing`; fix the unchecked `vec[0]` index at `pipeline.rs:200` with a length check.

7. **Move blocking PDF parsing off the async executor** — wrap `PdfExtractor::extract`'s `pdfium_render` calls in `tokio::task::spawn_blocking`.

8. **Move hardcoded magic numbers into `AppConfig`** — retry count/backoff (`pipeline.rs:224`), concurrency limits (`:232-234`), channel-buffer multipliers, and chunking window/size/overlap in `chunker/simple.rs`.

9. **Reuse a single `reqwest::Client`** in the embedder instead of constructing one per call (`embedder/mod.rs:22`). ✅

10. **Deduplicate the dispatcher loops** — extract `spawn_doc_dispatcher`/`spawn_embed_dispatcher`'s shared recv/acquire/spawn/log pattern into one generic helper.

11. **Dependency hygiene** — trim `tokio` from `features = ["full"]` to only what's used; investigate/eliminate the duplicate rustls 0.21/0.23 in the graph (`cargo tree -d`); pin `pdfium-render`/`sentencex` exactly; consider bumping `axum` to 0.8.

12. **Cleanup pass** — fix the 4 clippy warnings (`Default` impls, unnecessary closure, useless `Bytes::from`), typos (`UnableToLoadFromFlle`, "injestion"), remove commented-out dead code in `main.rs`/`chunker/simple.rs`, remove vestigial `EmbedderConfig.provider`.
