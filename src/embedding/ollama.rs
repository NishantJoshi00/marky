use ollama_rs::generation::embeddings::request::GenerateEmbeddingsRequest;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct Config {
    host: String,
    port: u16,
    model: String,
    vector_size: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: "http://localhost".to_string(),
            port: 11434,
            model: "nomic-embed-text:latest".to_string(),
            vector_size: 768,
        }
    }
}

pub struct Client {
    client: ollama_rs::Ollama,
    config: Config,
}

impl Client {
    pub fn new(config: Config) -> Self {
        let client = ollama_rs::Ollama::new(config.host.to_string(), config.port);

        Self { client, config }
    }
}

#[async_trait::async_trait]
impl super::Embedding for Client {
    async fn embed_multiple(&self, texts: Vec<String>) -> anyhow::Result<Vec<Vec<f32>>> {
        let response = self
            .client
            .generate_embeddings(GenerateEmbeddingsRequest::new(
                self.config.model.clone(),
                ollama_rs::generation::embeddings::request::EmbeddingsInput::Multiple(texts),
            ))
            .await?
            .embeddings;

        Ok(response)
    }

    fn size(&self) -> usize {
        self.config.vector_size
    }
}
