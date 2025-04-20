use std::{collections::HashSet, sync::Arc};

use dashmap::DashMap;

use rust_bert::pipelines::keywords_extraction::KeywordExtractionModel;

const SUMMARY_THRESHOLD: usize = 100;

#[derive(Clone)]
pub struct Registry {
    keyword_registry: Arc<DashMap<[u8; 32], Vec<String>>>, // blake3 hash
    reverse_index: Arc<DashMap<[u8; 32], super::Block>>,
    embedding_registry: Arc<DashMap<[u8; 32], Vec<f32>>>,
    summary_registry: Arc<DashMap<[u8; 32], String>>,
}

impl Registry {
    pub fn new() -> Self {
        Self {
            keyword_registry: Arc::new(DashMap::new()),
            reverse_index: Arc::new(DashMap::new()),
            embedding_registry: Arc::new(DashMap::new()),
            summary_registry: Arc::new(DashMap::new()),
        }
    }

    pub fn keyword_text(&self, content: &[super::Block]) -> anyhow::Result<()> {
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

    pub fn summarize_text(&self, content: &[super::Block]) -> anyhow::Result<()> {
        let summary_model =
            rust_bert::pipelines::summarization::SummarizationModel::new(Default::default())?;

        for block in content {
            self.reverse_index
                .insert(block.metadata.hash, block.clone());

            if self.summary_registry.contains_key(&block.metadata.hash)
                || block.stat.words < SUMMARY_THRESHOLD
            {
                continue;
            } else {
                let summary = summary_model.summarize(&[block.text.clone()])?;
                if let Some(summary) = summary.first() {
                    self.summary_registry
                        .insert(block.metadata.hash, summary.clone());
                }
            }
        }

        Ok(())
    }

    pub async fn embed_text(
        &self,
        content: &[super::Block],
        engine: &dyn crate::embedding::Embedding,
    ) -> anyhow::Result<()> {
        let (key_list, text_list): (Vec<_>, Vec<_>) = content
            .iter()
            .filter(|x| !self.embedding_registry.contains_key(&x.metadata.hash))
            .map(|data| (data.metadata.hash, data.text.clone()))
            .unzip();

        let embeddings = engine.embed_multiple(text_list).await?;

        for (key, embedding) in key_list.into_iter().zip(embeddings.into_iter()) {
            self.embedding_registry.insert(key, embedding);
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

        let garbage_keys = k_keys.difference(&true_keys).cloned();

        for key in garbage_keys {
            self.keyword_registry.remove(&key);
        }

        let e_keys = self
            .embedding_registry
            .iter()
            .map(|x| *x.key())
            .collect::<HashSet<_>>();

        let garbage_keys2 = e_keys.difference(&true_keys).cloned();

        for key in garbage_keys2 {
            self.embedding_registry.remove(&key);
        }

        let s_keys = self
            .summary_registry
            .iter()
            .map(|x| *x.key())
            .collect::<HashSet<_>>();

        let garbage_keys3 = s_keys.difference(&true_keys).cloned();

        for key in garbage_keys3 {
            self.summary_registry.remove(&key);
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

    pub fn get_summary(&self, content: &super::Block) -> Option<String> {
        if let Some(summary) = self.summary_registry.get(&content.metadata.hash) {
            let value = summary.clone();

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
