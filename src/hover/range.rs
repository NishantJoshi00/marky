use anyhow::bail;
use tower_lsp::lsp_types;

#[allow(dead_code)]
pub async fn hover(
    _ctx: &crate::Backend,
    _start: lsp_types::Position,
    _end: lsp_types::Position,
) -> anyhow::Result<Option<lsp_types::Hover>> {
    bail!("range hover not implemented");
}
