use std::path::Path;
use tauri::{AppHandle, Emitter};

use crate::error::{AppError, AppResult};
use crate::models::{AddonInfo, AddonVersion, InstalledAddon};
use crate::services::download::DownloadService;
use crate::utils::http::get_json;

pub async fn list_installed(server_path: &Path, kind: &str) -> AppResult<Vec<InstalledAddon>> {
    let dir = server_path.join(if kind == "mods" { "mods" } else { "plugins" });
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut rd = tokio::fs::read_dir(&dir).await?;
    let mut out = Vec::new();
    while let Some(entry) = rd.next_entry().await? {
        let name = entry.file_name().to_string_lossy().to_string();
        let enabled = name.ends_with(".jar");
        if name.ends_with(".jar") || name.ends_with(".jar.disabled") {
            out.push(InstalledAddon {
                name: name.trim_end_matches(".disabled").trim_end_matches(".jar").to_string(),
                file_name: name,
                enabled,
                version: None,
                path: entry.path().to_string_lossy().to_string(),
            });
        }
    }
    out.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(out)
}

pub async fn set_enabled(path: &Path, enabled: bool) -> AppResult<()> {
    let s = path.to_string_lossy();
    if enabled && s.ends_with(".jar.disabled") {
        let dest = Path::new(s.trim_end_matches(".disabled"));
        tokio::fs::rename(path, dest).await?;
    } else if !enabled && s.ends_with(".jar") {
        let dest = path.with_extension("jar.disabled");
        tokio::fs::rename(path, dest).await?;
    }
    Ok(())
}

pub async fn remove_addon(path: &Path) -> AppResult<()> {
    tokio::fs::remove_file(path).await?;
    Ok(())
}

pub fn hangar_platform(provider: &str) -> &'static str {
    match provider {
        "folia" => "FOLIA",
        "velocity" => "VELOCITY",
        "waterfall" => "WATERFALL",
        _ => "PAPER",
    }
}

pub fn plugin_loaders(provider: &str) -> Vec<&'static str> {
    match provider {
        "folia" => vec!["folia"],
        "purpur" => vec!["purpur", "paper", "spigot", "bukkit"],
        "spigot" => vec!["spigot", "bukkit", "paper"],
        _ => vec!["paper", "spigot", "bukkit", "purpur"],
    }
}

pub fn version_compatible(available: &[String], target: &str) -> bool {
    if target.is_empty() {
        return true;
    }
    if available.iter().any(|v| v == target) {
        return true;
    }
    let family = mc_family(target);
    available.iter().any(|v| mc_family(v) == family)
}

fn mc_family(version: &str) -> String {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() >= 2 {
        format!("{}.{}", parts[0], parts[1])
    } else {
        version.to_string()
    }
}

fn versions_from_json(value: &serde_json::Value) -> Vec<String> {
    value
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default()
}

