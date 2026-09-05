use crate::error::Result;
use crate::models::{AppConfig, ProviderConfig};
use once_cell::sync::OnceCell;
use parking_lot::Mutex;
use std::path::{Path, PathBuf};
use std::sync::Arc;

static CONFIG: OnceCell<Arc<Mutex<AppConfig>>> = OnceCell::new();
static CONFIG_PATH: OnceCell<PathBuf> = OnceCell::new();

const KEYRING_SERVICE: &str = "com.solano.translator";
const KEYRING_PREFIX: &str = "api_key";

fn save_key(provider: &str, key: &str) {
    let entry = keyring::Entry::new(KEYRING_SERVICE, &format!("{}_{}", KEYRING_PREFIX, provider));
    if let Ok(entry) = entry {
        let _ = entry.set_password(key);
    }
}

fn load_key(provider: &str) -> Option<String> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, &format!("{}_{}", KEYRING_PREFIX, provider))
        .ok()?;
    entry.get_password().ok()
}

fn delete_key(provider: &str) {
    let entry = keyring::Entry::new(KEYRING_SERVICE, &format!("{}_{}", KEYRING_PREFIX, provider));
    if let Ok(entry) = entry {
        let _ = entry.delete_credential();
    }
}

/// Versión del config SIN claves de API para persistir en disco.
fn without_keys(cfg: &AppConfig) -> AppConfig {
    let mut c = cfg.clone();
    for p in &mut c.providers {
        p.api_key = None;
    }
    c
}

/// Añade el proveedor "opencode" (local, vía opencode serve) a configs existentes
/// para que quede disponible al actualizar la aplicación.
fn ensure_opencode(cfg: &mut AppConfig) {
    let has = cfg.providers.iter().any(|p| p.name == "opencode");
    if !has {
        cfg.providers.insert(
            0,
            ProviderConfig {
                name: "opencode".into(),
                api_key: None,
                model: String::new(),
                base_url: Some("http://127.0.0.1:4096".into()),
                enabled: true,
                priority: 0,
                temperature: 0.3,
                max_tokens: 4096,
                batch_size: 50,
                timeout_secs: 120,
            },
        );
        cfg.fallback_order.retain(|n| n != "opencode");
        cfg.fallback_order.insert(0, "opencode".into());
    }
}

pub fn init(path: &Path) -> Result<()> {
    let mut config = if path.exists() {
        let content = std::fs::read_to_string(path)?;
        serde_json::from_str(&content)?
    } else {
        default_config()
    };

    ensure_opencode(&mut config);

    // Mover/recargar las API keys desde el almacén seguro del sistema
    for p in &mut config.providers {
        if let Some(k) = p.api_key.as_deref() {
            if !k.is_empty() {
                // Migración: clave venía en texto plano en disco -> mover a keyring
                save_key(&p.name, k);
            }
        }
        if p.api_key.as_deref().unwrap_or("").is_empty() {
            p.api_key = load_key(&p.name);
        }
    }

    // Persistir sin las keys reales
    let public = without_keys(&config);
    save(path, &public)?;

    CONFIG.set(Arc::new(Mutex::new(config)))
        .map_err(|_| crate::error::AppError::Config("Config already initialized".into()))?;
    CONFIG_PATH.set(path.to_path_buf())
        .map_err(|_| crate::error::AppError::Config("Config path already initialized".into()))?;
    Ok(())
}

pub fn get() -> AppConfig {
    CONFIG.get().expect("Config not initialized").lock().clone()
}

pub fn save_to_disk() -> Result<()> {
    let config = get();
    let path = CONFIG_PATH.get().expect("Config path not initialized");

    // Persistir las keys en el almacén seguro antes de escribir a disco
    for p in &config.providers {
        match p.api_key.as_deref() {
            Some(k) if !k.is_empty() => save_key(&p.name, k),
            _ => delete_key(&p.name),
        }
    }

    let public = without_keys(&config);
    save(path, &public)
}

fn save(path: &Path, config: &AppConfig) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(config)?;
    std::fs::write(path, content)?;
    Ok(())
}

pub fn update(config: AppConfig) -> Result<()> {
    if let Some(c) = CONFIG.get() {
        *c.lock() = config.clone();
    }
    save_to_disk()?;
    Ok(())
}

fn default_config() -> AppConfig {
    AppConfig {
        providers: vec![
            ProviderConfig {
                name: "gemini".into(),
                api_key: None,
                model: "gemini-2.0-flash".into(),
                base_url: None,
                enabled: true,
                priority: 1,
                temperature: 0.3,
                max_tokens: 4096,
                batch_size: 50,
                timeout_secs: 60,
            },
            ProviderConfig {
                name: "openai".into(),
                api_key: None,
                model: "gpt-4o-mini".into(),
                base_url: None,
                enabled: false,
                priority: 2,
                temperature: 0.3,
                max_tokens: 4096,
                batch_size: 50,
                timeout_secs: 60,
            },
            ProviderConfig {
                name: "ollama".into(),
                api_key: None,
                model: "llama3.2".into(),
                base_url: Some("http://localhost:11434".into()),
                enabled: false,
                priority: 3,
                temperature: 0.3,
                max_tokens: 4096,
                batch_size: 30,
                timeout_secs: 120,
            },
            ProviderConfig {
                name: "opencode".into(),
                api_key: None,
                model: String::new(),
                base_url: Some("http://127.0.0.1:4096".into()),
                enabled: true,
                priority: 0,
                temperature: 0.3,
                max_tokens: 4096,
                batch_size: 50,
                timeout_secs: 120,
            },
        ],
        fallback_order: vec![
            "opencode".into(),
            "gemini".into(),
            "openai".into(),
            "ollama".into(),
        ],
        source_locale: "en_us".into(),
        target_locale: "es_es".into(),
        workers: 4,
        global_batch_size: 50,
        enable_backup: true,
        enable_validation: true,
        enable_cache: true,
        reject_suspicious: true,
        max_retries: 3,
    }
}