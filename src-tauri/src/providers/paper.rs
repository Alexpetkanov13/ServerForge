use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;

use crate::error::{AppError, AppResult};
use crate::models::{
    ArtifactDownload, BuildInfo, Ecosystem, MinecraftVersionInfo, ProviderId, ProviderInfo,
};
use crate::utils::http::get_json;
use crate::utils::java_version::{channel_for_version, recommended_java_major};

use super::ServerProvider;

pub fn info() -> ProviderInfo {
    ProviderInfo {
        id: "paper".into(),
        name: "Paper".into(),
        description: "High-performance fork of Spigot with plugin support.".into(),
        recommended_use: "Survival, SMP, and almost every plugin-based community.".into(),
        ecosystem: Ecosystem::Bukkit,
        supports_plugins: true,
        supports_mods: false,
        performance: "Excellent".into(),
    }
}

pub struct PaperFamilyProvider {
    client: Client,
    project: &'static str,
    provider_id: ProviderId,
    meta: ProviderInfo,
}

impl PaperFamilyProvider {
    pub fn paper(client: Client) -> Self {
        Self {
            client,
            project: "paper",
            provider_id: ProviderId::Paper,
            meta: info(),
        }
    }

    pub fn folia(client: Client, meta: ProviderInfo) -> Self {
        Self {
            client,
            project: "folia",
            provider_id: ProviderId::Folia,
            meta,
        }
    }

    fn v3_project(&self) -> String {
        format!("https://fill.papermc.io/v3/projects/{}", self.project)
    }

    fn v3_builds(&self, version: &str) -> String {
        format!(
            "https://fill.papermc.io/v3/projects/{}/versions/{}/builds",
            self.project, version
        )
    }

    fn v2_project(&self) -> String {
        format!("https://api.papermc.io/v2/projects/{}", self.project)
    }

    fn v2_builds(&self, version: &str) -> String {
        format!(
            "https://api.papermc.io/v2/projects/{}/versions/{}/builds",
            self.project, version
        )
    }
}

#[derive(Debug, Deserialize)]
struct V2Project {
    versions: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct V2Builds {
    builds: Vec<V2Build>,
}

#[derive(Debug, Deserialize)]
struct V2Build {
    build: i64,
    channel: Option<String>,
    downloads: Option<Value>,
}

#[async_trait]
impl ServerProvider for PaperFamilyProvider {
    fn id(&self) -> ProviderId {
        self.provider_id
    }

    fn info(&self) -> ProviderInfo {
        self.meta.clone()
    }

    async fn list_minecraft_versions(&self) -> AppResult<Vec<MinecraftVersionInfo>> {
        if let Ok(versions) = self.list_v3_versions().await {
            return Ok(versions);
        }
        self.list_v2_versions().await
    }

    async fn list_builds(&self, minecraft_version: &str) -> AppResult<Vec<BuildInfo>> {
        if let Ok(builds) = self.list_v3_builds(minecraft_version).await {
            if !builds.is_empty() {
                return Ok(builds);
            }
        }
        self.list_v2_builds(minecraft_version).await
    }

    async fn resolve_download(
        &self,
        minecraft_version: &str,
        build: Option<&str>,
        _loader_version: Option<&str>,
        _installer_version: Option<&str>,
    ) -> AppResult<ArtifactDownload> {
        let builds = self.list_builds(minecraft_version).await?;
        let selected = select_build(&builds, build).ok_or_else(|| {
            AppError::new(
                "Build unavailable",
                format!(
                    "No {} build was found for Minecraft {minecraft_version}.",
                    self.meta.name
                ),
            )
        })?;
        if let Ok(Some(artifact)) = self.v3_artifact(minecraft_version, &selected.id).await {
            return Ok(artifact);
        }
        self.v2_artifact(minecraft_version, &selected.id).await
    }
}

impl PaperFamilyProvider {
    async fn list_v3_versions(&self) -> AppResult<Vec<MinecraftVersionInfo>> {
        let value: Value = get_json(&self.client, &self.v3_project()).await?;
        let mut ids = Vec::new();
        if let Some(map) = value.get("versions").and_then(|v| v.as_object()) {
            for versions in map.values() {
                if let Some(arr) = versions.as_array() {
                    for item in arr {
                        if let Some(id) = item.as_str() {
                            ids.push(id.to_string());
                        }
                    }
                }
            }
        } else if let Some(arr) = value.get("versions").and_then(|v| v.as_array()) {
            for item in arr {
                if let Some(id) = item.as_str() {
                    ids.push(id.to_string());
                }
            }
        }
        if ids.is_empty() {
            return Err(AppError::message("Paper versions list was empty"));
        }
        let latest = ids.first().cloned().unwrap_or_default();
        Ok(ids
            .into_iter()
            .map(|id| MinecraftVersionInfo {
                latest: id == latest,
                recommended: id == latest,
                java_major: recommended_java_major(&id),
                channel: channel_for_version(&id).into(),
                released: None,
                id,
            })
            .collect())
    }

