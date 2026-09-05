use crate::cache;
use crate::core::{jar_reader, lang_parser, validator};
use crate::core::lang_parser::LangMap;
use crate::error::{AppError, Result};
use crate::models::{AppConfig, GlossaryScope, JobStatus, TranslationEntry, TranslationJob};
use std::time::{Duration, Instant};
use crate::providers::manager::ProviderManager;
use crate::providers::traits::TranslationContext;
use parking_lot::Mutex;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::watch;
use tracing::{error, info, warn};

pub struct Translator {
    pub manager: Arc<ProviderManager>,
    pub current_job: Arc<Mutex<Option<TranslationJob>>>,
    pub cancel_txs: Arc<Mutex<Vec<watch::Sender<bool>>>>,
}

impl Translator {
    pub fn new() -> Self {
        Self {
            manager: Arc::new(ProviderManager::new()),
            current_job: Arc::new(Mutex::new(None)),
            cancel_txs: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Registra un canal de cancelación.
    pub fn register_cancel(&self, tx: watch::Sender<bool>) {
        self.cancel_txs.lock().push(tx);
    }

    /// Elimina un canal de cancelación (usando su igualdad por sender).
    pub fn unregister_cancel(&self, tx: &watch::Sender<bool>) {
        self.cancel_txs.lock().retain(|c| !std::ptr::eq(c, tx));
    }

    /// Cancela todos los trabajos activos.
    pub fn cancel_all(&self) {
        for tx in self.cancel_txs.lock().iter() {
            tx.send(true).ok();
        }
    }

    pub async fn translate_mod(
        &self,
        jar_path: &Path,
        output_path: &Path,
        mod_id: &str,
        mod_name: &str,
        source_locale: &str,
        target_locale: &str,
        config: &AppConfig,
        progress_tx: watch::Sender<TranslationJob>,
        cancel_rx: watch::Receiver<bool>,
    ) -> Result<TranslationJob> {
        info!("🚀 Starting translation: {} -> {}", mod_id, target_locale);

        let langs = jar_reader::read_all_langs(jar_path)?;

        let source_lang = langs.iter().find(|(path, _)| {
            path.contains(&format!("/{}.json", source_locale))
                || path.contains(&format!("/{}.lang", source_locale))
        });

        let (source_path, (_, source_format, source_map)) = match source_lang {
            Some((p, v)) => (p.clone(), v.clone()),
            None => return Err(AppError::NotFound(format!(
                "Source lang {} not found in {}", source_locale, mod_id
            ))),
        };

        let target_path = source_path
            .replace(&format!("/{}.json", source_locale), &format!("/{}.json", target_locale))
            .replace(&format!("/{}.lang", source_locale), &format!("/{}.lang", target_locale));

        let existing_target: LangMap = langs
            .get(&target_path)
            .map(|(_, _, m)| m.clone())
            .unwrap_or_default();

        let total = source_map.len();
        let mut job = TranslationJob {
            id: uuid::Uuid::new_v4().to_string(),
            mod_id: mod_id.to_string(),
            total_strings: total,
            translated: 0,
            cached: 0,
            failed: 0,
            status: JobStatus::Translating,
            started_at: chrono::Utc::now().to_rfc3339(),
            current_key: None,
        };
        *self.current_job.lock() = Some(job.clone());
        let _ = progress_tx.send(job.clone());

        let mut to_translate: Vec<(String, String)> = Vec::new();
        // Mapa de fuente -> lista de reemplazos de variables protegidas
        let mut protections: std::collections::HashMap<String, Vec<(String, String)>> =
            std::collections::HashMap::new();
        let mut final_map: LangMap = existing_target.clone();

        for (key, source_text) in &source_map {
            if *cancel_rx.borrow() {
                job.status = JobStatus::Error;
                return Err(AppError::Cancelled);
            }

            if existing_target.contains_key(key) {
                final_map.insert(key.clone(), existing_target[key].clone());
                job.translated += 1;
                continue;
            }

            if config.enable_cache {
                if let Ok(Some(cached)) = cache::lookup_by_text(source_text, source_locale, target_locale) {
                    if !cached.is_manual_edit || final_map.get(key).is_none() {
                        final_map.insert(key.clone(), cached.target_text.clone());
                        job.cached += 1;
                        job.translated += 1;
                        continue;
                    }
                }
            }

            // Proteger variables/placeholders antes de enviarlas a la IA
            let (protected_text, replacements) = validator::protect_variables(source_text);
            protections.insert(protected_text.clone(), replacements);
            to_translate.push((key.clone(), protected_text));
        }

        info!("📊 {}: total={}, cached={}, to_translate={}",
              mod_id, total, job.cached, to_translate.len());

        let batch_size = config.global_batch_size as usize;
        let mut provider_used = String::new();
        let started = Instant::now();

        // Cargar el glosario desde el caché (global + específico del mod + locale)
        let glossary: Vec<(String, String)> = cache::get_glossary()
            .unwrap_or_default()
            .into_iter()
            .filter(|g| {
                // Solo aplicar si coincide el par de idiomas
                (g.source_locale.eq_ignore_ascii_case(source_locale)
                    && g.target_locale.eq_ignore_ascii_case(target_locale))
                    // O un glosario sin idiomas definidos (genérico)
                    || (g.source_locale.is_empty() && g.target_locale.is_empty())
            })
            .filter(|g| {
                match g.scope {
                    GlossaryScope::Global => true,
                    GlossaryScope::Mod => g.mod_id.as_deref() == Some(mod_id),
                    GlossaryScope::Locale => true,
                }
            })
            .map(|g| (g.source, g.target))
            .collect();

        for chunk in to_translate.chunks(batch_size) {
            if *cancel_rx.borrow() {
                job.status = JobStatus::Error;
                return Err(AppError::Cancelled);
            }

            let context = TranslationContext {
                mod_id: mod_id.to_string(),
                mod_name: mod_name.to_string(),
                category: chunk.first().map(|(k, _)| lang_parser::categorize_key(k).to_string()),
                glossary: glossary.clone(),
            };

            // Reintentar el lote con backoff exponencial si falla
            let mut attempt = 0;
            let max_attempts = config.max_retries.max(1) as usize + 1;
            let mut batch_result = None;
            while attempt < max_attempts {
                if *cancel_rx.borrow() {
                    job.status = JobStatus::Error;
                    return Err(AppError::Cancelled);
                }
                match self.manager
                    .translate_with_fallback(chunk, source_locale, target_locale, &context, config)
                    .await
                {
                    Ok(res) => { batch_result = Some(res); break; }
                    Err(e) => {
                        attempt += 1;
                        if attempt >= max_attempts {
                            error!("❌ Batch failed after {} attempts: {}", max_attempts, e);
                            break;
                        }
                        warn!("⚠️ Batch attempt {} failed: {}. Retrying...", attempt, e);
                        // Backoff exponencial: 2s, 4s, 8s, ...
                        let delay = Duration::from_secs(2u64.pow(attempt as u32));
                        tokio::time::sleep(delay).await;
                    }
                }
            }

            let (translations, provider) = match batch_result {
                Some(res) => res,
                None => {
                    job.failed += chunk.len();
                    job.current_key = chunk.last().map(|(k, _)| k.clone());
                    let _ = progress_tx.send(job.clone());
                    *self.current_job.lock() = Some(job.clone());
                    continue;
                }
            };

            provider_used = provider.clone();
            for (key, protected_src) in chunk {
                if let Some(translated) = translations.get(key) {
                    // Restaurar las variables protegidas en la traducción
                    let restored = protections
                        .get(protected_src)
                        .map(|repl| validator::restore_variables(translated, repl))
                        .unwrap_or_else(|| translated.clone());
                    let original_source = source_map
                        .get(key)
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| protected_src.clone());

                    if config.enable_validation {
                        let validation = validator::validate(&original_source, &restored)?;
                        if !validation.valid && config.reject_suspicious {
                            warn!("⚠️ Rejected translation for {}: {:?}", key, validation.errors);
                            job.failed += 1;
                            continue;
                        }
                    }

                    final_map.insert(key.clone(), restored.clone());

                    let entry = TranslationEntry {
                        id: None,
                        mod_id: mod_id.to_string(),
                        key: key.clone(),
                        source_text: original_source,
                        target_text: restored,
                        source_locale: source_locale.to_string(),
                        target_locale: target_locale.to_string(),
                        provider: provider.clone(),
                        confidence: 1.0,
                        is_manual_edit: false,
                        category: Some(lang_parser::categorize_key(key).to_string()),
                        created_at: chrono::Utc::now().to_rfc3339(),
                        updated_at: chrono::Utc::now().to_rfc3339(),
                    };
                    if config.enable_cache {
                        cache::upsert_translation(&entry).ok();
                    }

                    job.translated += 1;
                } else {
                    job.failed += 1;
                }
            }

            job.current_key = chunk.last().map(|(k, _)| k.clone());
            let _ = progress_tx.send(job.clone());
            *self.current_job.lock() = Some(job.clone());
        }

        jar_reader::write_translated_jar(jar_path, output_path, &target_path, &final_map, source_format)?;

        job.status = JobStatus::Completed;
        let _ = progress_tx.send(job.clone());
        *self.current_job.lock() = Some(job.clone());

        let duration = started.elapsed().as_secs() as i64;
        cache::record_history(
            mod_id,
            total as i64,
            job.translated as i64,
            job.cached as i64,
            job.failed as i64,
            &provider_used,
            duration,
        ).ok();

        info!("✅ {} completed: {}/{} translated, {} cached, {} failed",
              mod_id, job.translated, total, job.cached, job.failed);

        Ok(job)
    }

    pub fn get_current_job(&self) -> Option<TranslationJob> {
        self.current_job.lock().clone()
    }
}