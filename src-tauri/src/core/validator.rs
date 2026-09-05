use crate::error::Result;
use crate::models::ValidationResult;
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashSet;

static RE_FMT: Lazy<Regex> = Lazy::new(|| Regex::new(r"%(\d+\$)?[sdfixX]").unwrap());
static RE_BRACE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\{[a-zA-Z0-9_.]+\}").unwrap());
static RE_FORMAT_CODE: Lazy<Regex> = Lazy::new(|| Regex::new(r"§[0-9a-fk-or]").unwrap());
static RE_NUMBER: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b\d+(\.\d+)?\b").unwrap());
static RE_COMMAND: Lazy<Regex> = Lazy::new(|| Regex::new(r"/[a-z_]+|@[a-zpr]|minecraft:[a-z_]+").unwrap());

/// Extract all placeholders from a string
fn extract_placeholders(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    // %s, %d, %1$s, %2$d, etc.
    for m in RE_FMT.find_iter(s) {
        out.push(m.as_str().to_string());
    }
    // {placeholder}
    for m in RE_BRACE.find_iter(s) {
        out.push(m.as_str().to_string());
    }
    out
}

fn extract_format_codes(s: &str) -> Vec<String> {
    RE_FORMAT_CODE.find_iter(s).map(|m| m.as_str().to_string()).collect()
}

fn extract_numbers(s: &str) -> Vec<String> {
    RE_NUMBER.find_iter(s).map(|m| m.as_str().to_string()).collect()
}

fn extract_commands(s: &str) -> Vec<String> {
    RE_COMMAND.find_iter(s).map(|m| m.as_str().to_string()).collect()
}

pub fn validate(source: &str, translation: &str) -> Result<ValidationResult> {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // Placeholders
    let src_ph: HashSet<_> = extract_placeholders(source).into_iter().collect();
    let tr_ph: HashSet<_> = extract_placeholders(translation).into_iter().collect();
    let placeholders_preserved = src_ph == tr_ph;
    if !placeholders_preserved {
        let missing: Vec<_> = src_ph.difference(&tr_ph).cloned().collect();
        let extra: Vec<_> = tr_ph.difference(&src_ph).cloned().collect();
        if !missing.is_empty() {
            errors.push(format!("Missing placeholders: {:?}", missing));
        }
        if !extra.is_empty() {
            errors.push(format!("Extra placeholders: {:?}", extra));
        }
    }

    // Format codes (§)
    let src_fmt: HashSet<_> = extract_format_codes(source).into_iter().collect();
    let tr_fmt: HashSet<_> = extract_format_codes(translation).into_iter().collect();
    let formatting_preserved = src_fmt == tr_fmt;
    if !formatting_preserved {
        warnings.push("Format codes (§) differ between source and translation".into());
    }

    // Numbers
    let src_nums = extract_numbers(source);
    let tr_nums = extract_numbers(translation);
    let numbers_preserved = src_nums == tr_nums;
    if !numbers_preserved {
        errors.push(format!(
            "Numbers changed: {:?} -> {:?}",
            src_nums, tr_nums
        ));
    }

    // Commands
    let src_cmds: HashSet<_> = extract_commands(source).into_iter().collect();
    let tr_cmds: HashSet<_> = extract_commands(translation).into_iter().collect();
    let commands_preserved = src_cmds.is_subset(&tr_cmds) || src_cmds == tr_cmds;
    if !commands_preserved {
        errors.push("Commands or identifiers were modified".into());
    }

    // Length sanity (translation shouldn't be 5x longer)
    if translation.len() > source.len() * 5 && source.len() > 10 {
        warnings.push("Translation is suspiciously long".into());
    }

    let valid = errors.is_empty();
    Ok(ValidationResult {
        valid,
        errors,
        warnings,
        placeholders_preserved,
        numbers_preserved,
        commands_preserved,
        formatting_preserved,
    })
}

/// Protect variables before sending to IA, restore after
pub fn protect_variables(text: &str) -> (String, Vec<(String, String)>) {
    let mut protected = text.to_string();
    let mut replacements = Vec::new();

    let patterns = [
        (&*RE_FMT, "FMT"),
        (&*RE_BRACE, "BRACE"),
        (&*RE_FORMAT_CODE, "FMT_CODE"),
    ];

    let mut counter = 0;
    for (regex, prefix) in patterns {
        let matches: Vec<String> = regex.find_iter(&protected).map(|m| m.as_str().to_string()).collect();
        for m in matches {
            let token = format!("__{}_{}__", prefix, counter);
            counter += 1;
            protected = protected.replacen(&m, &token, 1);
            replacements.push((token, m));
        }
    }

    (protected, replacements)
}

pub fn restore_variables(text: &str, replacements: &[(String, String)]) -> String {
    let mut out = text.to_string();
    for (token, original) in replacements {
        out = out.replace(token, original);
    }
    out
}