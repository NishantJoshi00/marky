use std::{collections::HashSet, sync::Arc};

use dashmap::DashMap;

use rust_bert::pipelines::keywords_extraction::KeywordExtractionModel;

#[derive(Clone)]
pub struct Registry {
    keyword_registry: Arc<DashMap<[u8; 32], Vec<String>>>, // blake3 hash
    reverse_index: Arc<DashMap<[u8; 32], super::Block>>,
}

impl Registry {
    pub fn new() -> Self {
        Self {
            keyword_registry: Arc::new(DashMap::new()),
            reverse_index: Arc::new(DashMap::new()),
        }
    }

    pub fn index_text(&self, content: &[super::Block]) -> anyhow::Result<()> {
        let model = KeywordExtractionModel::new(Default::default())?;

        for block in content {
            self.reverse_index
                .insert(block.metadata.hash, block.clone());
            if self.keyword_registry.contains_key(&block.metadata.hash) {
                continue;
            } else {
                self.keyword_registry
                    .insert(block.metadata.hash, self.generate_keywords(block, &model)?);
            }
        }

        Ok(())
    }

    // WARNING: This is an optimization to perform garbage collection, please reconsider this
    // when we start indexing multiple files
    //
    // The approach assumes that everything that exists in the project, will be present in the
    // reverse index, this assumption is used to remove redundant entries in the keyword registry.
    pub fn garbage_collect(&self) -> anyhow::Result<()> {
        let true_keys = self
            .reverse_index
            .iter()
            .map(|x| *x.key())
            .collect::<HashSet<_>>();
        let k_keys = self
            .keyword_registry
            .iter()
            .map(|x| *x.key())
            .collect::<HashSet<_>>();

        let garbage_keys = k_keys.difference(&true_keys).cloned().collect::<Vec<_>>();

        for key in garbage_keys {
            self.keyword_registry.remove(&key);
        }

        Ok(())
    }

    fn generate_keywords(
        &self,
        content: &super::Block,
        model: &KeywordExtractionModel<'_>,
    ) -> anyhow::Result<Vec<String>> {
        let text = content.text.clone();
        let output = model
            .predict(&[text])?
            .first()
            .ok_or_else(|| anyhow::anyhow!("No keywords found"))?
            .iter()
            .filter(|x| x.score > 0.4)
            .map(|x| x.text.clone())
            .collect();
        Ok(output)
    }

    pub fn get_keywords(&self, content: &super::Block) -> Option<Vec<String>> {
        if let Some(keywords) = self.keyword_registry.get(&content.metadata.hash) {
            let value = keywords.clone();

            if value.is_empty() {
                return None;
            }

            return Some(value);
        }
        None
    }
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}
