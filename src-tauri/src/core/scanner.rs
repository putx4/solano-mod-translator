use crate::core::jar_reader;
use crate::error::Result;
use crate::models::{LangFile, Loader, ModInfo};
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

pub fn scan_folder(folder: &Path) -> Result<Vec<ModInfo>> {
    info!("🔍 Scanning folder: {:?}", folder);

    let jars: Vec<PathBuf> = collect_jars(folder)?;
    info!("📦 Found {} jar files", jars.len());

    let mods: Vec<ModInfo> = jars
        .par_iter()
        .filter_map(|jar| match read_mod_info(jar) {
            Ok(m) => Some(m),
            Err(e) => {
                warn!("Failed to read {:?}: {}", jar, e);
                None
            }
        })
        .collect();

    info!("✅ Scanned {} valid mods", mods.len());
    Ok(mods)
}

fn collect_jars(folder: &Path) -> Result<Vec<PathBuf>> {
    let mut jars = Vec::new();
    if !folder.exists() {
        return Ok(jars);
    }

    fn walk(dir: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
        if dir.is_dir() {
            for entry in std::fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    walk(&path, out)?;
                } else if path.extension().and_then(|s| s.to_str()) == Some("jar") {
                    out.push(path);
                }
            }
        }
        Ok(())
    }

    walk(folder, &mut jars)?;
    Ok(jars)
}

