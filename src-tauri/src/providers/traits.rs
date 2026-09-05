use async_trait::async_trait;
use crate::error::Result;
use crate::models::ProviderConfig;
use std::collections::BTreeMap;

#[async_trait]
pub trait TranslationProvider: Send + Sync {
    async fn translate_batch(
        &self,
        entries: &[(String, String)], // (key, source_text)
        source_locale: &str,
        target_locale: &str,
        context: &TranslationContext,
        config: &ProviderConfig,
    ) -> Result<BTreeMap<String, String>>;

    async fn test_connection(&self, config: &ProviderConfig) -> Result<bool>;
}

#[derive(Debug, Clone)]
pub struct TranslationContext {
    pub mod_id: String,
    pub mod_name: String,
    pub category: Option<String>,
    pub glossary: Vec<(String, String)>,
}