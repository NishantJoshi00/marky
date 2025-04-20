# Marky

Marky is a Language Server Protocol (LSP) implementation for Markdown files that leverages embeddings and LLMs to provide intelligent assistance.

## Features

- **LSP Integration**: Seamlessly integrates with any editor supporting the Language Server Protocol
- **AI-Powered**: Uses Ollama models for both embeddings and language model functionality
- **Hover Information**: Provides contextual information when hovering over text
- **Configurable**: Easily configure embedding and LLM settings through LSP initialization

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (2021 edition or newer)
- [Ollama](https://ollama.ai/) running locally with:
  - `nomic-embed-text` model for embeddings
  - `llama3.2` model for language model capabilities

## Installation

```bash
# Clone the repository
git clone https://github.com/NishantJoshi00/marky.git
cd marky

# Build the project
cargo build --release

# The binary will be available at target/release/marky-stdio
```

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

### Neovim (with nvim-lspconfig)

Add the following to your Neovim configuration:

```lua
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
    cmd = { "/path/to/marky/target/release/marky-stdio" },
    filetypes = { "markdown", "md" },
    root_dir = function()
        return vim.fn.getcwd()
    end,
})
```

## Milestones and TODOs

The project has several TODOs marked in the codebase that need to be addressed:

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
  - [ ] Reference: `initialized` method in `lib.rs`

- [ ] **Document Management**
  - [ ] Handle document opening for indexing, chunking and embedding
  - [ ] Update storage structure when documents change
  - [ ] Reference: `did_open` and `did_change` methods in `lib.rs`

- [ ] **Completion Provider**
  - [ ] Decide whether to implement completion functionality
  - [ ] Reference: `completion` method in `lib.rs`

- [ ] **Shutdown Cleanup**
  - [ ] Implement cleanup of data from vector store and LLM on shutdown
  - [ ] Reference: `shutdown` method in `lib.rs`

- [ ] **Hover Content**
  - [ ] Replace "Hello world" placeholder with meaningful content
  - [ ] Implement context-aware hover information
  - [ ] Reference: `hover` method in `lib.rs`

## Architecture

Marky consists of two main components:

1. **Embedding Client**: Handles text embeddings using Ollama's API
2. **LLM Client**: Manages language model interactions through Ollama

Both components are initialized lazily when the language server is started.

## Development

```bash
# Run the language server
cargo run --bin marky-stdio

# Build the project
cargo build

# Build for release
cargo build --release
```
---

<p align="center">Built by Human, Documented by LLM.</p>

