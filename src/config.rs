use crate::{embedding, llm};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
#[serde(default)]
pub struct Config {
    pub embedding: embedding::Config,
    pub llm: llm::Config,
}
