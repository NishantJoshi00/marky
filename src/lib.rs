use std::sync::Arc;

use tokio::sync::RwLock;
use tokio::task;
use tower_lsp::jsonrpc::Result;
use tower_lsp::{LanguageServer, lsp_types};

mod config;
mod embedding;
pub mod handler;
mod llm;

#[macro_use]
mod logging;

#[allow(dead_code)]
enum Artifacts {
    Lazy,
    Loaded {
        embedding: embedding::Client,
        llm: llm::Client,
    },
}

#[allow(dead_code)]
impl Artifacts {
    pub fn activate(&mut self, config: config::Config) -> anyhow::Result<()> {
        if let Self::Lazy = self {
            let embedding = embedding::Client::new(config.embedding);
            let llm = llm::Client::new(config.llm);

            *self = Self::Loaded { embedding, llm };
        }
        Ok(())
    }

    pub fn llm(&self) -> anyhow::Result<&llm::Client> {
        match self {
            Self::Lazy => {
                debug_assert!(false, "LLM client not initialized");
                anyhow::bail!("LLM client not initialized");
            }
            Self::Loaded { llm, .. } => Ok(llm),
        }
    }

    pub fn embedding(&self) -> anyhow::Result<&embedding::Client> {
        match self {
            Self::Lazy => {
                debug_assert!(false, "Embedding client not initialized");
                anyhow::bail!("Embedding client not initialized");
            }
            Self::Loaded { embedding, .. } => Ok(embedding),
        }
    }
}

pub struct Project {
    pub current_file: Arc<RwLock<Option<handler::Handle>>>,
    pub registry: handler::registry::Registry,
}

pub struct Backend {
    client: tower_lsp::Client,
    artifacts: Arc<RwLock<Artifacts>>,
    project: Project,
}

impl Backend {
    pub fn new(client: tower_lsp::Client) -> Self {
        Self {
            client,
            artifacts: Arc::new(RwLock::new(Artifacts::Lazy)),
            project: Project {
                current_file: Arc::new(RwLock::new(None)),
                registry: handler::registry::Registry::new(),
            },
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
            text_document_sync: Some(lsp_types::TextDocumentSyncCapability::Kind(
                lsp_types::TextDocumentSyncKind::FULL,
            )),
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

    async fn hover(&self, params: lsp_types::HoverParams) -> Result<Option<lsp_types::Hover>> {
        let loc = params.text_document_position_params.position;

        if let Some(handle) = self.project.current_file.read().await.as_ref() {
            #[allow(clippy::as_conversions)]
            let block = handle.get_block(loc.line as usize, loc.character as usize);
            if let Some(block) = block {
                let stats = [
                    format!("lines = {}", block.stat.lines),
                    format!("words = {}", block.stat.words),
                    format!("average.line_size = {}", block.stat.avg_line_size),
                ]
                .join("\n");

                let keywords = self.project.registry.get_keywords(&block);
                let mut data = ["[statistics]", &stats].join("\n");

                if let Some(keywords) = keywords {
                    let keywords = keywords
                        .iter()
                        .map(|value| format!("\"{}\"", value))
                        .collect::<Vec<_>>()
                        .join(", ");
                    data.push_str(
                        &["", "[analytics]", &format!("keywords = [{}]", keywords)].join("\n"),
                    );
                }

                let hover = lsp_types::Hover {
                    contents: lsp_types::HoverContents::Scalar(
                        lsp_types::MarkedString::LanguageString(lsp_types::LanguageString {
                            language: "toml".to_string(),
                            value: data,
                        }),
                    ),
                    range: Some(lsp_types::Range {
                        start: loc,
                        end: loc,
                    }),
                };

                return Ok(Some(hover));
            }
        }

        Ok(None)
    }

    async fn did_open(&self, params: lsp_types::DidOpenTextDocumentParams) {
        self.client
            .log_message(
                lsp_types::MessageType::INFO,
                format!("[START] didOpen - {}", params.text_document.uri),
            )
            .await;

        let contents = params.text_document.text;

        let mut parser = tree_sitter::Parser::new();
        if parser.set_language(tree_sitter_md::language()).is_err() {
            error!(self, "Failed to set language");
            return;
        }

        info!(self, "parsing file: {}", params.text_document.uri);

        let handle = match handler::Handle::new(&contents, &mut parser) {
            Ok(handle) => handle,
            Err(e) => {
                error!(self, "Failed to parse file: {}", e);
                return;
            }
        };

        info!(self, "parsed file: {}", params.text_document.uri);

        if let Ok(blocks) = handle.blocks.clone().read() {
            let registry = self.project.registry.clone();
            let blocks = blocks.clone();

            task::spawn_blocking(move || {
                let _ = registry.index_text(&blocks);
            });
        } else {
            error!(self, "Failed to read blocks");
        }

        info!(self, "[END] didOpen - {}", params.text_document.uri);

        self.project.current_file.write().await.replace(handle);
    }

    async fn did_change(&self, changes: lsp_types::DidChangeTextDocumentParams) {
        let content = changes.content_changes.first();

        info!(self, "[START] didChange - {}", changes.text_document.uri);

        if let Some(content) = content {
            let text = content.text.clone();
            let mut parser = tree_sitter::Parser::new();
            if parser.set_language(tree_sitter_md::language()).is_err() {
                error!(self, "Failed to set language");
                return;
            }

            let mut handle = self.project.current_file.write().await;
            if let Some(handle) = handle.as_mut() {
                if handle.update(&text, &mut parser).is_err() {
                    error!(self, "Failed to update file: {}", changes.text_document.uri);
                    return;
                }
            }
        } else {
            warn!(self, "No content changes found");
        }
    }
}
