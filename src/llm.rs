pub mod ollama;


#[async_trait::async_trait]
pub trait Llm {
    async fn generate(&self, instruction: &str, prompt: &str) -> anyhow::Result<String>;
}

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

#[async_trait::async_trait]
impl Llm for Client {
    async fn generate(&self, instruction: &str, prompt: &str) -> anyhow::Result<String> {
        match self {
            Client::Ollama(client) => client.generate(instruction, prompt).await,
        }
    }
}