pub async fn search_plugins(
    client: &reqwest::Client,
    query: &str,
    minecraft_version: &str,
    provider: &str,
) -> AppResult<Vec<AddonInfo>> {
    let q = query.trim();
    if q.is_empty() {
        return Ok(vec![]);
    }
    let platform = hangar_platform(provider);
    let loaders = plugin_loaders(provider);
    let (hangar, modrinth) = tokio::join!(
        search_hangar_plugins(client, q, minecraft_version, platform),
        search_modrinth_plugins(client, q, minecraft_version, &loaders)
    );
    let mut merged = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for item in modrinth.unwrap_or_default().into_iter().chain(hangar.unwrap_or_default()) {
        let key = item.name.to_ascii_lowercase();
        if seen.insert(key) {
            merged.push(item);
        }
    }
    merged.sort_by(|a, b| {
        b.compatible
            .cmp(&a.compatible)
            .then(b.downloads.cmp(&a.downloads))
            .then(a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    Ok(merged)
}

async fn search_hangar_plugins(
    client: &reqwest::Client,
    query: &str,
    minecraft_version: &str,
    platform: &str,
) -> AppResult<Vec<AddonInfo>> {
    let mut url = format!(
        "https://hangar.papermc.io/api/v1/projects?q={}&limit=25&offset=0&platform={}",
        urlencoding::encode(query),
        platform
    );
    if !minecraft_version.is_empty() {
        url.push_str(&format!("&version={}", urlencoding::encode(minecraft_version)));
    }
    let mut value: serde_json::Value = get_json(client, &url).await?;
    let mut result = value.get("result").and_then(|v| v.as_array()).cloned().unwrap_or_default();
    if result.is_empty() && !minecraft_version.is_empty() {
        let fallback = format!(
            "https://hangar.papermc.io/api/v1/projects?q={}&limit=25&offset=0&platform={}",
            urlencoding::encode(query),
            platform
        );
        value = get_json(client, &fallback).await?;
        result = value.get("result").and_then(|v| v.as_array()).cloned().unwrap_or_default();
    }
    Ok(result
        .into_iter()
        .filter_map(|item| {
            let versions = item
                .get("supportedPlatforms")
                .and_then(|v| v.get(platform).or_else(|| v.get("PAPER")))
                .map(versions_from_json)
                .unwrap_or_default();
            Some(AddonInfo {
                id: item.get("namespace")?.get("slug")?.as_str()?.to_string(),
                slug: item.get("namespace")?.get("slug")?.as_str()?.to_string(),
                name: item.get("name")?.as_str()?.to_string(),
                description: item.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                author: item.get("namespace")?.get("owner")?.as_str()?.to_string(),
                downloads: item.get("stats")?.get("downloads")?.as_u64().unwrap_or(0),
                icon_url: item.get("avatarUrl").and_then(|v| v.as_str()).map(|s| s.to_string()),
                platform: platform.to_ascii_lowercase(),
                source: "hangar".into(),
                compatible: version_compatible(&versions, minecraft_version),
                game_versions: versions,
            })
        })
        .collect())
}

async fn search_modrinth_plugins(
    client: &reqwest::Client,
    query: &str,
    minecraft_version: &str,
    loaders: &[&str],
) -> AppResult<Vec<AddonInfo>> {
    let mut categories: Vec<String> = loaders.iter().map(|l| format!("categories:{l}")).collect();
    if categories.is_empty() {
        categories.push("categories:paper".into());
    }
    let mut facets = serde_json::json!([["project_type:plugin"], categories]);
    if !minecraft_version.is_empty() {
        facets = serde_json::json!([
            ["project_type:plugin"],
            [format!("versions:{minecraft_version}")],
            loaders.iter().map(|l| format!("categories:{l}")).collect::<Vec<_>>()
        ]);
    }
    let mut url = format!(
        "https://api.modrinth.com/v2/search?query={}&limit=25&facets={}",
        urlencoding::encode(query),
        urlencoding::encode(&facets.to_string())
    );
    let mut value: serde_json::Value = get_json(client, &url).await?;
    let mut hits = value.get("hits").and_then(|v| v.as_array()).cloned().unwrap_or_default();
    if hits.is_empty() && !minecraft_version.is_empty() {
        facets = serde_json::json!([
            ["project_type:plugin"],
            loaders.iter().map(|l| format!("categories:{l}")).collect::<Vec<_>>()
        ]);
        url = format!(
            "https://api.modrinth.com/v2/search?query={}&limit=25&facets={}",
            urlencoding::encode(query),
            urlencoding::encode(&facets.to_string())
        );
        value = get_json(client, &url).await?;
        hits = value.get("hits").and_then(|v| v.as_array()).cloned().unwrap_or_default();
    }
    Ok(hits
        .into_iter()
        .filter_map(|item| {
            let versions = versions_from_json(item.get("versions").unwrap_or(&serde_json::Value::Null));
            Some(AddonInfo {
                id: item.get("project_id")?.as_str()?.to_string(),
                slug: item.get("slug")?.as_str()?.to_string(),
                name: item.get("title")?.as_str()?.to_string(),
                description: item.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                author: item.get("author").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                downloads: item.get("downloads").and_then(|v| v.as_u64()).unwrap_or(0),
                icon_url: item.get("icon_url").and_then(|v| v.as_str()).map(|s| s.to_string()),
                platform: loaders.first().copied().unwrap_or("paper").to_string(),
                source: "modrinth".into(),
                compatible: version_compatible(&versions, minecraft_version),
                game_versions: versions,
            })
        })
        .collect())
}

pub async fn search_mods(client: &reqwest::Client, query: &str, loader: &str, minecraft_version: &str) -> AppResult<Vec<AddonInfo>> {
    let mut facets = serde_json::json!([["project_type:mod"], [format!("categories:{loader}")]]);
    if !minecraft_version.is_empty() {
        facets = serde_json::json!([
            ["project_type:mod"],
            [format!("categories:{loader}")],
            [format!("versions:{minecraft_version}")]
        ]);
    }
    let url = format!(
        "https://api.modrinth.com/v2/search?query={}&limit=25&facets={}",
        urlencoding::encode(query),
        urlencoding::encode(&facets.to_string())
    );
    let value: serde_json::Value = get_json(client, &url).await?;
    let hits = value.get("hits").and_then(|v| v.as_array()).cloned().unwrap_or_default();
    Ok(hits
        .into_iter()
        .filter_map(|item| {
            let versions = versions_from_json(item.get("versions").unwrap_or(&serde_json::Value::Null));
            Some(AddonInfo {
                id: item.get("project_id")?.as_str()?.to_string(),
                slug: item.get("slug")?.as_str()?.to_string(),
                name: item.get("title")?.as_str()?.to_string(),
                description: item.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                author: item.get("author").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                downloads: item.get("downloads").and_then(|v| v.as_u64()).unwrap_or(0),
                icon_url: item.get("icon_url").and_then(|v| v.as_str()).map(|s| s.to_string()),
                platform: loader.to_string(),
                source: "modrinth".into(),
                compatible: version_compatible(&versions, minecraft_version),
                game_versions: versions,
            })
        })
        .collect())
}

pub async fn plugin_versions(
    client: &reqwest::Client,
    author: &str,
    slug: &str,
    minecraft_version: &str,
    platform: &str,
) -> AppResult<Vec<AddonVersion>> {
    let platform = if platform.is_empty() { "PAPER" } else { platform };
    let url = format!(
        "https://hangar.papermc.io/api/v1/projects/{author}/{slug}/versions?limit=25&offset=0&platform={platform}"
    );
    let value: serde_json::Value = get_json(client, &url).await?;
    let result = value.get("result").and_then(|v| v.as_array()).cloned().unwrap_or_default();
    Ok(result
        .into_iter()
        .filter_map(|item| {
            let name = item.get("name")?.as_str()?.to_string();
            let downloads = item.get("downloads")?.as_object()?;
            let platform_dl = downloads
                .get(platform)
                .or_else(|| downloads.get("PAPER"))
                .or_else(|| downloads.values().next())?;
            let game_versions = item
                .get("platformDependencies")
                .and_then(|v| v.get(platform).or_else(|| v.get("PAPER")))
                .map(versions_from_json)
                .unwrap_or_default();
            if !minecraft_version.is_empty() && !version_compatible(&game_versions, minecraft_version) {
                return None;
            }
            let file_name = platform_dl
                .get("fileInfo")
                .and_then(|v| v.get("name"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let download_url = platform_dl
                .get("downloadUrl")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty() && s.starts_with("https://"))
                .map(|s| s.to_string())
                .or_else(|| {
                    if file_name.is_some() {
                        Some(format!(
                            "https://hangar.papermc.io/api/v1/projects/{author}/{slug}/versions/{name}/{platform}/download"
                        ))
                    } else {
                        None
                    }
                })?;
            Some(AddonVersion {
                id: name.clone(),
                name: name.clone(),
                version_number: name,
                game_versions,
                loaders: vec![platform.to_ascii_lowercase()],
                download_url: Some(download_url),
                file_name,
                sha512: None,
                required_dependencies: item
                    .get("pluginDependencies")
                    .and_then(|v| v.get(platform).or_else(|| v.get("PAPER")))
                    .and_then(|v| v.as_array())
                    .map(|deps| {
                        deps.iter()
                            .filter(|d| d.get("required").and_then(|v| v.as_bool()).unwrap_or(true))
                            .filter_map(|d| d.get("name").and_then(|v| v.as_str()).map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default(),
            })
        })
        .collect())
}

pub async fn resolve_plugin_download(
    client: &reqwest::Client,
    source: &str,
    id: &str,
    author: &str,
    slug: &str,
    minecraft_version: &str,
    provider: &str,
) -> AppResult<AddonVersion> {
    if source == "modrinth" {
        for loader in plugin_loaders(provider) {
            let versions = mod_versions(client, id, loader, minecraft_version).await?;
            if let Some(v) = versions.into_iter().find(|v| v.download_url.is_some()) {
                return Ok(v);
            }
        }
        let versions = mod_versions(client, id, plugin_loaders(provider)[0], "").await?;
        if let Some(v) = versions.into_iter().find(|v| {
            v.download_url.is_some() && version_compatible(&v.game_versions, minecraft_version)
        }) {
            return Ok(v);
        }
        return Err(AppError::new(
            "No compatible plugin file",
            "Modrinth does not have a jar for this Minecraft version.",
        ));
    }
    let versions = plugin_versions(client, author, slug, minecraft_version, hangar_platform(provider)).await?;
    versions
        .into_iter()
        .find(|v| v.download_url.is_some())
        .ok_or_else(|| {
            AppError::new(
                "No downloadable jar",
                "Hangar listed this plugin but did not provide a jar. Try a Modrinth result with the same name.",
            )
        })
}

pub async fn mod_versions(client: &reqwest::Client, id: &str, loader: &str, game: &str) -> AppResult<Vec<AddonVersion>> {
    let url = format!("https://api.modrinth.com/v2/project/{id}/version");
    let value: serde_json::Value = get_json(client, &url).await?;
    let arr = value.as_array().cloned().unwrap_or_default();
    Ok(arr
        .into_iter()
        .filter(|item| {
            let loaders = item.get("loaders").and_then(|v| v.as_array()).cloned().unwrap_or_default();
            let games = item.get("game_versions").and_then(|v| v.as_array()).cloned().unwrap_or_default();
            let loader_ok = loader.is_empty()
                || loaders.iter().any(|v| v.as_str() == Some(loader));
            let listed: Vec<String> = games.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect();
            let game_ok = version_compatible(&listed, game);
            loader_ok && game_ok
        })
        .filter_map(|item| {
            let files = item.get("files")?.as_array()?;
            let file = files.iter().find(|f| f.get("primary").and_then(|v| v.as_bool()) == Some(true)).or_else(|| files.first())?;
            Some(AddonVersion {
                id: item.get("id")?.as_str()?.to_string(),
                name: item.get("name")?.as_str()?.to_string(),
                version_number: item.get("version_number")?.as_str()?.to_string(),
                game_versions: item
                    .get("game_versions")?
                    .as_array()?
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect(),
                loaders: item
                    .get("loaders")?
                    .as_array()?
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect(),
                download_url: file.get("url").and_then(|v| v.as_str()).map(|s| s.to_string()),
                file_name: file.get("filename").and_then(|v| v.as_str()).map(|s| s.to_string()),
                sha512: file.pointer("/hashes/sha512").and_then(|v| v.as_str()).map(|s| s.to_string()),
                required_dependencies: item
                    .get("dependencies")
                    .and_then(|v| v.as_array())
                    .map(|deps| {
                        deps.iter()
                            .filter(|d| d.get("dependency_type").and_then(|v| v.as_str()) == Some("required"))
                            .filter_map(|d| d.get("project_id").and_then(|v| v.as_str()).map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default(),
            })
        })
        .collect())
}

pub async fn install_from_url(
    app: AppHandle,
    downloads: &DownloadService,
    client: &reqwest::Client,
    url: &str,
    dest: &Path,
    label: &str,
) -> AppResult<()> {
    if let Some(parent) = dest.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    if !url.starts_with("https://") {
        return Err(AppError::new(
            "Insecure download blocked",
            "Addons can only be downloaded over HTTPS.",
        ));
    }
    let app2 = app.clone();
    downloads
        .download_file(client, url, dest, label, None, None, move |p| {
            let _ = app2.emit("download-progress", p);
        })
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::version_compatible;

    #[test]
    fn plugin_version_family_matches() {
        assert!(version_compatible(&["1.21.8".into(), "1.20.6".into()], "1.21.1"));
        assert!(version_compatible(&["1.21".into()], "1.21.4"));
        assert!(!version_compatible(&["1.20.6".into()], "1.21.1"));
    }
}
