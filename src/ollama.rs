use anyhow::bail;
use reqwest::Client;
use serde::Deserialize;
use tokenizers::Tokenizer;

#[derive(Deserialize)]
struct EmbeddingResponse {
    embedding: Vec<f32>,
}

pub struct Ollama {
    client: Client,
    address: String,
    tokenizer: Tokenizer,
}

impl Ollama {
    pub fn from_url(address: String) -> anyhow::Result<Self> {
        let tokenizer =
            Tokenizer::from_pretrained("mixedbread-ai/mxbai-embed-large-v1", None).unwrap();

        Ok(Self {
            client: Client::new(),
            address,
            tokenizer,
        })
    }

    pub async fn embeddings(&self, text: &str) -> anyhow::Result<Vec<(String, Vec<f32>)>> {
        let mut embeddings = Vec::new();

        let chunks = self.chunks(text, 512, 56)?;

        for chunk in chunks {
            let embedding = self.generate_embedding(&chunk).await?;

            embeddings.push((chunk, embedding));
        }

        Ok(embeddings)
    }

    fn chunks(
        &self,
        text: &str,
        chunk_size: usize,
        chunk_overlap: usize,
    ) -> anyhow::Result<Vec<String>> {
        let mut chunks = Vec::new();

        let tokens = self.tokenizer.encode(text, true).unwrap();

        if tokens.get_ids().len() <= chunk_size {
            chunks.push(text.to_string());
            return Ok(chunks);
        }

        let tokens = tokens.get_tokens();

        let mut start = 0usize;
        while start < tokens.len() {
            let end = (start + chunk_size).min(tokens.len());

            chunks.push(tokens[start..end].concat());

            start += chunk_size - chunk_overlap
        }

        Ok(chunks)
    }

    async fn generate_embedding(&self, text: &str) -> anyhow::Result<Vec<f32>> {
        let url = format!("{}/api/embeddings", self.address);

        let model = "mxbai-embed-large";

        let response = self
            .client
            .post(url)
            .body(format!(
                "{{\"model\":\"{}\",\"prompt\":\"{}\"}}",
                model, text
            ))
            .send()
            .await?;

        let bytes = response.bytes().await?.to_vec();

        let mut embedding = match serde_json::from_slice::<EmbeddingResponse>(&bytes) {
            Ok(r) => r.embedding,
            Err(e) => {
                bail!(
                    "Couldn't parse embedding response '{}': {}",
                    String::from_utf8_lossy(&bytes),
                    e
                )
            }
        };

        embedding.shrink_to_fit();

        Ok(embedding)
    }
}
