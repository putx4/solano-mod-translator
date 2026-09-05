use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("ZIP error: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("SQL error: {0}")]
    Sql(#[from] rusqlite::Error),

    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("Provider error: {provider} - {message}")]
    Provider { provider: String, message: String },

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Config error: {0}")]
    Config(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Cancelled")]
    Cancelled,

    #[error("{0}")]
    Other(String),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl From<AppError> for tauri::Error {
    fn from(e: AppError) -> Self {
        tauri::Error::Anyhow(e.into())
    }
}

pub type Result<T> = std::result::Result<T, AppError>;