pub fn read_mod_info(jar_path: &Path) -> Result<ModInfo> {
    let file = std::fs::File::open(jar_path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    let size = std::fs::metadata(jar_path)?.len();

    // ⚠️ SHA1 quitado por rendimiento con muchos mods
    // Se puede calcular después si es necesario
    let sha1 = String::from("pending");

    // Detect loader + metadata
    let (loader, base_info) = detect_loader(&mut archive)?;

    // Find lang files
    let lang_files = find_lang_files(&mut archive)?;

    Ok(ModInfo {
        id: base_info.id,
        name: base_info.name,
        version: base_info.version,
        author: base_info.author,
        mc_version: base_info.mc_version,
        loader,
        path: jar_path.to_string_lossy().to_string(),
        sha1,
        size_bytes: size,
        lang_files,
        dependencies: base_info.dependencies,
        description: base_info.description,
    })
}

struct BaseInfo {
    id: String,
    name: String,
    version: String,
    author: String,
    mc_version: String,
    dependencies: Vec<String>,
    description: Option<String>,
}

fn detect_loader(archive: &mut zip::ZipArchive<std::fs::File>) -> Result<(Loader, BaseInfo)> {
    // Try Forge / NeoForge: META-INF/mods.toml or neoforge.mods.toml
    if let Ok(mut file) = archive.by_name("META-INF/neoforge.mods.toml") {
        let mut s = String::new();
        std::io::Read::read_to_string(&mut file, &mut s)?;
        if let Ok(info) = parse_mods_toml(&s, Loader::NeoForge) {
            return Ok((Loader::NeoForge, info));
        }
    }
    if let Ok(mut file) = archive.by_name("META-INF/mods.toml") {
        let mut s = String::new();
        std::io::Read::read_to_string(&mut file, &mut s)?;
        if let Ok(info) = parse_mods_toml(&s, Loader::Forge) {
            return Ok((Loader::Forge, info));
        }
    }

    // Try Fabric: fabric.mod.json
    if let Ok(mut file) = archive.by_name("fabric.mod.json") {
        let mut s = String::new();
        std::io::Read::read_to_string(&mut file, &mut s)?;
        if let Ok(info) = parse_fabric_json(&s) {
            return Ok((Loader::Fabric, info));
        }
    }

    // Try Quilt: quilt.mod.json
    if let Ok(mut file) = archive.by_name("quilt.mod.json") {
        let mut s = String::new();
        std::io::Read::read_to_string(&mut file, &mut s)?;
        if let Ok(info) = parse_quilt_json(&s) {
            return Ok((Loader::Quilt, info));
        }
    }

    // Fallback: guess from jar name
    debug!("No loader metadata found, using filename fallback");
    Ok((Loader::Unknown, BaseInfo {
        id: "unknown".into(),
        name: "Unknown Mod".into(),
        version: "0.0.0".into(),
        author: "Unknown".into(),
        mc_version: "".into(),
        dependencies: vec![],
        description: None,
    }))
}

fn parse_mods_toml(content: &str, _loader: Loader) -> Result<BaseInfo> {
    let v: toml::Value = toml::from_str(content)?;

    // mods.toml has [[mods]] array usually
    let mods_arr = v.get("mods").and_then(|m| m.as_array());
    let first = mods_arr.and_then(|a| a.first());

    let id = first
        .and_then(|m| m.get("modId"))
        .and_then(|v| v.as_str())
        .or_else(|| {
            // Older format: top-level modId
            v.get("modId").and_then(|v| v.as_str())
        })
        .unwrap_or("unknown")
        .to_string();

    let name = first
        .and_then(|m| m.get("displayName"))
        .and_then(|v| v.as_str())
        .unwrap_or(&id)
        .to_string();

    let version = first
        .and_then(|m| m.get("version"))
        .and_then(|v| v.as_str())
        .unwrap_or("0.0.0")
        .to_string();

    let author = first
        .and_then(|m| m.get("authors"))
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown")
        .to_string();

    let description = first
        .and_then(|m| m.get("description"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    // MC version from dependencies.minecraft.versionRange
    let mc_version = v
        .get("dependencies")
        .and_then(|d| d.get(&id))
        .and_then(|arr| arr.as_array())
        .and_then(|arr| arr.first())
        .and_then(|dep| dep.get("minecraft"))
        .and_then(|mc| mc.get("versionRange"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let deps = v
        .get("dependencies")
        .and_then(|d| d.get(&id))
        .and_then(|arr| arr.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|dep| dep.get("modId").and_then(|v| v.as_str()).map(String::from))
                .filter(|s| s != "minecraft")
                .collect()
        })
        .unwrap_or_default();

    Ok(BaseInfo { id, name, version, author, mc_version, dependencies: deps, description })
}

fn parse_fabric_json(content: &str) -> Result<BaseInfo> {
    let v: serde_json::Value = serde_json::from_str(content)?;

    let id = v["id"].as_str().unwrap_or("unknown").to_string();
    let name = v["name"].as_str().unwrap_or(&id).to_string();
    let version = v["version"].as_str().unwrap_or("0.0.0").to_string();

    let author = match v["authors"].as_array() {
        Some(arr) => arr
            .iter()
            .filter_map(|a| {
                if let Some(s) = a.as_str() {
                    Some(s.to_string())
                } else {
                    a.get("name").and_then(|n| n.as_str()).map(String::from)
                }
            })
            .collect::<Vec<_>>()
            .join(", "),
        None => "Unknown".into(),
    };

    let description = v["description"].as_str().map(String::from);

    // MC version from depends.minecraft
    let mc_version = v["depends"]["minecraft"]
        .as_str()
        .unwrap_or("")
        .to_string();

    let deps = v["depends"]
        .as_object()
        .map(|o| o.keys().filter(|k| *k != "minecraft" && *k != "fabricloader").cloned().collect())
        .unwrap_or_default();

    Ok(BaseInfo { id, name, version, author, mc_version, dependencies: deps, description })
}

fn parse_quilt_json(content: &str) -> Result<BaseInfo> {
    let v: serde_json::Value = serde_json::from_str(content)?;

    let id = v["quilt_loader"]["id"].as_str().unwrap_or("unknown").to_string();
    let name = v["quilt_loader"]["metadata"]["name"].as_str().unwrap_or(&id).to_string();
    let version = v["quilt_loader"]["version"].as_str().unwrap_or("0.0.0").to_string();

    let author = v["quilt_loader"]["metadata"]["contributors"]
        .as_object()
        .map(|o| o.keys().cloned().collect::<Vec<_>>().join(", "))
        .unwrap_or_else(|| "Unknown".into());

    let description = v["quilt_loader"]["metadata"]["description"].as_str().map(String::from);

    let mc_version = v["quilt_loader"]["depends"]["minecraft"]
        .as_str()
        .unwrap_or("")
        .to_string();

    let deps = v["quilt_loader"]["depends"]
        .as_object()
        .map(|o| o.keys().filter(|k| *k != "minecraft").cloned().collect())
        .unwrap_or_default();

    Ok(BaseInfo { id, name, version, author, mc_version, dependencies: deps, description })
}

fn find_lang_files(archive: &mut zip::ZipArchive<std::fs::File>) -> Result<Vec<LangFile>> {
    let mut files = Vec::new();

    for i in 0..archive.len() {
        let entry = match archive.by_index(i) {
            Ok(e) => e,
            Err(_) => continue,
        };

        let name = entry.name().to_string();

        // Solo miramos nombres, NO leemos contenido (mucho más rápido)
        if let Some((locale, format)) = jar_reader::parse_lang_path_public(&name) {
            files.push(LangFile {
                path: name,
                locale,
                format,
                keys_count: 0, // Se calcula solo cuando se traduce, no en el escaneo
                chars_count: entry.size() as usize,
            });
        }
    }

    Ok(files)
}