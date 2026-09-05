#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod cache;
mod commands;
mod config;
mod core;
mod error;
mod glossary;
mod models;
mod providers;
mod translator;

use tauri::Manager;
use tracing_subscriber::{fmt, EnvFilter};

fn main() {
    // Logger
    fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("solano=info".parse().unwrap()))
        .init();

    tracing::info!("🧩 Solano Mod Translator starting...");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_process::init())
        .plugin(
            tauri_plugin_sql::Builder::new()
                .add_migrations("sqlite:translations.db", cache::get_migrations())
                .build(),
        )
        .setup(|app| {
            // Init database
            let app_data = app.path().app_data_dir().expect("app data dir");
            std::fs::create_dir_all(&app_data).ok();
            cache::init(&app_data.join("translations.db"))?;
            config::init(&app_data.join("config.json"))?;
            tracing::info!("✅ Database and config initialized");

            // Lanzar opencode en segundo plano si no está corriendo (no bloquea el arranque)
            tauri::async_runtime::spawn(async move {
                match commands::ensure_opencode().await {
                    Ok(true) => tracing::info!("✅ opencode listo en :4096"),
                    Ok(false) => tracing::warn!("⚠️ opencode no respondió a tiempo"),
                    Err(e) => tracing::warn!("⚠️ No se pudo iniciar opencode: {}", e),
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
    commands::scan_mods_folder,
    commands::get_mod_details,
    commands::translate_mod,
    commands::cancel_translation,
    commands::get_translations_for_mod,
    commands::update_translation,
    commands::create_backup,
    commands::list_backups,
    commands::restore_backup,
    commands::get_config,
    commands::save_config,
    commands::test_provider,
    commands::get_stats,
    commands::get_history,
    commands::get_glossary,
    commands::add_glossary_entry,
    commands::delete_glossary_entry,
    commands::diagnose_mod,
    commands::repair_mod,
    commands::get_translation_progress,
    commands::ensure_opencode,
])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}