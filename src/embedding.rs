#[async_trait::async_trait]
pub trait Embedding {
    async fn embed(&self, text: String) -> anyhow::Result<Vec<f32>> {
        let texts = vec![text];
        let embeddings = self.embed_multiple(texts).await?;
        Ok(embeddings
            .into_iter()
            .next()
            .ok_or(anyhow::anyhow!("No embeddings returned"))?)
    }
    async fn embed_multiple(&self, texts: Vec<String>) -> anyhow::Result<Vec<Vec<f32>>>;
    fn size(&self) -> usize;
}

mod ollama;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", content = "config")]
pub enum Config {
    Ollama(ollama::Config),
}

impl Default for Config {
    fn default() -> Self {
        Self::Ollama(ollama::Config::default())
    }
}

pub enum Client {
    Ollama(ollama::Client),
}

impl Client {
    pub fn new(config: Config) -> Self {
        match config {
            Config::Ollama(config) => Self::Ollama(ollama::Client::new(config)),
        }
    }
}
