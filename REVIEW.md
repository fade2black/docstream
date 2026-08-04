# docstream — Full Codebase Review

Findings-only report, ranked critical → minor. No code changes were made as part of this review.

## Critical

1. **Zero automated test coverage anywhere.** No `#[test]`, `#[cfg(test)]`, or `tests/` directory exist in any of the 28 source files; no `[dev-dependencies]` in `Cargo.toml`. Even trivially-testable pure logic is untested: `SimpleChunker::chunk` (`src/chunker/simple.rs`), `RetryFetcher` backoff math (`src/fetcher/retry.rs`), `QdrantStore::parse_chunk_id`/`parse_metadata` (`src/store/qdrant.rs`), `utils::dot`. The whole fetch→extract→chunk→embed→store pipeline and REST layer have no regression safety net.

2. **A background-loop edge case can panic the entire server process.** `src/main.rs:35,38` `panic!()` if either dispatcher's `JoinHandle` resolves. Those loops (`src/pipeline.rs:111-213`) only exit when their channel closes or a semaphore `acquire_owned()` errors — rare today, but any future path that closes a semaphore or drops all senders early takes down `main`, killing all in-flight HTTP traffic including `/health`.

## Moderate

3. **Dead/unwired code creates a false impression of feature support.** `S3Fetcher` (`fetcher/s3.rs`) and `PdfExtractor` (`extractor/pdf.rs`) are fully implemented but never reached — `PipelineBuilder::build()` (`pipeline.rs:221-235`) hardcodes `LocalFileFetcher`+`TextExtractor` with no content-type dispatch. Two drifted duplicate `QdrantConfig` structs exist (`config/loader.rs:10-22` real one vs. dead `store/mod.rs:7-13` with a different `vector_size` type). `rest/error.rs` is a fully empty file despite being declared as a module — a centralized HTTP error type was clearly planned, never built. `core::Metadata` and `ConfigError` are defined but never used.

4. **Typed errors get discarded right before they'd be useful.** `DocWorker::process` (`worker/doc_worker.rs:38-42`) collapses `FetcherError`/`ExtractorError`/`ChunkerError` into opaque `anyhow::Error`; the code's own `TODO` (`:19-24`) acknowledges this blocks retry/error-classification logic.

5. **REST errors collapse into undifferentiated 500s.** No leak of internal details, but `search`/`ingest_document` handlers give clients no way to distinguish bad input from a Qdrant outage; `search` also returns a `200`-shaped body on a `500` status.

6. **Worker/chunk-level panics are invisible.** Per-job/per-chunk tasks spawned via bare `tokio::spawn` with discarded `JoinHandle`s (`pipeline.rs:149,189`); an unchecked `vec[0]` index (`pipeline.rs:200`) would panic silently if an embedder ever returned an empty vector — no `tracing` log, just tokio's default stderr message.

7. **Blocking, CPU-bound work runs inside `async fn`.** `PdfExtractor::extract` (`extractor/pdf.rs:26-37`) calls synchronous `pdfium_render` with no `spawn_blocking` — latent since it's unwired today, but will stall the runtime once connected.

8. **Hardcoded magic numbers that belong in config**: retry count/backoff (`pipeline.rs:224`), concurrency limits `50`/`100` (`:232-234`), channel-buffer multipliers, and the chunking window (fixed 2-sentence pairing, no size/overlap bound — a single long sentence produces an unbounded chunk).

9. **A new `reqwest::Client` is built on every embedding call** (`embedder/mod.rs:22`) instead of reused — defeats connection pooling on the hottest per-chunk network call.

10. **Duplicated ~50-line dispatcher loops** (`spawn_doc_dispatcher`/`spawn_embed_dispatcher`, `pipeline.rs:111-213`) — any backpressure/error-handling fix needs to be applied twice.

11. **Two rustls versions in the dependency graph** (0.23.41 direct + 0.21.12 transitive) — extra compile time/size, worth a `cargo tree -d`.

12. **Loose `aws-sdk-s3`/`aws-config = "1"` paired with `behavior-version-latest`**, which is explicitly designed to change runtime behavior across SDK releases without a major bump — a routine `cargo update` could silently change retry/timeout/credential behavior.

13. **A prebuilt native binary is committed to git**: `vendor/pdfium/libpdfium.dylib` (7.4MB, macOS-only), no visible provenance/checksum, will break non-macOS CI.

## Minor

- 4 real clippy warnings (otherwise clean): missing `Default` impls for `SimpleChunker::new`/`TextExtractor::new`, unnecessary closure in `fetcher/s3.rs:21`, useless `Bytes::from(...)` conversion in `fetcher/s3.rs:46`.
- `tokio` uses `features = ["full"]` in a service binary — wasteful compile time/binary size.
- `axum` pinned a major version behind upstream (`0.7.9` vs `0.8.x`).
- Pre-1.0 crates (`pdfium-render`, `sentencex`) not exactly pinned despite being free to break on patch releases.
- `PipelineError::Internal(String)` is a stringly-typed catch-all inside an otherwise well-typed enum.
- `Chunker::chunk` takes an `mpsc::Sender<Chunk>` directly in its trait signature, coupling it to a transport instead of returning a `Stream`/`Vec`.
- Typos (`ConfigError::UnableToLoadFromFlle`, "queued for injestion"), commented-out dead code in `main.rs:45-69` and `chunker/simple.rs:49-52`, inconsistent zero-field-struct constructor pattern, vestigial `EmbedderConfig.provider`, likely-dead `utils::dot`.
- `Cargo.lock` correctly committed for a binary crate, but currently has uncommitted local changes.

## Per-topic summary

- **Structure**: layered around 5 async traits (`Fetcher`/`Extractor`/`Chunker`/`Embedder`/`VectorStore`) re-exported via `prelude.rs`; `pipeline.rs` is the composition root but does triple duty as DI container, error-type owner, and scheduler.
- **Error handling**: `thiserror` for domain errors, `anyhow` only at true edges — correct split, one exception in `doc_worker.rs`. All 3 panics in the whole tree live in `main.rs`; one is reasonable startup fail-fast, two are process-killing (#2). No info-leak to HTTP clients found.
- **Unsafe**: none in `src/` at all; `pdfium-render` is used entirely through its safe wrapper API.
- **Dependencies**: generally idiomatic; concerns are versioning/graph hygiene, not wrong choices.
- **Code quality**: clippy is nearly clean; real issues are structural (duplication, blocking-in-async, magic numbers, per-call client construction, dead modules).
- **Tests**: 0% coverage, project-wide.
