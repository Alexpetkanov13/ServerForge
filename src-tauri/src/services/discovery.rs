use std::path::{Path, PathBuf};

use crate::error::AppResult;
use crate::models::DiscoveredServer;
use crate::utils::path::looks_like_server_dir;
use crate::utils::properties::{get_port, parse_properties};

pub async fn scan(root: &Path) -> AppResult<Vec<DiscoveredServer>> {
    let mut found = Vec::new();
    scan_dir(root, 0, &mut found);
    Ok(found)
}

fn scan_dir(dir: &Path, depth: u8, out: &mut Vec<DiscoveredServer>) {
    if depth > 4 {
        return;
    }
    if looks_like_server_dir(dir) {
        out.push(inspect(dir));
        return;
    }
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in rd.flatten() {
        if entry.path().is_dir() {
            scan_dir(&entry.path(), depth + 1, out);
        }
    }
}

pub fn inspect(dir: &Path) -> DiscoveredServer {
    let name = dir
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "Imported Server".into());
    let props = std::fs::read_to_string(dir.join("server.properties")).unwrap_or_default();
    let provider = detect_provider(dir);
    DiscoveredServer {
        path: dir.to_string_lossy().to_string(),
        name,
        provider: provider.clone(),
        minecraft_version: detect_version(dir),
        plugin_count: count_jars(&dir.join("plugins")),
        mod_count: count_jars(&dir.join("mods")),
        world_count: count_worlds(dir),
        has_eula: dir.join("eula.txt").exists(),
        port: Some(get_port(&props)),
    }
}

fn detect_provider(dir: &Path) -> Option<String> {
    let names = file_names(dir);
    if dir.join("paper.yml").exists() || names.iter().any(|n| n.starts_with("paper-")) {
        return Some("paper".into());
    }
    if dir.join("purpur.yml").exists() {
        return Some("purpur".into());
    }
    if dir.join("folia.yml").exists() || names.iter().any(|n| n.contains("folia")) {
        return Some("folia".into());
    }
    if dir.join("spigot.yml").exists() {
        return Some("spigot".into());
    }
    if names.iter().any(|n| n.contains("fabric")) || dir.join(".fabric").exists() {
        return Some("fabric".into());
    }
    if names.iter().any(|n| n.contains("neoforge")) {
        return Some("neoforge".into());
    }
    if names.iter().any(|n| n.contains("forge")) || dir.join("user_jvm_args.txt").exists() {
        return Some("forge".into());
    }
    if dir.join("server.jar").exists() || dir.join("server.properties").exists() {
        return Some("vanilla".into());
    }
    None
}

fn detect_version(dir: &Path) -> Option<String> {
    for name in file_names(dir) {
        if let Some(v) = extract_mc_version(&name) {
            return Some(v);
        }
    }
    let version_json = dir.join("version.json");
    if let Ok(text) = std::fs::read_to_string(version_json) {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
            if let Some(id) = v.get("id").and_then(|x| x.as_str()) {
                return Some(id.to_string());
            }
        }
    }
    let _ = parse_properties("");
    None
}

fn extract_mc_version(name: &str) -> Option<String> {
    let re = regex::Regex::new(r"1\.\d{1,2}(?:\.\d+)?").ok()?;
    re.find(name).map(|m| m.as_str().to_string())
}

fn file_names(dir: &Path) -> Vec<String> {
    std::fs::read_dir(dir)
        .map(|rd| {
            rd.flatten()
                .map(|e| e.file_name().to_string_lossy().to_ascii_lowercase())
                .collect()
        })
        .unwrap_or_default()
}

fn count_jars(dir: &Path) -> u32 {
    std::fs::read_dir(dir)
        .map(|rd| {
            rd.flatten()
                .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("jar"))
                .count() as u32
        })
        .unwrap_or(0)
}

fn count_worlds(dir: &Path) -> u32 {
    ["world", "world_nether", "world_the_end"]
        .iter()
        .filter(|name| dir.join(name).join("level.dat").exists() || dir.join(name).is_dir())
        .count() as u32
}

pub fn suggested_path(root: &Path, name: &str) -> PathBuf {
    root.join(crate::utils::path::folder_name_from_server(name))
}
