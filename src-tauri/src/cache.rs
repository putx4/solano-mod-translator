use crate::error::Result;
use crate::models::{GlossaryEntry, TranslationEntry};
use once_cell::sync::OnceCell;
use parking_lot::Mutex;
use rusqlite::{params, Connection};
use std::path::Path;
use std::sync::Arc;

static DB: OnceCell<Arc<Mutex<Connection>>> = OnceCell::new();

pub fn get_migrations() -> Vec<tauri_plugin_sql::Migration> {
    vec![]
}

pub fn init(path: &Path) -> Result<()> {
    let conn = Connection::open(path)?;
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS translations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            mod_id TEXT NOT NULL,
            key TEXT NOT NULL,
            source_text TEXT NOT NULL,
            target_text TEXT NOT NULL,
            source_locale TEXT NOT NULL,
            target_locale TEXT NOT NULL,
            provider TEXT NOT NULL,
            confidence REAL NOT NULL DEFAULT 1.0,
            is_manual_edit INTEGER NOT NULL DEFAULT 0,
            category TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            UNIQUE(mod_id, key, source_locale, target_locale)
        );

        CREATE INDEX IF NOT EXISTS idx_source_text ON translations(source_text, source_locale, target_locale);
        CREATE INDEX IF NOT EXISTS idx_mod ON translations(mod_id);

        CREATE TABLE IF NOT EXISTS glossary (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            source TEXT NOT NULL,
            target TEXT NOT NULL,
            scope TEXT NOT NULL,
            mod_id TEXT,
            source_locale TEXT NOT NULL,
            target_locale TEXT NOT NULL,
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS run_history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            mod_id TEXT NOT NULL,
            total_strings INTEGER NOT NULL,
            translated INTEGER NOT NULL,
            cached INTEGER NOT NULL,
            failed INTEGER NOT NULL,
            provider TEXT NOT NULL,
            started_at TEXT NOT NULL,
            finished_at TEXT,
            duration_secs INTEGER NOT NULL DEFAULT 0
        );
        "#,
    )?;
    DB.set(Arc::new(Mutex::new(conn)))
        .map_err(|_| crate::error::AppError::Config("DB already initialized".into()))?;
    Ok(())
}

fn with_db<F, T>(f: F) -> Result<T>
where
    F: FnOnce(&Connection) -> Result<T>,
{
    let db = DB.get().ok_or_else(|| crate::error::AppError::Config("DB not initialized".into()))?;
    let lock = db.lock();
    f(&lock)
}

/// Look up a translation by source text (cross-mod reuse)
pub fn lookup_by_text(
    source_text: &str,
    source_locale: &str,
    target_locale: &str,
) -> Result<Option<TranslationEntry>> {
    with_db(|conn| {
        let mut stmt = conn.prepare(
            "SELECT id, mod_id, key, source_text, target_text, source_locale, target_locale,
                    provider, confidence, is_manual_edit, category, created_at, updated_at
             FROM translations
             WHERE source_text = ?1 AND source_locale = ?2 AND target_locale = ?3
             LIMIT 1",
        )?;
        let mut rows = stmt.query_map(params![source_text, source_locale, target_locale], |row| {
            Ok(TranslationEntry {
                id: row.get(0)?,
                mod_id: row.get(1)?,
                key: row.get(2)?,
                source_text: row.get(3)?,
                target_text: row.get(4)?,
                source_locale: row.get(5)?,
                target_locale: row.get(6)?,
                provider: row.get(7)?,
                confidence: row.get(8)?,
                is_manual_edit: row.get::<_, i32>(9)? != 0,
                category: row.get(10)?,
                created_at: row.get(11)?,
                updated_at: row.get(12)?,
            })
        })?;
        match rows.next() {
            Some(Ok(entry)) => Ok(Some(entry)),
            Some(Err(e)) => Err(e.into()),
            None => Ok(None),
        }
    })
}

pub fn upsert_translation(entry: &TranslationEntry) -> Result<()> {
    with_db(|conn| {
        conn.execute(
            "INSERT INTO translations (mod_id, key, source_text, target_text, source_locale,
             target_locale, provider, confidence, is_manual_edit, category, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
             ON CONFLICT(mod_id, key, source_locale, target_locale) DO UPDATE SET
                target_text = CASE WHEN is_manual_edit = 1 THEN target_text ELSE ?4 END,
                provider = ?7,
                confidence = ?8,
                updated_at = ?12",
            params![
                entry.mod_id,
                entry.key,
                entry.source_text,
                entry.target_text,
                entry.source_locale,
                entry.target_locale,
                entry.provider,
                entry.confidence,
                entry.is_manual_edit as i32,
                entry.category,
                entry.created_at,
                entry.updated_at,
            ],
        )?;
        Ok(())
    })
}

pub fn get_translations_for_mod(mod_id: &str) -> Result<Vec<TranslationEntry>> {
    with_db(|conn| {
        let mut stmt = conn.prepare(
            "SELECT id, mod_id, key, source_text, target_text, source_locale, target_locale,
                    provider, confidence, is_manual_edit, category, created_at, updated_at
             FROM translations WHERE mod_id = ?1 ORDER BY key",
        )?;
        let rows = stmt.query_map(params![mod_id], |row| {
            Ok(TranslationEntry {
                id: row.get(0)?,
                mod_id: row.get(1)?,
                key: row.get(2)?,
                source_text: row.get(3)?,
                target_text: row.get(4)?,
                source_locale: row.get(5)?,
                target_locale: row.get(6)?,
                provider: row.get(7)?,
                confidence: row.get(8)?,
                is_manual_edit: row.get::<_, i32>(9)? != 0,
                category: row.get(10)?,
                created_at: row.get(11)?,
                updated_at: row.get(12)?,
            })
        })?;
        rows.collect::<std::result::Result<Vec<_>, _>>().map_err(|e| e.into())
    })
}

