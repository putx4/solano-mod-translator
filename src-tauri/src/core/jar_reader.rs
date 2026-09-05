use crate::core::lang_parser::{self, LangMap};
use crate::error::Result;
use crate::models::LangFormat;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::Path;
use tracing::info;

/// Read all lang files from a jar, returning {path -> (locale, format, map)}
pub fn read_all_langs(jar_path: &Path) -> Result<HashMap<String, (String, LangFormat, LangMap)>> {
    let file = std::fs::File::open(jar_path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    let mut result = HashMap::new();

    let names: Vec<String> = (0..archive.len())
        .filter_map(|i| archive.by_index(i).ok().map(|f| f.name().to_string()))
        .collect();

    for name in names {
        let (locale, format) = match parse_lang_path(&name) {
            Some(v) => v,
            None => continue,
        };

        let mut file = archive.by_name(&name)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        let map = lang_parser::parse_lang(&content, format)?;
        result.insert(name, (locale, format, map));
    }

    Ok(result)
}

/// Write a translated jar: copy original jar, replacing/adding target lang file
pub fn write_translated_jar(
    source_jar: &Path,
    output_jar: &Path,
    lang_path: &str,
    new_map: &LangMap,
    format: LangFormat,
) -> Result<()> {
    let source_file = std::fs::File::open(source_jar)?;
    let mut source_archive = zip::ZipArchive::new(source_file)?;

    let out_file = std::fs::File::create(output_jar)?;
    let mut out_archive = zip::ZipWriter::new(out_file);

    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    let new_content = lang_parser::serialize_lang(new_map, format)?;
    let mut written_target = false;

    for i in 0..source_archive.len() {
        let mut entry = source_archive.by_index(i)?;
        let name = entry.name().to_string();

        if name == lang_path {
            // Replace with translated version
            out_archive.start_file(&name, options)?;
            out_archive.write_all(new_content.as_bytes())?;
            written_target = true;
        } else {
            // Copy as-is
            out_archive.start_file(&name, options)?;
            std::io::copy(&mut entry, &mut out_archive)?;
        }
    }

    if !written_target {
        // Add new lang file
        out_archive.start_file(lang_path, options)?;
        out_archive.write_all(new_content.as_bytes())?;
    }

    out_archive.finish()?;
    info!("✅ Wrote translated jar: {:?}", output_jar);
    Ok(())
}

pub fn parse_lang_path_public(path: &str) -> Option<(String, LangFormat)> {
    parse_lang_path(path)
}

fn parse_lang_path(path: &str) -> Option<(String, LangFormat)> {
    static RE_JSON: once_cell::sync::Lazy<regex::Regex> = once_cell::sync::Lazy::new(|| {
        regex::Regex::new(r"assets/[^/]+/lang/([a-z]{2}_[a-z]{2})\.json$").unwrap()
    });
    static RE_LEGACY: once_cell::sync::Lazy<regex::Regex> = once_cell::sync::Lazy::new(|| {
        regex::Regex::new(r"^lang/([a-z]{2}_[a-z]{2})\.lang$").unwrap()
    });

    if let Some(caps) = RE_JSON.captures(path) {
        return Some((caps[1].to_string(), LangFormat::Json));
    }
    if let Some(caps) = RE_LEGACY.captures(path) {
        return Some((caps[1].to_string(), LangFormat::Legacy));
    }
    None
}