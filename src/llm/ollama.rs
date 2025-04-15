use ollama_rs::{generation::completion::request::GenerationRequest, models};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct Config {
    host: String,
    port: u16,
    model: String,
    guard_prompt: Option<String>,
    temperature: Option<f32>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: "http://localhost".to_string(),
            port: 11434,
            model: "llama3.2:latest".to_string(),
            guard_prompt: Some("You are a helpful assistant. Respond with concise and clear responses; keep it short.".to_string()),
            temperature: Some(0.2),
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
impl super::Llm for Client {
    async fn generate(&self, instruction: &str, prompt: &str) -> anyhow::Result<String> {
        let mut options = models::ModelOptions::default();
        if let Some(temp) = self.config.temperature {
            options = options.temperature(temp);
        }
        let mut prompt = prompt.to_string();
        if let Some(guard_prompt) = &self.config.guard_prompt {
            prompt = format!(
                "{}\n\n### Instruction:\n{}\n\n### Context:\n\n{}",
                guard_prompt, instruction, prompt
            );
        } else {
            prompt = format!(
                "### Instruction:\n{}\n\n## Context:\n\n{}",
                instruction, prompt
            );
        }

        let request =
            GenerationRequest::new(self.config.model.clone(), prompt.to_string()).options(options);

        let response = self.client.generate(request).await?;

        Ok(response.response)
    }
}