pub fn record_history(
    mod_id: &str,
    total: i64,
    translated: i64,
    cached: i64,
    failed: i64,
    provider: &str,
    duration_secs: i64,
) -> Result<()> {
    with_db(|conn| {
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO run_history (mod_id, total_strings, translated, cached, failed, provider, started_at, duration_secs)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![mod_id, total, translated, cached, failed, provider, now, duration_secs],
        )?;
        Ok(())
    })
}

pub fn get_history(limit: i64) -> Result<Vec<crate::models::HistoryEntry>> {
    with_db(|conn| {
        let mut stmt = conn.prepare(
            "SELECT id, mod_id, total_strings, translated, cached, failed, provider, started_at, finished_at, duration_secs
             FROM run_history
             ORDER BY started_at DESC
             LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit], |row| {
            Ok(crate::models::HistoryEntry {
                id: row.get(0)?,
                mod_id: row.get(1)?,
                total_strings: row.get(2)?,
                translated: row.get(3)?,
                cached: row.get(4)?,
                failed: row.get(5)?,
                provider: row.get(6)?,
                started_at: row.get(7)?,
                finished_at: row.get(8)?,
                duration_secs: row.get(9)?,
            })
        })?;
        rows.collect::<std::result::Result<Vec<_>, _>>().map_err(|e| e.into())
    })
}

pub fn get_glossary() -> Result<Vec<GlossaryEntry>> {
    with_db(|conn| {
        let mut stmt = conn.prepare(
            "SELECT id, source, target, scope, mod_id, source_locale, target_locale, created_at
             FROM glossary
             ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(GlossaryEntry {
                id: row.get(0)?,
                source: row.get(1)?,
                target: row.get(2)?,
                scope: crate::models::GlossaryScope::from_str(&row.get::<_, String>(3)?),
                mod_id: row.get(4)?,
                source_locale: row.get(5)?,
                target_locale: row.get(6)?,
                created_at: row.get(7)?,
            })
        })?;
        rows.collect::<std::result::Result<Vec<_>, _>>().map_err(|e| e.into())
    })
}

pub fn add_glossary_entry(entry: &GlossaryEntry) -> Result<i64> {
    with_db(|conn| {
        conn.execute(
            "INSERT INTO glossary (source, target, scope, mod_id, source_locale, target_locale, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                entry.source,
                entry.target,
                entry.scope.to_string(),
                entry.mod_id,
                entry.source_locale,
                entry.target_locale,
                entry.created_at,
            ],
        )?;
        Ok(conn.last_insert_rowid())
    })
}

pub fn delete_glossary_entry(id: i64) -> Result<()> {
    with_db(|conn| {
        conn.execute("DELETE FROM glossary WHERE id = ?1", params![id])?;
        Ok(())
    })
}

pub fn get_stats() -> Result<crate::models::Stats> {
    with_db(|conn| {
        let total_translated: i64 = conn.query_row(
            "SELECT COUNT(*) FROM translations", [], |r| r.get(0),
        )?;
        let ai_requests: i64 = conn.query_row(
            "SELECT COUNT(*) FROM translations WHERE is_manual_edit = 0", [], |r| r.get(0),
        )?;
        let mods_processed: i64 = conn.query_row(
            "SELECT COUNT(DISTINCT mod_id) FROM translations", [], |r| r.get(0),
        )?;
        let errors: i64 = conn.query_row(
            "SELECT COALESCE(SUM(failed), 0) FROM run_history", [], |r| r.get(0),
        )?;

        let cached_total: i64 = conn.query_row(
            "SELECT COALESCE(SUM(cached), 0) FROM run_history", [], |r| r.get(0),
        )?;
        let string_total: i64 = conn.query_row(
            "SELECT COALESCE(SUM(total_strings), 0) FROM run_history", [], |r| r.get(0),
        )?;
        let cache_hit_percent = if string_total > 0 {
            (cached_total as f64 / string_total as f64) * 100.0
        } else {
            0.0
        };

        // Tiempo total de trabajo en segundos (suma de duraciones registradas)
        let total_time_secs: i64 = conn.query_row(
            "SELECT COALESCE(SUM(duration_secs), 0) FROM run_history", [], |r| r.get(0),
        )?;

        // Dinero ahorrado estimado: cada string reutilizado del caché
        // equivale a una llamada a la IA que NO se hizo. Costo estimado
        // promedio por llamada de traducción (conservador).
        const COST_PER_AI_CALL: f64 = 0.002; // $0.002 por string traducido por IA
        let money_saved = cached_total as f64 * COST_PER_AI_CALL;

        Ok(crate::models::Stats {
            total_translated,
            total_ai_requests: ai_requests,
            cache_hit_percent,
            money_saved,
            total_time_secs,
            errors,
            mods_processed,
        })
    })
}