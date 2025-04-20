use tower_lsp::lsp_types;

pub async fn hover(
    ctx: &crate::Backend,
    loc: lsp_types::Position,
) -> anyhow::Result<Option<lsp_types::Hover>> {
    if let Some(handle) = ctx.project.current_file.read().await.as_ref() {
        #[allow(clippy::as_conversions)]
        let block = handle.get_block(loc.line as usize, loc.character as usize);
        if let Some(block) = block {
            let stats = [
                format!("lines = {}", block.stat.lines),
                format!("words = {}", block.stat.words),
                format!("average.line_size = {}", block.stat.avg_line_size),
            ]
            .join("\n");

            #[cfg(feature = "intelligence")]
            let keywords = ctx.project.registry.get_keywords(&block);
            #[cfg(not(feature = "intelligence"))]
            let keywords: Option<Vec<String>> = None;

            #[cfg(feature = "intelligence")]
            let summary = ctx.project.registry.get_summary(&block);
            #[cfg(not(feature = "intelligence"))]
            let summary: Option<String> = None;

            let mut data = ["[statistics]", &stats].join("\n");

            match (keywords, summary) {
                (None, None) => {}
                (a, b) => {
                    let mut list = vec!["".to_string(), "[analytics]".to_string()];

                    if let Some(keyword) = a {
                        let keywords = keyword
                            .iter()
                            .map(|value| format!("\"{}\"", value))
                            .collect::<Vec<_>>()
                            .join(", ");
                        list.push(format!("keywords = [{}]", keywords));
                    }

                    if let Some(summary) = b {
                        list.push(format!("summary = \"{}\"", summary));
                    }
                    data.push_str(&list.join("\n"));
                }
            }

            #[allow(clippy::as_conversions)]
            let start = lsp_types::Position {
                line: block.start.0 as u32,
                character: block.start.1 as u32,
            };

            #[allow(clippy::as_conversions)]
            let end = lsp_types::Position {
                line: block.end.0 as u32,
                character: block.end.1 as u32,
            };

            let hover = lsp_types::Hover {
                contents: lsp_types::HoverContents::Scalar(
                    lsp_types::MarkedString::LanguageString(lsp_types::LanguageString {
                        language: "toml".to_string(),
                        value: data,
                    }),
                ),
                range: Some(lsp_types::Range { start, end }),
            };

            return Ok(Some(hover));
        }
    }

    Ok(None)
}
