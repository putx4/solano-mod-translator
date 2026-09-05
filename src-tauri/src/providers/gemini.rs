use super::traits::{TranslationContext, TranslationProvider};
use crate::error::{AppError, Result};
use crate::models::ProviderConfig;
use reqwest::Client;
use serde_json::{json, Value};
use std::collections::BTreeMap;

pub struct GeminiProvider {
    client: Client,
}

impl GeminiProvider {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

#[async_trait::async_trait]
impl TranslationProvider for GeminiProvider {
    async fn translate_batch(
        &self,
        entries: &[(String, String)],
        source_locale: &str,
        target_locale: &str,
        context: &TranslationContext,
        config: &ProviderConfig,
    ) -> Result<BTreeMap<String, String>> {
        let api_key = config.api_key.as_ref().ok_or_else(|| {
            AppError::Config("Gemini API key not set".to_string())
        })?;

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            config.model, api_key
        );

        let mut numbered = Vec::new();
        for (i, (_, text)) in entries.iter().enumerate() {
            numbered.push(format!("{}: {}", i, text));
        }
        let numbered_text = numbered.join("\n");

        let glossary_hint = if context.glossary.is_empty() {
            String::new()
        } else {
            let g: Vec<String> = context
                .glossary
                .iter()
                .map(|(s, t)| format!("{} -> {}", s, t))
                .collect();
            format!("\n\nGlossary (MUST follow):\n{}", g.join("\n"))
        };

        let category_hint = context
            .category
            .as_ref()
            .map(|c| format!("Category: {} (Minecraft context)\n", c))
            .unwrap_or_default();

        let prompt = format!(
            "You are a professional translator for Minecraft mods.\n\
Translate the following strings from {} to {}.\n\n\
Rules (CRITICAL):\n\
- Preserve EXACTLY all placeholders: %s, %d, %1$s, {{player}}, {{count}}, etc.\n\
- Preserve EXACTLY all Minecraft format codes: §a, §l, §r, etc.\n\
- Preserve EXACTLY all numbers (100 must stay 100).\n\
- Preserve EXACTLY all commands and identifiers: /give, @p, minecraft:diamond.\n\
- Do NOT add any information not in the source.\n\
- Keep the same numbered format in output.\n\
{}\n\
{}\n\
Input:\n\
{}\n\n\
Output (same numbered format, ONLY the translations, no commentary):",
            source_locale, target_locale, category_hint, glossary_hint, numbered_text
        );

        let body = json!({
            "contents": [{"parts": [{"text": prompt}]}],
            "generationConfig": {
                "temperature": config.temperature,
                "maxOutputTokens": config.max_tokens
            }
        });

        let resp = self
            .client
            .post(&url)
            .json(&body)
            .timeout(std::time::Duration::from_secs(config.timeout_secs as u64))
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(AppError::Provider {
                provider: "gemini".to_string(),
                message: format!("HTTP {}: {}", status, text),
            });
        }

        let v: Value = resp.json().await?;
        let text = v["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .ok_or_else(|| AppError::Provider {
                provider: "gemini".to_string(),
                message: "No text in response".to_string(),
            })?;

        parse_numbered_response(text, entries)
    }

    async fn test_connection(&self, config: &ProviderConfig) -> Result<bool> {
        let entries = vec![("test.key".to_string(), "Hello".to_string())];
        let ctx = TranslationContext {
            mod_id: "test".to_string(),
            mod_name: "Test".to_string(),
            category: None,
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
            provider: "gemini".to_string(),
            message: format!(
                "Expected {} translations, got {}. Raw: {}",
                entries.len(),
                result.len(),
                text
            ),
        });
    }
    
    Ok(result)
}