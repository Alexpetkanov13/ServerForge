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

const MANIFEST: &str = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";

pub fn info() -> ProviderInfo {
    ProviderInfo {
        id: "vanilla".into(),
        name: "Vanilla".into(),
        description: "Official unmodified Minecraft server from Mojang.".into(),
        recommended_use: "Pure survival, snapshots, and vanilla-only communities.".into(),
        ecosystem: Ecosystem::Vanilla,
        supports_plugins: false,
        supports_mods: false,
        performance: "Baseline".into(),
    }
}

pub struct VanillaProvider {
    client: Client,
}

impl VanillaProvider {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[derive(Debug, Deserialize)]
struct Manifest {
    latest: Latest,
    versions: Vec<VersionEntry>,
}

#[derive(Debug, Deserialize)]
struct Latest {
    release: String,
}

#[derive(Debug, Deserialize)]
struct VersionEntry {
    id: String,
    #[serde(rename = "type")]
    kind: String,
    url: String,
    #[serde(rename = "releaseTime")]
    release_time: Option<String>,
}

#[derive(Debug, Deserialize)]
struct VersionMeta {
    downloads: Downloads,
    #[serde(rename = "javaVersion")]
    java_version: Option<JavaVersion>,
}

#[derive(Debug, Deserialize)]
struct Downloads {
    server: Option<ServerDownload>,
}

#[derive(Debug, Deserialize)]
struct ServerDownload {
    url: String,
    sha1: Option<String>,
    size: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct JavaVersion {
    #[serde(rename = "majorVersion")]
    major_version: Option<u32>,
}

#[async_trait]
impl ServerProvider for VanillaProvider {
    fn id(&self) -> ProviderId {
        ProviderId::Vanilla
    }

    fn info(&self) -> ProviderInfo {
        info()
    }

    async fn list_minecraft_versions(&self) -> AppResult<Vec<MinecraftVersionInfo>> {
        let manifest: Manifest = get_json(&self.client, MANIFEST).await?;
        let latest = manifest.latest.release.clone();
        Ok(manifest
            .versions
            .into_iter()
            .filter(|v| v.kind == "release" || v.kind == "snapshot")
            .map(|v| MinecraftVersionInfo {
                latest: v.id == latest,
                recommended: v.id == latest && v.kind == "release",
                java_major: recommended_java_major(&v.id),
                channel: if v.kind == "snapshot" {
                    "snapshot".into()
                } else {
                    channel_for_version(&v.id).into()
                },
                released: v.release_time,
                id: v.id,
            })
            .collect())
    }

    async fn list_builds(&self, minecraft_version: &str) -> AppResult<Vec<BuildInfo>> {
        Ok(vec![BuildInfo {
            id: minecraft_version.to_string(),
            label: "Official server jar".into(),
            channel: "stable".into(),
            recommended: true,
            released: None,
            loader_version: None,
            installer_version: None,
        }])
    }

    async fn resolve_download(
        &self,
        minecraft_version: &str,
        _build: Option<&str>,
        _loader_version: Option<&str>,
        _installer_version: Option<&str>,
    ) -> AppResult<ArtifactDownload> {
        let manifest: Manifest = get_json(&self.client, MANIFEST).await?;
        let entry = manifest
            .versions
            .iter()
            .find(|v| v.id == minecraft_version)
            .ok_or_else(|| {
                AppError::new(
                    "Version unavailable",
                    format!("Minecraft {minecraft_version} is not in the Mojang version manifest."),
                )
            })?;
        let meta: VersionMeta = get_json(&self.client, &entry.url).await?;
        let server = meta.downloads.server.ok_or_else(|| {
            AppError::new(
                "Version unavailable",
                format!("Mojang does not publish a dedicated server jar for {minecraft_version}."),
            )
        })?;
        Ok(ArtifactDownload {
            url: server.url,
            file_name: "server.jar".into(),
            sha256: None,
            sha1: server.sha1,
            size: server.size,
            kind: "server".into(),
        })
    }

    fn recommended_java(&self, minecraft_version: &str) -> u32 {
        recommended_java_major(minecraft_version)
    }
}
