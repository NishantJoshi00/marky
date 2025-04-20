use tower_lsp::lsp_types;

mod point;
mod range;

pub async fn hover(
    ctx: &crate::Backend,
    params: lsp_types::HoverParams,
) -> anyhow::Result<Option<lsp_types::Hover>> {
    let loc = params.text_document_position_params.position;

    point::hover(ctx, loc).await
}
