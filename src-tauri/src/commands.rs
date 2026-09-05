use crate::cache;
use crate::config;
use crate::core::{backup, scanner};
use crate::error::{AppError, Result};
use crate::models::*;
use crate::providers::manager::ProviderManager;
use crate::translator::Translator;
use once_cell::sync::OnceCell;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::watch;
use tracing::{error, info};

static TRANSLATOR: OnceCell<Arc<Translator>> = OnceCell::new();

fn translator() -> &'static Arc<Translator> {
    TRANSLATOR.get_or_init(|| Arc::new(Translator::new()))
}

#[derive(Clone, serde::Serialize)]
pub struct ProgressEvent {
    pub job_id: String,
    pub mod_id: String,
    pub mod_name: String,
    pub total: usize,
    pub translated: usize,
    pub cached: usize,
    pub failed: usize,
    pub status: String,
    pub current_key: Option<String>,
    pub percent: f32,
}

#[tauri::command]
pub async fn scan_mods_folder(folder: String) -> Result<Vec<ModInfo>> {
    scanner::scan_folder(&PathBuf::from(folder))
}

#[tauri::command]
pub async fn get_mod_details(jar_path: String) -> Result<ModInfo> {
    scanner::read_mod_info(&PathBuf::from(jar_path))
}

#[tauri::command]
pub async fn translate_mod(
    app: AppHandle,
    jar_path: String,
    output_folder: String,
    mod_id: String,
    mod_name: String,
    source_locale: Option<String>,
    target_locale: Option<String>,
) -> Result<String> {
    let cfg = config::get();
    let t = translator().clone();

    // Permitir elegir idiomas por mod; si no se pasa, usar la config global
    let src_locale = source_locale.unwrap_or_else(|| cfg.source_locale.clone());
    let tgt_locale = target_locale.unwrap_or_else(|| cfg.target_locale.clone());

    let jar = PathBuf::from(&jar_path);
    let out_dir = PathBuf::from(&output_folder);
    std::fs::create_dir_all(&out_dir).map_err(|e| AppError::Io(e))?;

    let output_path = out_dir.join(jar.file_name().ok_or_else(|| {
        AppError::Other("Invalid jar path".into())
    })?);

    // Backup
    if cfg.enable_backup {
        let backup_root = out_dir.join("backup");
        backup::create_backup(&jar, &backup_root)?;
        info!("💾 Backup created");
    }

    let job_id = uuid::Uuid::new_v4().to_string();

    // Emitir evento de inicio
    let start_event = ProgressEvent {
        job_id: job_id.clone(),
        mod_id: mod_id.clone(),
        mod_name: mod_name.clone(),
        total: 0,
        translated: 0,
        cached: 0,
        failed: 0,
        status: "starting".to_string(),
        current_key: None,
        percent: 0.0,
    };
    let _ = app.emit("translation-progress", &start_event);

    // Lanzar la traducción en un task separado para no bloquear
    let app_clone = app.clone();
    let jar_clone = jar.clone();
    let output_clone = output_path.clone();
    let mod_id_clone = mod_id.clone();
    let mod_name_clone = mod_name.clone();
    let job_id_clone = job_id.clone();

    tokio::spawn(async move {
        let (progress_tx, mut progress_rx) = watch::channel(TranslationJob {
            id: job_id_clone.clone(),
            mod_id: mod_id_clone.clone(),
            total_strings: 0,
            translated: 0,
            cached: 0,
            failed: 0,
            status: JobStatus::Translating,
            started_at: chrono::Utc::now().to_rfc3339(),
            current_key: None,
        });

        let (cancel_tx, cancel_rx) = watch::channel(false);
        let cancel_tx_ref = cancel_tx.clone();
        {
            t.register_cancel(cancel_tx.clone());
        }

        // Task para emitir eventos de progreso cada vez que cambia
        let app_event = app_clone.clone();
        let job_id_event = job_id_clone.clone();
        let mod_id_event = mod_id_clone.clone();
        let mod_name_event = mod_name_clone.clone();
        
        tokio::spawn(async move {
            while progress_rx.changed().await.is_ok() {
                let job = progress_rx.borrow().clone();
                let percent = if job.total_strings > 0 {
                    (job.translated as f32 / job.total_strings as f32) * 100.0
                } else {
                    0.0
                };

                let event = ProgressEvent {
                    job_id: job_id_event.clone(),
                    mod_id: mod_id_event.clone(),
                    mod_name: mod_name_event.clone(),
                    total: job.total_strings,
                    translated: job.translated,
                    cached: job.cached,
                    failed: job.failed,
                    status: format!("{:?}", job.status).to_lowercase(),
                    current_key: job.current_key.clone(),
                    percent,
                };
                let _ = app_event.emit("translation-progress", &event);
            }
        });

        // Ejecutar la traducción real
        let result = t
            .translate_mod(
                &jar_clone,
                &output_clone,
                &mod_id_clone,
                &mod_name_clone,
                &src_locale,
                &tgt_locale,
                &cfg,
                progress_tx,
                cancel_rx,
            )
            .await;

        // Limpiar el canal de cancelación de este trabajo
        t.unregister_cancel(&cancel_tx_ref);

        match result {
            Ok(job) => {
                info!("✅ Translation completed: {}", mod_id_clone);
                let final_event = ProgressEvent {
                    job_id: job_id_clone.clone(),
                    mod_id: mod_id_clone.clone(),
                    mod_name: mod_name_clone.clone(),
                    total: job.total_strings,
                    translated: job.translated,
                    cached: job.cached,
                    failed: job.failed,
                    status: "completed".to_string(),
                    current_key: None,
                    percent: 100.0,
                };
                let _ = app_clone.emit("translation-progress", &final_event);
                let _ = app_clone.emit("translation-complete", &final_event);
            }
            Err(e) => {
                error!("❌ Translation failed: {}", e);
                let error_event = ProgressEvent {
                    job_id: job_id_clone.clone(),
                    mod_id: mod_id_clone.clone(),
                    mod_name: mod_name_clone.clone(),
                    total: 0,
                    translated: 0,
                    cached: 0,
                    failed: 0,
                    status: format!("error: {}", e),
                    current_key: None,
                    percent: 0.0,
                };
                let _ = app_clone.emit("translation-progress", &error_event);
                let _ = app_clone.emit("translation-error", &error_event);
            }
        }
    });

    Ok(job_id)
}

