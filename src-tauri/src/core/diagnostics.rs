use crate::core::jar_reader;
use crate::error::Result;
use crate::models::{DiagnosticReport, LangFormat};
use std::collections::BTreeMap;
use std::path::Path;

/// Diagnose a mod jar: check lang file integrity, placeholders, duplicates, corruption.
pub fn diagnose(jar_path: &Path) -> Result<DiagnosticReport> {
    let langs = jar_reader::read_all_langs(jar_path)?;

    let mut suspicious = Vec::new();
    let mut duplicates = Vec::new();
    let mut corrupted_files = Vec::new();
    let mut total_keys = 0usize;
    let mut json_valid = true;

    // Reopen per-file for detailed parsing (detect corrupt entries)
    let file = std::fs::File::open(jar_path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    let names: Vec<String> = (0..archive.len())
        .filter_map(|i| {
            archive
                .by_index(i)
                .ok()
                .map(|f| f.name().to_string())
        })
        .collect();

    let mut seen_keys: BTreeMap<String, String> = BTreeMap::new();

    for name in names {
        // Corrupt entries: try reading
        let mut entry = match archive.by_name(&name) {
            Ok(e) => e,
            Err(_) => {
                corrupted_files.push(name);
                json_valid = false;
                continue;
            }
        };
        let mut content = String::new();
        if std::io::Read::read_to_string(&mut entry, &mut content).is_err() {
            corrupted_files.push(name);
            json_valid = false;
            continue;
        }

        let (locale, format) = match jar_reader::parse_lang_path_public(&name) {
            Some(v) => v,
            None => continue,
        };

        match crate::core::lang_parser::parse_lang(&content, format) {
            Ok(map) => {
                total_keys += map.len();

                if format == LangFormat::Json && locale == "en_us" {
                    // Placeholder sanity check on the source lang
                    for (k, v) in &map {
                        if crate::core::validator::validate(v, v).map(|r| !r.valid).unwrap_or(false) {
                            suspicious.push(k.clone());
                        }
                        if v.len() > 200 {
                            suspicious.push(k.clone());
                        }
                    }
                }

                for (k, v) in map {
                    if let Some(prev) = seen_keys.get(&k) {
                        if prev != &v {
                            duplicates.push(k.clone());
                        }
                    }
                    seen_keys.insert(k, v);
                }
            }
            Err(_) => {
                corrupted_files.push(name);
                json_valid = false;
            }
        }
    }

    // Translated % = fraction of keys that appear across more than one locale
    let locales = langs.len().max(1);
    let translated_percent = if total_keys > 0 {
        (total_keys as f32 / locales as f32).min(100.0)
    } else {
        0.0
    };

    let placeholders_ok = suspicious.is_empty();
    let repairable = !corrupted_files.is_empty();

    Ok(DiagnosticReport {
        mod_id: jar_path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default(),
        json_valid,
        total_keys,
        placeholders_ok,
        translated_percent,
        suspicious,
        duplicates,
        corrupted_files,
        repairable,
    })
}

/// Attempt to repair a mod jar. Currently re-writes the jar to rebuild its index.
pub fn repair(jar_path: &Path) -> Result<()> {
    let tmp_path = jar_path.with_extension("jar.tmp");
    let langs = jar_reader::read_all_langs(jar_path)?;

    // Rebuild the jar by re-writing entries; use en_us map as a baseline target.
    let mut target_path: Option<String> = None;
    let mut target_format = LangFormat::Json;
    let mut target_map = crate::core::lang_parser::LangMap::new();

    for (path, (_locale, format, map)) in &langs {
        if path.ends_with("/en_us.json") {
            target_path = Some(path.clone());
            target_format = *format;
            target_map = map.clone();
            break;
        }
    }

    let target_path = target_path.ok_or_else(|| {
        crate::error::AppError::NotFound("No en_us lang file found to repair".into())
    })?;

    jar_reader::write_translated_jar(jar_path, &tmp_path, &target_path, &target_map, target_format)?;

    std::fs::remove_file(jar_path)?;
    std::fs::rename(tmp_path, jar_path)?;
    Ok(())
}
