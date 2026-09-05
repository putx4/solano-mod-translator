use crate::error::Result;
use chrono::Local;
use std::path::{Path, PathBuf};
use tracing::info;

pub fn create_backup(jar_path: &Path, backup_root: &Path) -> Result<PathBuf> {
    let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S");
    let backup_dir = backup_root.join(timestamp.to_string());
    std::fs::create_dir_all(&backup_dir)?;

    let filename = jar_path.file_name().ok_or_else(|| {
        crate::error::AppError::Other("Invalid jar path".into())
    })?;
    let dest = backup_dir.join(filename);
    std::fs::copy(jar_path, &dest)?;

    info!("💾 Backup created: {:?}", dest);
    Ok(backup_dir)
}

pub fn list_backups(backup_root: &Path) -> Result<Vec<BackupInfo>> {
    let mut backups = Vec::new();
    if !backup_root.exists() {
        return Ok(backups);
    }
    for entry in std::fs::read_dir(backup_root)? {
        let entry = entry?;
        if entry.path().is_dir() {
            let files: Vec<String> = std::fs::read_dir(entry.path())?
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("jar"))
                .map(|e| e.file_name().to_string_lossy().to_string())
                .collect();
            backups.push(BackupInfo {
                timestamp: entry.file_name().to_string_lossy().to_string(),
                path: entry.path().to_string_lossy().to_string(),
                files,
            });
        }
    }
    backups.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    Ok(backups)
}

pub fn restore_backup(backup_dir: &Path, target_folder: &Path) -> Result<()> {
    for entry in std::fs::read_dir(backup_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("jar") {
            let dest = target_folder.join(entry.file_name());
            std::fs::copy(&path, &dest)?;
            info!("↩️ Restored: {:?}", dest);
        }
    }
    Ok(())
}

#[derive(serde::Serialize)]
pub struct BackupInfo {
    pub timestamp: String,
    pub path: String,
    pub files: Vec<String>,
}