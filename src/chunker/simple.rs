use crate::chunker::ChunkerError;
use crate::core::Chunk;
use async_trait::async_trait;
use tokio::sync::mpsc;
use uuid::Uuid;

#[derive(Default)]
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

            tx.send(chunk)
                .await
                .map_err(|e| ChunkerError::ChunkingFailed(e.to_string()))?;

            i += 2;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunker::Chunker;

    #[tokio::test]
    async fn empty_content_is_rejected() {
        let chunker = SimpleChunker::new();
        let (tx, mut rx) = mpsc::channel(4);

        let result = chunker.chunk("   \n  ", Uuid::new_v4(), tx).await;

        assert!(matches!(result, Err(ChunkerError::EmptyContent)));
        assert!(rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn pairs_sentences_into_chunks() {
        let chunker = SimpleChunker::new();
        let (tx, mut rx) = mpsc::channel(4);
        let source_id = Uuid::new_v4();

        let content = "The cat sat on the mat. The dog barked loudly. \
                        Birds flew away quickly. The sun set slowly.";

        let result = chunker.chunk(content, source_id, tx).await;
        assert!(result.is_ok());

        let mut chunks = Vec::new();
        while let Some(chunk) = rx.recv().await {
            chunks.push(chunk);
        }

        assert_eq!(chunks.len(), 2);
        assert_eq!(
            chunks[0].text,
            "The cat sat on the mat.  The dog barked loudly. "
        );
        assert_eq!(
            chunks[1].text,
            "Birds flew away quickly.  The sun set slowly."
        );

        for chunk in &chunks {
            assert_eq!(chunk.doc_id, source_id);
        }
        assert_ne!(chunks[0].id, chunks[1].id);
    }
}
