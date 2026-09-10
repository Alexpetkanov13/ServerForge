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

const GAME: &str = "https://meta.fabricmc.net/v2/versions/game";
const LOADER: &str = "https://meta.fabricmc.net/v2/versions/loader";
const INSTALLER: &str = "https://meta.fabricmc.net/v2/versions/installer";

pub fn info() -> ProviderInfo {
    ProviderInfo {
        id: "fabric".into(),
        name: "Fabric".into(),
        description: "Lightweight mod loader with a fast update cadence.".into(),
        recommended_use: "Modded servers that want current Minecraft versions quickly.".into(),
        ecosystem: Ecosystem::Modded,
        supports_plugins: false,
        supports_mods: true,
        performance: "Excellent".into(),
    }
}

pub struct FabricProvider {
    client: Client,
}

impl FabricProvider {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[derive(Debug, Deserialize)]
struct GameVersion {
    version: String,
    stable: bool,
}

#[derive(Debug, Deserialize)]
struct LoaderVersion {
    version: String,
    stable: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct InstallerVersion {
    version: String,
    stable: Option<bool>,
}

#[async_trait]
impl ServerProvider for FabricProvider {
    fn id(&self) -> ProviderId {
        ProviderId::Fabric
    }

    fn info(&self) -> ProviderInfo {
        info()
    }

    async fn list_minecraft_versions(&self) -> AppResult<Vec<MinecraftVersionInfo>> {
        let versions: Vec<GameVersion> = get_json(&self.client, GAME).await?;
        let latest = versions
            .iter()
            .find(|v| v.stable)
            .map(|v| v.version.clone())
            .unwrap_or_default();
        Ok(versions
            .into_iter()
            .map(|v| MinecraftVersionInfo {
                latest: v.version == latest,
                recommended: v.stable && v.version == latest,
                java_major: recommended_java_major(&v.version),
                channel: if v.stable {
                    channel_for_version(&v.version).into()
                } else {
                    "snapshot".into()
                },
                released: None,
                id: v.version,
            })
            .collect())
    }

    async fn list_builds(&self, minecraft_version: &str) -> AppResult<Vec<BuildInfo>> {
        let loaders: Vec<LoaderVersion> = match get_json(
            &self.client,
            &format!("{LOADER}/{minecraft_version}"),
        )
        .await
        {
            Ok(v) => v,
            Err(_) => get_json(&self.client, LOADER).await?,
        };
        let installers: Vec<InstallerVersion> = get_json(&self.client, INSTALLER).await?;
        let installer = installers
            .iter()
            .find(|i| i.stable.unwrap_or(false))
            .or_else(|| installers.first())
            .ok_or_else(|| AppError::message("No Fabric installer versions were returned"))?;
        Ok(loaders
            .into_iter()
            .enumerate()
            .map(|(idx, loader)| BuildInfo {
                id: loader.version.clone(),
                label: format!("Loader {}", loader.version),
                channel: if loader.stable.unwrap_or(idx == 0) {
                    "stable".into()
                } else {
                    "latest".into()
                },
                recommended: loader.stable.unwrap_or(idx == 0) || idx == 0,
                released: None,
                loader_version: Some(loader.version),
                installer_version: Some(installer.version.clone()),
            })
            .collect())
    }

    async fn resolve_download(
        &self,
        minecraft_version: &str,
        build: Option<&str>,
        loader_version: Option<&str>,
        installer_version: Option<&str>,
    ) -> AppResult<ArtifactDownload> {
        let builds = self.list_builds(minecraft_version).await?;
        let selected = loader_version
            .or(build)
            .and_then(|id| builds.iter().find(|b| b.id == id || b.loader_version.as_deref() == Some(id)))
            .or_else(|| builds.iter().find(|b| b.recommended))
            .or_else(|| builds.first())
            .ok_or_else(|| {
                AppError::new(
                    "Loader unavailable",
                    format!("No Fabric loader was found for Minecraft {minecraft_version}."),
                )
            })?;
        let installer = installer_version
            .map(|s| s.to_string())
            .or_else(|| selected.installer_version.clone())
            .ok_or_else(|| AppError::message("Fabric installer version missing"))?;
        let loader = selected
            .loader_version
            .clone()
            .unwrap_or_else(|| selected.id.clone());
        Ok(ArtifactDownload {
            url: format!(
                "https://meta.fabricmc.net/v2/versions/loader/{minecraft_version}/{loader}/{installer}/server/jar"
            ),
            file_name: "server.jar".into(),
            sha256: None,
            sha1: None,
            size: None,
            kind: "server".into(),
        })
    }
}
