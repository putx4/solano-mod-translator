use super::gemini::GeminiProvider;
use super::ollama::OllamaProvider;
use super::openai::OpenAIProvider;
use super::opencode::OpenCodeProvider;
use super::traits::{TranslationContext, TranslationProvider};
use crate::error::{AppError, Result};
use crate::models::{AppConfig, ProviderConfig};
use std::collections::BTreeMap;
use std::sync::Arc;
use tracing::{info, warn};

pub struct ProviderManager {
    providers: Vec<(String, Arc<dyn TranslationProvider>)>,
}

impl ProviderManager {
    pub fn new() -> Self {
        let providers: Vec<(String, Arc<dyn TranslationProvider>)> = vec![
            ("gemini".into(), Arc::new(GeminiProvider::new())),
            ("openai".into(), Arc::new(OpenAIProvider::new())),
            ("grok".into(), Arc::new(OpenAIProvider::new())), // Grok uses OpenAI-compatible API
            ("claude".into(), Arc::new(OpenAIProvider::new())), // placeholder
            ("deepseek".into(), Arc::new(OpenAIProvider::new())), // placeholder
            ("ollama".into(), Arc::new(OllamaProvider::new())),
            ("opencode".into(), Arc::new(OpenCodeProvider::new())),
        ];
        Self { providers }
    }

    fn get_provider(&self, name: &str) -> Option<&Arc<dyn TranslationProvider>> {
        self.providers.iter().find(|(n, _)| n == name).map(|(_, p)| p)
    }

    pub async fn translate_with_fallback(
        &self,
        entries: &[(String, String)],
        source_locale: &str,
        target_locale: &str,
        context: &TranslationContext,
        config: &AppConfig,
    ) -> Result<(BTreeMap<String, String>, String)> {
        let mut last_error = None;

        for provider_name in &config.fallback_order {
            let provider_cfg = config.providers.iter().find(|p| &p.name == provider_name);
            let provider_cfg = match provider_cfg {
                Some(c) if c.enabled => c,
                _ => continue,
            };

            let provider = match self.get_provider(provider_name) {
                Some(p) => p,
                None => continue,
            };

            info!("🔄 Trying provider: {}", provider_name);
            match provider
                .translate_batch(entries, source_locale, target_locale, context, provider_cfg)
                .await
            {
                Ok(result) => {
                    info!("✅ {} succeeded", provider_name);
                    return Ok((result, provider_name.clone()));
                }
                Err(e) => {
                    warn!("❌ {} failed: {}", provider_name, e);
                    last_error = Some(e);
                    continue;
                }
            }
        }

        Err(last_error.unwrap_or_else(|| AppError::Other("No providers available".into())))
    }

    pub async fn test_provider(&self, name: &str, config: &ProviderConfig) -> Result<bool> {
        let provider = self.get_provider(name).ok_or_else(|| {
            AppError::NotFound(format!("Unknown provider: {}", name))
        })?;
        provider.test_connection(config).await
    }
}