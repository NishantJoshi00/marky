use std::sync::Arc;

use std::sync::RwLock;

pub mod registry;

#[derive(Debug, Clone)]
pub struct Handle {
    tree: Arc<RwLock<tree_sitter::Tree>>,
    pub blocks: Arc<RwLock<Vec<Block>>>,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub text: String,
    pub start: (usize, usize),
    pub end: (usize, usize),
    pub stat: Stat,
    pub metadata: Metadata,
}

#[derive(Debug, Clone)]
pub struct Stat {
    pub lines: usize,
    pub words: usize,
    pub avg_line_size: f32,
    // TODO: pub avg_word_size: f32,
}

#[derive(Debug, Clone)]
pub struct Metadata {
    pub hash: [u8; 32], // blake3 hash
}

impl Handle {
    pub fn new(text: &str, parser: &mut tree_sitter::Parser) -> anyhow::Result<Self> {
        let tree = parser.parse(text, None).unwrap();
        let root_node = tree.root_node();
        let mut blocks = Vec::new();
        Self::construct_blocks(&root_node, &mut blocks, text)?;

        let blocks = Arc::new(RwLock::new(blocks));
        let tree = Arc::new(RwLock::new(tree));

        Ok(Self { tree, blocks })
    }
    pub fn update(&mut self, text: &str, parser: &mut tree_sitter::Parser) -> anyhow::Result<()> {
        let mut tree = self
            .tree
            .write()
            .map_err(|_| anyhow::anyhow!("Failed while writing to the tree"))?;
        *tree = parser.parse(text, Some(&tree)).unwrap();

        let mut blocks = self
            .blocks
            .write()
            .map_err(|_| anyhow::anyhow!("Failed while writing to the blocks"))?;
        blocks.clear();
        let root_node = tree.root_node();
        Self::construct_blocks(&root_node, &mut blocks, text)?;

        Ok(())
    }

    fn construct_blocks(
        node: &tree_sitter::Node,
        blocks: &mut Vec<Block>,
        text: &str,
    ) -> Result<(), anyhow::Error> {
        if node.kind() == "paragraph" || node.kind() == "heading_content" {
            let text = node.utf8_text(text.as_bytes())?.trim();
            let start = (node.start_position().row, node.start_position().column);
            let end = (node.end_position().row, node.end_position().column);
            let lines = text.split('.').filter(|line| !line.is_empty());
            let line_count = lines.clone().count();
            let total_word_count: usize = lines
                .map(|line| {
                    line.split_whitespace()
                        .filter(|word| !word.is_empty())
                        .count()
                })
                .sum();
            let avg_words_per_line = total_word_count as f32 / line_count as f32;

            let block = Block {
                text: text.to_string(),
                start,
                end,
                stat: Stat {
                    lines: line_count,
                    words: total_word_count,
                    avg_line_size: avg_words_per_line,
                },
                metadata: Metadata {
                    hash: *blake3::hash(text.as_bytes()).as_bytes(),
                },
            };

            blocks.push(block);

            Ok(())
        } else {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                Self::construct_blocks(&child, blocks, text)?;
            }

            Ok(())
        }
    }

    pub fn get_block(&self, target_row: usize, target_col: usize) -> Option<Block> {
        let blocks = self.blocks.read().expect("Failed to read blocks");

        // First, do a binary search to find blocks containing the target row
        let mut left = 0;
        let mut right = blocks.len();

        while left < right {
            let mid = left + (right - left) / 2;
            let block = &blocks[mid];

            // Check if target is before this block
            if target_row < block.start.0
                || (target_row == block.start.0 && target_col < block.start.1)
            {
                right = mid;
            }
            // Check if target is after this block
            else if target_row > block.end.0
                || (target_row == block.end.0 && target_col > block.end.1)
            {
                left = mid + 1;
            }
            // Target is within this block
            else {
                return Some(block.clone());
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_markdown_parser() -> anyhow::Result<()> {
        let code = [
            "# Title",
            "",
            "Hello, world!",
            "",
            "## Subtitle",
            "",
            "Lorem ipsum dolor sit amet, consectetur adipiscing elit.",
            "### Subtitle 2",
            "",
            "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.",
            "### Subtitle 3",
            "",
            "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do *eiusmod* tempor incididunt ut labore et dolore magna aliqua.",
            "",
            "> This is a blockquote",
            "> ",
            "> Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.",
            "",
        ];

        let code = code.join("\n");

        let mut parser = tree_sitter::Parser::new();
        parser.set_language(tree_sitter_md::language())?;

        let handle = super::Handle::new(&code, &mut parser)?;
        let blocks = handle.blocks.read().expect("Failed to read blocks");

        assert_eq!(blocks.len(), 10);
        assert_eq!(blocks[0].text, "Title");
        assert_eq!(blocks[5].stat.lines, 2);
        assert_eq!(blocks[5].stat.words, 19);
        assert_eq!(blocks[5].stat.avg_line_size, 9.5);

        println!("{:#?}", blocks);
        Ok(())
    }

    // #[tokio::test]
    // async fn test_get_block() -> anyhow::Result<()> {
    //     let code = [
    //         "# Title",
    //         "",
    //         "Hello, world!",
    //         "",
    //         "## Subtitle",
    //         "",
    //         "Lorem ipsum dolor sit amet, consectetur adipiscing elit.",
    //         "### Subtitle 2",
    //         "",
    //         "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.",
    //         "### Subtitle 3",
    //         "",
    //         "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do *eiusmod* tempor incididunt ut labore et dolore magna aliqua.",
    //         "",
    //         "> This is a blockquote",
    //         "> ",
    //         "> Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.",
    //         "",
    //     ];

    //     let code = code.join("\n");

    //     let mut parser = tree_sitter::Parser::new();
    //     parser.set_language(tree_sitter_md::language())?;

    //     let handle = super::Handle::new(&code, &mut parser)?;

    //     let block = handle.get_block(0, 0);

    //     assert!(block.is_some());

    //     Ok(())
    // }
}