    async fn list_v2_versions(&self) -> AppResult<Vec<MinecraftVersionInfo>> {
        let project: V2Project = get_json(&self.client, &self.v2_project()).await?;
        let latest = project.versions.last().cloned().unwrap_or_default();
        Ok(project
            .versions
            .into_iter()
            .rev()
            .map(|id| MinecraftVersionInfo {
                latest: id == latest,
                recommended: id == latest,
                java_major: recommended_java_major(&id),
                channel: channel_for_version(&id).into(),
                released: None,
                id,
            })
            .collect())
    }

    async fn list_v3_builds(&self, version: &str) -> AppResult<Vec<BuildInfo>> {
        let value: Value = get_json(&self.client, &self.v3_builds(version)).await?;
        let arr = value.as_array().cloned().unwrap_or_default();
        let mut builds = Vec::new();
        for item in arr {
            let id = item
                .get("id")
                .or_else(|| item.get("number"))
                .and_then(|v| v.as_i64().or_else(|| v.as_u64().map(|n| n as i64)))
                .map(|n| n.to_string())
                .or_else(|| item.get("id").and_then(|v| v.as_str()).map(|s| s.to_string()));
            let Some(id) = id else { continue };
            let channel = item
                .get("channel")
                .and_then(|v| v.as_str())
                .unwrap_or("UNKNOWN")
                .to_ascii_lowercase();
            builds.push(BuildInfo {
                id: id.clone(),
                label: format!("Build #{id}"),
                recommended: channel == "stable",
                channel,
                released: item
                    .get("time")
                    .or_else(|| item.get("createdAt"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                loader_version: None,
                installer_version: None,
            });
        }
        if let Some(first_stable) = builds.iter_mut().find(|b| b.channel == "stable") {
            first_stable.recommended = true;
        } else if let Some(first) = builds.first_mut() {
            first.recommended = true;
            first.label = format!("{} (latest)", first.label);
        }
        Ok(builds)
    }

    async fn list_v2_builds(&self, version: &str) -> AppResult<Vec<BuildInfo>> {
        let payload: V2Builds = get_json(&self.client, &self.v2_builds(version)).await?;
        let mut builds: Vec<BuildInfo> = payload
            .builds
            .into_iter()
            .rev()
            .map(|b| {
                let channel = b.channel.unwrap_or_else(|| "default".into()).to_ascii_lowercase();
                BuildInfo {
                    id: b.build.to_string(),
                    label: format!("Build #{}", b.build),
                    recommended: channel == "default" || channel == "stable",
                    channel,
                    released: None,
                    loader_version: None,
                    installer_version: None,
                }
            })
            .collect();
        if let Some(first) = builds.first_mut() {
            first.recommended = true;
        }
        let _ = payload;
        Ok(builds)
    }

    async fn v3_artifact(&self, version: &str, build: &str) -> AppResult<Option<ArtifactDownload>> {
        let value: Value = get_json(&self.client, &self.v3_builds(version)).await?;
        let arr = value.as_array().cloned().unwrap_or_default();
        for item in arr {
            let id = item
                .get("id")
                .or_else(|| item.get("number"))
                .and_then(|v| v.as_i64().map(|n| n.to_string()).or_else(|| v.as_str().map(|s| s.to_string())));
            if id.as_deref() != Some(build) {
                continue;
            }
            let download = item
                .pointer("/downloads/server:default")
                .or_else(|| item.pointer("/downloads/application"))
                .cloned();
            if let Some(download) = download {
                let url = download
                    .get("url")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| AppError::message("Paper download URL missing"))?;
                let name = download
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("server.jar");
                let sha256 = download
                    .pointer("/checksums/sha256")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                let size = download.get("size").and_then(|v| v.as_u64());
                return Ok(Some(ArtifactDownload {
                    url: url.to_string(),
                    file_name: name.to_string(),
                    sha256,
                    sha1: None,
                    size,
                    kind: "server".into(),
                }));
            }
        }
        Ok(None)
    }

    async fn v2_artifact(&self, version: &str, build: &str) -> AppResult<ArtifactDownload> {
        let url = format!(
            "https://api.papermc.io/v2/projects/{}/versions/{}/builds/{}",
            self.project, version, build
        );
        let value: Value = get_json(&self.client, &url).await?;
        let downloads = value.get("downloads").cloned().unwrap_or(Value::Null);
        let application = downloads
            .get("application")
            .cloned()
            .or_else(|| downloads.get("server:default").cloned())
            .ok_or_else(|| AppError::message("Paper v2 download metadata missing"))?;
        let name = application
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("server.jar")
            .to_string();
        let sha256 = application
            .pointer("/checksums/sha256")
            .and_then(|v| v.as_str())
            .or_else(|| application.get("sha256").and_then(|v| v.as_str()))
            .map(|s| s.to_string());
        Ok(ArtifactDownload {
            url: format!("{url}/downloads/{name}"),
            file_name: name,
            sha256,
            sha1: None,
            size: None,
            kind: "server".into(),
        })
    }
}

fn select_build<'a>(builds: &'a [BuildInfo], requested: Option<&str>) -> Option<&'a BuildInfo> {
    if let Some(id) = requested {
        if id != "latest" && id != "recommended" {
            if let Some(found) = builds.iter().find(|b| b.id == id) {
                return Some(found);
            }
        }
    }
    builds
        .iter()
        .find(|b| b.recommended)
        .or_else(|| builds.first())
}