#[tauri::command]
pub async fn cancel_translation() -> Result<()> {
    translator().cancel_all();
    Ok(())
}

#[tauri::command]
pub async fn get_translations_for_mod(mod_id: String) -> Result<Vec<TranslationEntry>> {
    cache::get_translations_for_mod(&mod_id)
}

#[tauri::command]
pub async fn update_translation(entry: TranslationEntry) -> Result<()> {
    let mut e = entry;
    e.is_manual_edit = true;
    e.updated_at = chrono::Utc::now().to_rfc3339();
    cache::upsert_translation(&e)
}

#[tauri::command]
pub async fn create_backup(jar_path: String) -> Result<String> {
    let backup_root = PathBuf::from(&jar_path).parent()
        .map(|p| p.join("backup"))
        .unwrap_or_else(|| PathBuf::from("backup"));
    let path = backup::create_backup(&PathBuf::from(jar_path), &backup_root)?;
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn list_backups(backup_root: String) -> Result<Vec<backup::BackupInfo>> {
    backup::list_backups(&PathBuf::from(backup_root))
}

#[tauri::command]
pub async fn restore_backup(backup_dir: String, target_folder: String) -> Result<()> {
    backup::restore_backup(&PathBuf::from(backup_dir), &PathBuf::from(target_folder))
}

#[tauri::command]
pub async fn get_config() -> Result<AppConfig> {
    Ok(config::get())
}

#[tauri::command]
pub async fn save_config(cfg: AppConfig) -> Result<()> {
    config::update(cfg)
}

#[tauri::command]
pub async fn test_provider(name: String, cfg: ProviderConfig) -> Result<bool> {
    let manager = ProviderManager::new();
    manager.test_provider(&name, &cfg).await
}

#[tauri::command]
pub async fn get_stats() -> Result<Stats> {
    cache::get_stats()
}

#[tauri::command]
pub async fn get_history() -> Result<Vec<HistoryEntry>> {
    cache::get_history(200)
}

#[tauri::command]
pub async fn get_glossary() -> Result<Vec<GlossaryEntry>> {
    cache::get_glossary()
}

#[tauri::command]
pub async fn add_glossary_entry(entry: GlossaryEntry) -> Result<i64> {
    let mut e = entry;
    if e.created_at.is_empty() {
        e.created_at = chrono::Utc::now().to_rfc3339();
    }
    cache::add_glossary_entry(&e)
}

#[tauri::command]
pub async fn delete_glossary_entry(id: i64) -> Result<()> {
    cache::delete_glossary_entry(id)
}

#[tauri::command]
pub async fn diagnose_mod(jar_path: String) -> Result<DiagnosticReport> {
    crate::core::diagnostics::diagnose(&PathBuf::from(jar_path))
}

#[tauri::command]
pub async fn repair_mod(jar_path: String) -> Result<()> {
    crate::core::diagnostics::repair(&PathBuf::from(jar_path))
}

#[tauri::command]
pub async fn get_translation_progress() -> Result<Option<TranslationJob>> {
    Ok(translator().get_current_job())
}

/// Verifica si opencode sirve en :4096 y, si no, lo lanza en segundo plano.
/// Devuelve true si opencode está disponible tras la espera.
#[tauri::command]
pub async fn ensure_opencode() -> Result<bool> {
    const HEALTH_URL: &str = "http://127.0.0.1:4096/global/health";

    async fn is_up() -> bool {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(3))
            .build();
        match client {
            Ok(c) => match c.get(HEALTH_URL).send().await {
                Ok(r) => r.status().is_success(),
                Err(_) => false,
            },
            Err(_) => false,
        }
    }

    if is_up().await {
        return Ok(true);
    }

    info!("📡 opencode no está corriendo. Lanzándolo...");

    // Localizar opencode ejecutable
    let exe = std::env::var("OPENCODE_EXE")
        .ok()
        .filter(|p| std::path::Path::new(p).exists())
        .unwrap_or_else(|| "opencode".to_string());

    // Lanzar opencode serve como proceso separado con su propia consola
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NEW_CONSOLE: u32 = 0x0000_0010;
        let _ = std::process::Command::new(&exe)
            .args(["serve"])
            .creation_flags(CREATE_NEW_CONSOLE)
            .spawn();
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = std::process::Command::new(&exe).args(["serve"]).spawn();
    }

    // Esperar hasta ~45s a que arranque
    for _ in 0..30 {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        if is_up().await {
            return Ok(true);
        }
    }

    info!("⚠️ opencode no respondió a tiempo");
    Ok(false)
}