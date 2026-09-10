use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

use crate::error::{AppError, AppResult};
use crate::models::{
    ArtifactDownload, BuildInfo, Ecosystem, MinecraftVersionInfo, ProviderId, ProviderInfo,
};
use crate::utils::http::get_json;
use crate::utils::java_version::{channel_for_version, recommended_java_major};

use super::ServerProvider;

pub fn info() -> ProviderInfo {
    ProviderInfo {
        id: "purpur".into(),
        name: "Purpur".into(),
        description: "Paper fork with extra gameplay configuration and extras.".into(),
        recommended_use: "Servers that want Paper performance plus richer vanilla-like options.".into(),
        ecosystem: Ecosystem::Bukkit,
        supports_plugins: true,
        supports_mods: false,
        performance: "Excellent".into(),
    }
}

pub struct PurpurProvider {
    client: Client,
}

impl PurpurProvider {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[derive(Debug, Deserialize)]
struct Project {
    versions: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct Version {
    builds: Builds,
}

#[derive(Debug, Deserialize)]
struct Builds {
    latest: String,
    all: Vec<String>,
}

#[async_trait]
impl ServerProvider for PurpurProvider {
    fn id(&self) -> ProviderId {
        ProviderId::Purpur
    }

    fn info(&self) -> ProviderInfo {
        info()
    }

    async fn list_minecraft_versions(&self) -> AppResult<Vec<MinecraftVersionInfo>> {
        let project: Project = get_json(&self.client, "https://api.purpurmc.org/v2/purpur").await?;
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

    async fn list_builds(&self, minecraft_version: &str) -> AppResult<Vec<BuildInfo>> {
        let version: Version = get_json(
            &self.client,
            &format!("https://api.purpurmc.org/v2/purpur/{minecraft_version}"),
        )
        .await?;
        Ok(version
            .builds
            .all
            .into_iter()
            .rev()
            .map(|id| BuildInfo {
                recommended: id == version.builds.latest,
                label: if id == version.builds.latest {
                    format!("Build #{id} (latest)")
                } else {
                    format!("Build #{id}")
                },
                channel: "stable".into(),
                released: None,
                loader_version: None,
                installer_version: None,
                id,
            })
            .collect())
    }

    async fn resolve_download(
        &self,
        minecraft_version: &str,
        build: Option<&str>,
        _loader_version: Option<&str>,
        _installer_version: Option<&str>,
    ) -> AppResult<ArtifactDownload> {
        let builds = self.list_builds(minecraft_version).await?;
        let selected = build
            .and_then(|b| builds.iter().find(|item| item.id == b))
            .or_else(|| builds.iter().find(|b| b.recommended))
            .or_else(|| builds.first())
            .ok_or_else(|| {
                AppError::new(
                    "Build unavailable",
                    format!("No Purpur build was found for Minecraft {minecraft_version}."),
                )
            })?;
        Ok(ArtifactDownload {
            url: format!(
                "https://api.purpurmc.org/v2/purpur/{minecraft_version}/{}/download",
                selected.id
            ),
            file_name: format!("purpur-{}-{}.jar", minecraft_version, selected.id),
            sha256: None,
            sha1: None,
            size: None,
            kind: "server".into(),
        })
    }
}
