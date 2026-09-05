use super::traits::{TranslationContext, TranslationProvider};
use crate::error::{AppError, Result};
use crate::models::ProviderConfig;
use reqwest::Client;
use serde_json::{json, Value};
use std::collections::BTreeMap;

pub struct OpenCodeProvider {
    client: Client,
}

impl OpenCodeProvider {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    fn base_url(config: &ProviderConfig) -> String {
        config
            .base_url
            .clone()
            .unwrap_or_else(|| "http://127.0.0.1:4096".to_string())
    }

    fn timeout(config: &ProviderConfig) -> std::time::Duration {
        // opencode forward the request to a real model and waits; give generous room
        std::time::Duration::from_secs(config.timeout_secs.max(120) as u64)
    }

    async fn create_session(
        &self,
        base: &str,
        title: &str,
        timeout: std::time::Duration,
    ) -> Result<String> {
        let url = format!("{}/session", base);
        let resp = self
            .client
            .post(&url)
            .json(&json!({ "title": title }))
            .timeout(timeout)
            .send()
            .await
            .map_err(|e| AppError::Provider {
                provider: "opencode".into(),
                message: format!(
                    "Could not reach opencode server at {} (is `opencode serve` running?). Error: {}",
                    base, e
                ),
            })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(AppError::Provider {
                provider: "opencode".into(),
                message: format!("Failed to create session. HTTP {}: {}", status, text),
            });
        }

        let v: Value = resp.json().await?;
        v["id"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| AppError::Provider {
                provider: "opencode".into(),
                message: "Create session response missing id".into(),
            })
    }

    async fn send_message(
        &self,
        base: &str,
        session_id: &str,
        prompt: &str,
        timeout: std::time::Duration,
    ) -> Result<String> {
        let url = format!("{}/session/{}/message", base, session_id);
        let body = json!({
            "parts": [{ "type": "text", "text": prompt }]
        });

        let resp = self
            .client
            .post(&url)
            .json(&body)
            .timeout(timeout)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(AppError::Provider {
                provider: "opencode".into(),
                message: format!("Message failed. HTTP {}: {}", status, text),
            });
        }

        let v: Value = resp.json().await?;

        let mut collected = String::new();
        if let Some(parts) = v["parts"].as_array() {
            for part in parts {
                let is_text = part["type"]
                    .as_str()
                    .map(|t| t.eq_ignore_ascii_case("text"))
                    .unwrap_or(false);
                if is_text {
                    if let Some(t) = part["text"].as_str() {
                        collected.push_str(t);
                        collected.push('\n');
                    }
                }
            }
        }

        let recovered = collected.trim();
        if recovered.is_empty() {
            // Fallback: try the info.error or any text field
            if let Some(err) = v["info"]["error"].as_str() {
                return Err(AppError::Provider {
                    provider: "opencode".into(),
                    message: format!("opencode returned an error: {}", err),
                });
            }
            return Err(AppError::Provider {
                provider: "opencode".into(),
                message: "No text in opencode response".into(),
            });
        }

        Ok(recovered.to_string())
    }

    async fn delete_session(&self, base: &str, session_id: &str) {
        let url = format!("{}/session/{}", base, session_id);
        let _ = self.client.delete(&url).send().await;
    }
}

#[async_trait::async_trait]
impl TranslationProvider for OpenCodeProvider {
    async fn translate_batch(
        &self,
        entries: &[(String, String)],
        source_locale: &str,
        target_locale: &str,
        context: &TranslationContext,
        config: &ProviderConfig,
    ) -> Result<BTreeMap<String, String>> {
        let base = Self::base_url(config);

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

        let prompt = format!(
            "You are a professional translator for Minecraft mods.\n\
Translate from {} to {}.\n\
Minecraft mod: {} ({})\n\
Category: {}\n\n\
Rules (CRITICAL):\n\
- Preserve EXACTLY all placeholders: %s, %d, %1$s, {{player}}, {{count}}, etc.\n\
- Preserve EXACTLY all Minecraft formatting codes: §a, §l, §r, etc.\n\
- Preserve EXACTLY all numbers.\n\
- Preserve EXACTLY all commands and identifiers: /give, @p, minecraft:diamond.\n\
- Do not translate technical identifiers.\n\
- Do not invent information.\n\
- Do NOT use any tools, do NOT run commands, do NOT edit files. Reply with text only.\n\n\
Glossary (MUST follow):\n{}\n\n\
Input:\n{}\n\n\
Output ONLY numbered lines in this exact format, no commentary:\n\
0: translation\n1: translation\n...",
            source_locale, target_locale, context.mod_name, context.mod_id, category, glossary_text, numbered.join("\n")
        );

        let session_id = self
            .create_session(&base, "mod-translation", Self::timeout(config))
            .await?;

        let result = self
            .send_message(&base, &session_id, &prompt, Self::timeout(config))
            .await;

        self.delete_session(&base, &session_id).await;

        let text = result?;
        parse_numbered_response(&text, entries, "opencode")
    }

    async fn test_connection(&self, config: &ProviderConfig) -> Result<bool> {
        let entries = vec![("test.key".to_string(), "Hello world".to_string())];
        let ctx = TranslationContext {
            mod_id: "test".into(),
            mod_name: "Test".into(),
            category: Some("generic".into()),
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
