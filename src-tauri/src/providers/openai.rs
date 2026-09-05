use super::traits::{TranslationContext, TranslationProvider};
use crate::error::{AppError, Result};
use crate::models::ProviderConfig;
use reqwest::Client;
use serde_json::{json, Value};
use std::collections::BTreeMap;

pub struct OpenAIProvider {
    client: Client,
}

impl OpenAIProvider {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    fn base_url(config: &ProviderConfig) -> String {
        config
            .base_url
            .clone()
            .unwrap_or_else(|| "https://api.openai.com/v1".to_string())
    }
}

#[async_trait::async_trait]
impl TranslationProvider for OpenAIProvider {
    async fn translate_batch(
        &self,
        entries: &[(String, String)],
        source_locale: &str,
        target_locale: &str,
        context: &TranslationContext,
        config: &ProviderConfig,
    ) -> Result<BTreeMap<String, String>> {
        let api_key = config.api_key.as_ref().ok_or_else(|| {
            AppError::Config("OpenAI API key not set".to_string())
        })?;

        let url = format!("{}/chat/completions", Self::base_url(config));

        let mut numbered = Vec::new();
        for (i, (_, text)) in entries.iter().enumerate() {
            numbered.push(format!("{}: {}", i, text));
        }

        let category = context
            .category
            .clone()
            .unwrap_or_else(|| "generic".to_string());

        let glossary_text = if context.glossary.is_empty() {
            "No glossary provided.".to_string()
        } else {
            context
                .glossary
                .iter()
                .map(|(source, target)| format!("{} -> {}", source, target))
                .collect::<Vec<_>>()
                .join("\n")
        };

        let system = format!(
            "You are a professional translator for Minecraft mods.\n\
Translate from {} to {}.\n\
Minecraft mod: {} ({})\n\
Category: {}\n\n\
Rules:\n\
- Preserve EXACTLY all placeholders: %s, %d, %1$s, {{player}}, {{count}}, etc.\n\
- Preserve EXACTLY all Minecraft formatting codes: §a, §l, §r, etc.\n\
- Preserve EXACTLY all numbers.\n\
- Preserve EXACTLY all commands and identifiers: /give, @p, minecraft:diamond.\n\
- Do not translate technical identifiers.\n\
- Do not invent information.\n\
- Return ONLY numbered lines using this format: 0: translation\n\n\
Glossary:\n{}",
            source_locale,
            target_locale,
            context.mod_name,
            context.mod_id,
            category,
            glossary_text
        );

        let user = numbered.join("\n");

        let body = json!({
            "model": config.model,
            "messages": [
                {
                    "role": "system",
                    "content": system
                },
                {
                    "role": "user",
                    "content": user
                }
            ],
            "temperature": config.temperature,
            "max_tokens": config.max_tokens
        });

        let resp = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&body)
            .timeout(std::time::Duration::from_secs(config.timeout_secs as u64))
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();

            return Err(AppError::Provider {
                provider: "openai".to_string(),
                message: format!("HTTP {}: {}", status, text),
            });
        }

        let v: Value = resp.json().await?;

        let text = v["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| AppError::Provider {
                provider: "openai".to_string(),
                message: "No content in OpenAI-compatible response".to_string(),
            })?;

        parse_numbered_response(text, entries, "openai")
    }

    async fn test_connection(&self, config: &ProviderConfig) -> Result<bool> {
        let entries = vec![("test.key".to_string(), "Hello".to_string())];

        let ctx = TranslationContext {
            mod_id: "test".to_string(),
            mod_name: "Test Mod".to_string(),
            category: Some("generic".to_string()),
            glossary: vec![],
        };

        self.translate_batch(&entries, "en_us", "es_es", &ctx, config)
            .await?;

        Ok(true)
    }
}

fn parse_numbered_response(
    text: &str,
    entries: &[(String, String)],
    provider: &str,
) -> Result<BTreeMap<String, String>> {
    let mut result = BTreeMap::new();

    for line in text.lines() {
        let line = line.trim();

        if line.is_empty() {
            continue;
        }

        if let Some((num_str, translation)) = line.split_once(':') {
            if let Ok(num) = num_str.trim().parse::<usize>() {
                if num < entries.len() {
                    result.insert(entries[num].0.clone(), translation.trim().to_string());
                }
            }
        }
    }

    if result.len() != entries.len() {
        return Err(AppError::Provider {
            provider: provider.to_string(),
            message: format!(
                "Expected {} translations, got {}. Raw response: {}",
                entries.len(),
                result.len(),
                text
            ),
        });
    }

    Ok(result)
}