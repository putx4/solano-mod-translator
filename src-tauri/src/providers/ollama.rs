use super::traits::{TranslationContext, TranslationProvider};
use crate::error::{AppError, Result};
use crate::models::ProviderConfig;
use reqwest::Client;
use serde_json::{json, Value};
use std::collections::BTreeMap;

pub struct OllamaProvider {
    client: Client,
}

impl OllamaProvider {
    pub fn new() -> Self {
        Self { client: Client::new() }
    }
}

#[async_trait::async_trait]
impl TranslationProvider for OllamaProvider {
    async fn translate_batch(
        &self,
        entries: &[(String, String)],
        source_locale: &str,
        target_locale: &str,
        _context: &TranslationContext,
        config: &ProviderConfig,
    ) -> Result<BTreeMap<String, String>> {
        let base = config.base_url.as_ref()
            .ok_or_else(|| AppError::Config("Ollama base_url not set".into()))?;
        let url = format!("{}/api/generate", base.trim_end_matches('/'));

        let mut numbered = Vec::new();
        for (i, (_, text)) in entries.iter().enumerate() {
            numbered.push(format!("{}: {}", i, text));
        }

        let prompt = format!(
            "Translate from {} to {}. Preserve all placeholders (%s, %1$s, {{player}}), format codes (§a, §l), numbers, commands. Return ONLY numbered lines.\n\n{}",
            source_locale, target_locale,
            numbered.join("\n")
        );

        let body = json!({
            "model": config.model,
            "prompt": prompt,
            "stream": false,
            "options": {
                "temperature": config.temperature as f64,
                "num_predict": config.max_tokens
            }
        });

        let resp = self.client.post(&url)
            .json(&body)
            .timeout(std::time::Duration::from_secs(config.timeout_secs as u64))
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(AppError::Provider {
                provider: "ollama".into(),
                message: format!("HTTP {}: {}", status, text),
            });
        }

        let v: Value = resp.json().await?;
        let text = v["response"].as_str().ok_or_else(|| AppError::Provider {
            provider: "ollama".into(),
            message: "No response field".into(),
        })?;

        let mut result = BTreeMap::new();
        for line in text.lines() {
            if let Some((num_str, tr)) = line.trim().split_once(':') {
                if let Ok(num) = num_str.trim().parse::<usize>() {
                    if num < entries.len() {
                        result.insert(entries[num].0.clone(), tr.trim().to_string());
                    }
                }
            }
        }
        Ok(result)
    }

    async fn test_connection(&self, config: &ProviderConfig) -> Result<bool> {
        let entries = vec![("test".into(), "Hello".into())];
        let ctx = TranslationContext {
            mod_id: "test".into(), mod_name: "Test".into(),
            category: None, glossary: vec![],
        };
        self.translate_batch(&entries, "en_us", "es_es", &ctx, config).await?;
        Ok(true)
    }
}