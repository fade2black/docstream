use crate::chunker::ChunkerError;
use crate::core::Chunk;
use async_trait::async_trait;
use tokio::sync::mpsc;
use uuid::Uuid;

pub struct SimpleChunker;

impl SimpleChunker {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl crate::chunker::Chunker for SimpleChunker {
    async fn chunk(
        &self,
        content: &str,
        source_id: Uuid,
        tx: mpsc::Sender<Chunk>,
    ) -> Result<(), ChunkerError> {
        if content.trim().is_empty() {
            return Err(ChunkerError::EmptyContent);
        }

        let sentences: Vec<&str> = sentencex::segment("en", content)
            .into_iter()
            .filter(|s| !s.trim().is_empty())
            .collect();

        let mut i = 0;

        while i < sentences.len() {
            let first = sentences[i];
            let second = sentences.get(i + 1);

            let text = match second {
                Some(s2) => format!("{} {}", first, s2),
                None => first.to_string(),
            };

            let chunk = Chunk {
                id: Uuid::new_v4(),
                doc_id: source_id,
                text,
            };

            // info!(
            //     "doc_id={}, chunk_id={}, text={}",
            //     chunk.doc_id, chunk.id, chunk.text
            // );

            tx.send(chunk)
                .await
                .map_err(|e| ChunkerError::ChunkingFailed(e.to_string()))?;

            i += 2;
        }

        Ok(())
    }
}
