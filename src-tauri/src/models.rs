use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub mc_version: String,
    pub loader: Loader,
    pub path: String,
    pub sha1: String,
    pub size_bytes: u64,
    pub lang_files: Vec<LangFile>,
    pub dependencies: Vec<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Loader {
    Forge,
    NeoForge,
    Fabric,
    Quilt,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LangFile {
    pub path: String,
    pub locale: String,
    pub format: LangFormat,
    pub keys_count: usize,
    pub chars_count: usize,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LangFormat {
    Json,
    Legacy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationEntry {
    pub id: Option<i64>,
    pub mod_id: String,
    pub key: String,
    pub source_text: String,
    pub target_text: String,
    pub source_locale: String,
    pub target_locale: String,
    pub provider: String,
    pub confidence: f32,
    pub is_manual_edit: bool,
    pub category: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationJob {
    pub id: String,
    pub mod_id: String,
    pub total_strings: usize,
    pub translated: usize,
    pub cached: usize,
    pub failed: usize,
    pub status: JobStatus,
    pub started_at: String,
    pub current_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    Waiting,
    Scanning,
    Translating,
    Validating,
    Completed,
    Error,
    Skipped,
    Paused,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub name: String,
    pub api_key: Option<String>,
    pub model: String,
    pub base_url: Option<String>,
    pub enabled: bool,
    pub priority: i32,
    pub temperature: f32,
    pub max_tokens: i32,
    pub batch_size: i32,
    pub timeout_secs: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub providers: Vec<ProviderConfig>,
    pub fallback_order: Vec<String>,
    pub source_locale: String,
    pub target_locale: String,
    pub workers: i32,
    pub global_batch_size: i32,
    pub enable_backup: bool,
    pub enable_validation: bool,
    pub enable_cache: bool,
    pub reject_suspicious: bool,
    pub max_retries: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub placeholders_preserved: bool,
    pub numbers_preserved: bool,
    pub commands_preserved: bool,
    pub formatting_preserved: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticReport {
    pub mod_id: String,
    pub json_valid: bool,
    pub total_keys: usize,
    pub placeholders_ok: bool,
    pub translated_percent: f32,
    pub suspicious: Vec<String>,
    pub duplicates: Vec<String>,
    pub corrupted_files: Vec<String>,
    pub repairable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlossaryEntry {
    pub id: Option<i64>,
    pub source: String,
    pub target: String,
    pub scope: GlossaryScope,
    pub mod_id: Option<String>,
    pub source_locale: String,
    pub target_locale: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum GlossaryScope {
    Global,
    Mod,
    Locale,
}

impl GlossaryScope {
    pub fn from_str(s: &str) -> Self {
        match s {
            "mod" => GlossaryScope::Mod,
            "locale" => GlossaryScope::Locale,
            _ => GlossaryScope::Global,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            GlossaryScope::Global => "global".into(),
            GlossaryScope::Mod => "mod".into(),
            GlossaryScope::Locale => "locale".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: i64,
    pub mod_id: String,
    pub total_strings: i64,
    pub translated: i64,
    pub cached: i64,
    pub failed: i64,
    pub provider: String,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub duration_secs: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stats {
    pub total_translated: i64,
    pub total_ai_requests: i64,
    pub cache_hit_percent: f64,
    pub money_saved: f64,
    pub total_time_secs: i64,
    pub errors: i64,
    pub mods_processed: i64,
}