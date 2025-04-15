use tower_lsp::{LspService, Server};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(marky::Backend::new);

    Server::new(stdin, stdout, socket).serve(service).await;

    Ok(())
}
