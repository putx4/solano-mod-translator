use crate::error::Result;
use crate::models::LangFormat;
use std::collections::BTreeMap;

pub type LangMap = BTreeMap<String, String>;

pub fn parse_lang(content: &str, format: LangFormat) -> Result<LangMap> {
    match format {
        LangFormat::Json => parse_json(content),
        LangFormat::Legacy => parse_legacy(content),
    }
}

fn parse_json(content: &str) -> Result<LangMap> {
    let v: serde_json::Value = serde_json::from_str(content)?;
    let mut map = LangMap::new();
    if let Some(obj) = v.as_object() {
        for (k, v) in obj {
            if let Some(s) = v.as_str() {
                map.insert(k.clone(), s.to_string());
            }
        }
    }
    Ok(map)
}

fn parse_legacy(content: &str) -> Result<LangMap> {
    let mut map = LangMap::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            map.insert(k.trim().to_string(), v.trim().to_string());
        }
    }
    Ok(map)
}

pub fn serialize_lang(map: &LangMap, format: LangFormat) -> Result<String> {
    match format {
        LangFormat::Json => {
            let v: serde_json::Value = serde_json::to_value(map)?;
            Ok(serde_json::to_string_pretty(&v)?)
        }
        LangFormat::Legacy => {
            let mut out = String::new();
            for (k, v) in map {
                out.push_str(&format!("{}={}\n", k, v.replace('\n', "\\n")));
            }
            Ok(out)
        }
    }
}

/// Categorize a key into a Minecraft category for context-aware translation
pub fn categorize_key(key: &str) -> &'static str {
    if key.starts_with("block.") {
        "block"
    } else if key.starts_with("item.") {
        "item"
    } else if key.starts_with("entity.") {
        "entity"
    } else if key.starts_with("enchantment.") {
        "enchantment"
    } else if key.starts_with("effect.") {
        "effect"
    } else if key.starts_with("advancements.") || key.contains(".advancement.") {
        "advancement"
    } else if key.starts_with("death.") || key.contains(".death.") {
        "death_message"
    } else if key.contains(".tooltip") || key.ends_with(".tooltip") {
        "tooltip"
    } else if key.contains("gui.") || key.contains(".screen.") {
        "screen"
    } else if key.contains("chat.") || key.contains(".message.") {
        "chat"
    } else if key.contains("command.") || key.starts_with("commands.") {
        "command"
    } else if key.contains(".recipe.") {
        "recipe"
    } else if key.contains(".description") || key.contains(".lore") {
        "lore"
    } else if key.contains(".button") || key.contains(".btn") {
        "button"
    } else {
        "generic"
    }
}