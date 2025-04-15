use std::sync::Arc;

use tokio::sync::RwLock;
use tower_lsp::jsonrpc::Result;
use tower_lsp::{LanguageServer, lsp_types};

mod config;
mod embedding;
mod llm;

enum Artifacts {
    Lazy,
    Loaded {
        embedding: embedding::Client,
        llm: llm::Client,
    },
}

impl Artifacts {
    pub fn activate(&mut self, config: config::Config) -> anyhow::Result<()> {
        if let Artifacts::Lazy = self {
            let embedding = embedding::Client::new(config.embedding);
            let llm = llm::Client::new(config.llm);

            *self = Artifacts::Loaded { embedding, llm };
        }
        Ok(())
    }

    pub fn llm(&self) -> anyhow::Result<&llm::Client> {
        match self {
            Artifacts::Lazy => {
                debug_assert!(false, "LLM client not initialized");
                anyhow::bail!("LLM client not initialized");
            }
            Artifacts::Loaded { llm, .. } => Ok(llm),
        }
    }

    pub fn embedding(&self) -> anyhow::Result<&embedding::Client> {
        match self {
            Artifacts::Lazy => {
                debug_assert!(false, "Embedding client not initialized");
                anyhow::bail!("Embedding client not initialized");
            }
            Artifacts::Loaded { embedding, .. } => Ok(embedding),
        }
    }
}

pub struct Backend {
    client: tower_lsp::Client,
    artifacts: Arc<RwLock<Artifacts>>,
}

impl Backend {
    pub fn new(client: tower_lsp::Client) -> Self {
        Self {
            client,
            artifacts: Arc::new(RwLock::new(Artifacts::Lazy)),
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(
        &self,
        info: lsp_types::InitializeParams,
    ) -> Result<lsp_types::InitializeResult> {
        let capabilities = lsp_types::ServerCapabilities {
            hover_provider: Some(lsp_types::HoverProviderCapability::Simple(true)),
            ..Default::default()
        };

        let config = match info.initialization_options {
            Some(value) => serde_json::from_value(value)
                .map_err(|e| tower_lsp::jsonrpc::Error::invalid_params(e.to_string()))?,
            None => Default::default(),
        };

        self.artifacts.write().await.activate(config).map_err(|e| {
            let mut error = tower_lsp::jsonrpc::Error::internal_error();
            error.message = format!("Failed to activate artifacts: {}", e).into();
            error
        })?;

        let server_info = lsp_types::ServerInfo {
            name: env!("CARGO_PKG_NAME").to_string(),
            version: Some(env!("CARGO_PKG_VERSION").to_string()),
        };

        Ok(lsp_types::InitializeResult {
            capabilities,
            server_info: Some(server_info),
        })
    }

    async fn initialized(&self, _info: lsp_types::InitializedParams) {
        self.client
            .log_message(lsp_types::MessageType::INFO, "server initialized!")
            .await;

        // TODO: Can index the entire workspace here
    }

    async fn shutdown(&self) -> Result<()> {
        // TODO: Clear all the data, from the vector store and the LLM

        Ok(())
    }

    async fn completion(
        &self,
        _: lsp_types::CompletionParams,
    ) -> Result<Option<lsp_types::CompletionResponse>> {
        // TODO: No plan on doing completion for now

        Err(tower_lsp::jsonrpc::Error::method_not_found())
    }

    async fn hover(&self, _params: lsp_types::HoverParams) -> Result<Option<lsp_types::Hover>> {
        let hover = lsp_types::Hover {
            contents: lsp_types::HoverContents::Scalar(lsp_types::MarkedString::String(
                "Hello world".to_string(),
            )),
            range: None,
        };

        Ok(Some(hover))
    }

    async fn did_open(&self, _params: lsp_types::DidOpenTextDocumentParams) {
        // TODO: Handle document open (like indexing chunking and embedding) this is the current
        // document
    }

    async fn did_change(&self, _changes: lsp_types::DidChangeTextDocumentParams) {
        // TODO: Update the storage structure for the current document
    }
}
