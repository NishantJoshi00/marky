# Marky <img src="/assets/logo.png" alt="Marky Logo" align="right" width="150" />

Marky is a Language Server Protocol (LSP) implementation for Markdown files that leverages embeddings and LLMs to provide intelligent assistance and insights for your documentation.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Features

- **LSP Integration**: Seamlessly integrates with any editor supporting the Language Server Protocol
- **AI-Powered Insights**: 
  - Uses Ollama models for both embeddings and language model functionality
  - Generates keyword analysis of your Markdown content
  - Creates summaries of longer text blocks
  - Provides contextual statistical analysis
- **Hover Information**: Get detailed context when hovering over paragraphs and headings
- **Configurable**: Easily configure embedding and LLM settings through LSP initialization
- **Efficient Parsing**: Uses Tree-sitter for robust Markdown parsing and block analysis

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (2021 edition or newer)
- [Ollama](https://ollama.ai/) running locally with:
  - `nomic-embed-text` model for embeddings
  - `llama3.2` model for language model capabilities

You can install the required Ollama models with:

```bash
# Install nomic-embed-text for embeddings
ollama pull nomic-embed-text:latest

# Install llama3.2 for LLM capabilities
ollama pull llama3.2:latest
```

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/NishantJoshi00/marky.git
cd marky

# Build the project (without AI features)
cargo build --release

# Or build with intelligence features enabled
cargo build --release --features intelligence

# The binary will be available at target/release/marky-stdio
```

### Features

Marky supports the following feature flags:

- `intelligence`: Enables AI-powered features like keyword extraction and text summarization (requires additional dependencies)

## Configuration

Marky can be configured through the LSP initialization options:

```json
{
  "embedding": {
    "type": "Ollama",
    "config": {
      "host": "http://localhost",
      "port": 11434,
      "model": "nomic-embed-text:latest",
      "vector_size": 768
    }
  },
  "llm": {
    "type": "Ollama",
    "config": {
      "host": "http://localhost",
      "port": 11434,
      "model": "llama3.2:latest",
      "guard_prompt": "You are a helpful assistant. Respond with concise and clear responses; keep it short.",
      "temperature": 0.2
    }
  }
}
```

## Editor Integration

Marky communicates via stdio following the Language Server Protocol, making it compatible with any editor that supports LSP clients. Below are specific setup instructions for some popular editors:

### Neovim (with nvim-lspconfig)

Add the following to your Neovim configuration:

```lua
local marky_handler = function()
    local configs = require("lspconfig.configs")
    local util = require("lspconfig/util")

    if not configs.marky then
        configs.marky = {
            default_config = {},
            docs = {
                description = [[
            A custom Markdown language server named marky.
          ]],
                default_config = {},
            },
        }
    end

    require("lspconfig").marky.setup({
        -- For using a pre-built binary
        -- cmd = { "/path/to/marky/target/release/marky-stdio" },
        
        -- Or run directly from source (development)
        cmd = {
            "cargo", "run",
            "--release",
            "--bin", "marky-stdio",
            "--quiet",
        },
        cmd_cwd = "/path/to/marky/repo",
        filetypes = { "markdown", "md" },
        root_dir = function()
            return vim.fn.getcwd()
        end,
    })
end

-- Call the handler to set up Marky
marky_handler()
```

### VSCode

VSCode support is planned but not yet implemented.

### Other LSP-compatible Editors

Marky can be integrated with any editor that supports LSP clients. The general configuration needs:

- Command: Path to `marky-stdio` binary
- Language IDs: `markdown`, `md`
- Optional: LSP initialization options for embedding and LLM configuration

## Usage

Once Marky is integrated with your editor, it provides the following functionality:

### Hover Information

Hover over any paragraph or heading in a Markdown file to see:

1. **Statistics**:
   - Line count
   - Word count
   - Average line size (words per line)

2. **Analytics** (when built with `intelligence` feature):
   - Keywords extracted from the text
   - Summary of the content (for longer text blocks)

Example hover result:
```toml
[statistics]
lines = 3
words = 42
average.line_size = 14.0

[analytics]
keywords = ["markdown", "LSP", "intelligence"]
summary = "This paragraph describes the core functionality of the Marky language server."
```

## Architecture

Marky consists of several key components:

1. **LSP Backend**: Handles communication with editors through the Language Server Protocol
2. **Markdown Parser**: Uses Tree-sitter to parse and analyze Markdown documents
3. **Embedding Client**: Manages text embeddings using Ollama's API
4. **LLM Client**: Interfaces with language models through Ollama
5. **Registry**: Stores and manages document blocks, keywords, summaries, and embeddings

The system is designed to be modular, allowing for alternative backends for embedding and LLM functionality in the future.

## Development

### Project Structure

```
src/
├── bin/                  # Binary entry points
│   └── marky-stdio.rs    # LSP stdio server
├── config.rs             # Configuration handling
├── embedding/            # Embedding providers
│   └── ollama.rs         # Ollama implementation
├── embedding.rs          # Embedding trait definition
├── handler/              # Document handling
│   └── registry.rs       # Data registry
├── handler.rs            # Document parsing and block management
├── hover/                # Hover functionality
│   ├── point.rs          # Point-based hover
│   └── range.rs          # Range-based hover
├── hover.rs              # Hover implementation
├── lib.rs                # Main library code
├── llm/                  # LLM providers
│   └── ollama.rs         # Ollama implementation
├── llm.rs                # LLM trait definition
└── logging.rs            # Logging macros
```

### Running Tests

```bash
# Run all tests
cargo test

# Run benchmarks
cargo bench
```

### Building for Development

```bash
# Build with development features
cargo build --features intelligence

# Run the language server
cargo run --bin marky-stdio
```

## Milestones and Roadmap

The project has several milestones and TODOs:

- [x] **Basic LSP Setup**
  - [x] Server initialization 
  - [x] Client/server communication
  - [x] Core structure implementation

- [x] **Client Integration**
  - [x] Neovim integration via lspconfig
  - [ ] VSCode extension support
  - [ ] Other editor integrations

- [x] **Ollama Client Integration**
  - [x] Embedding client implementation
  - [x] LLM client implementation

- [ ] **Workspace Indexing**
  - [ ] Implement indexing of the entire workspace during initialization
  - [ ] Create vector store for embeddings

- [ ] **Document Management**
  - [ ] Handle document opening for indexing, chunking and embedding
  - [ ] Update storage structure when documents change

- [ ] **Enhanced Hover Content**
  - [ ] Implement context-aware hover information
  - [ ] Add semantic linking between related content

- [ ] **Completion Provider**
  - [ ] Implement intelligent auto-completion for Markdown

- [ ] **Performance Optimization**
  - [ ] Optimize memory usage for large documents
  - [ ] Implement caching for frequently accessed embeddings

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## Acknowledgments

* [Tower LSP](https://github.com/ebkalderon/tower-lsp) - The LSP implementation used by Marky
* [Tree-sitter](https://github.com/tree-sitter/tree-sitter) - Used for parsing Markdown documents
* [Ollama](https://ollama.ai/) - For providing the local inference API
* [rust-bert](https://github.com/guillaume-be/rust-bert) - For native Rust ML capabilities

---

<p align="center">Built by Human, Documented by LLM.</p>
