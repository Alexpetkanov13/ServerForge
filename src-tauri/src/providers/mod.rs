use async_trait::async_trait;
use std::path::Path;
use std::sync::Arc;

use crate::error::{AppError, AppResult};
use crate::models::{
    ArtifactDownload, BuildInfo, CompatibilityInfo, MinecraftVersionInfo, ProviderId, ProviderInfo,
};
use crate::utils::java_version::recommended_java_major;

mod fabric;
mod folia;
mod forge;
mod neoforge;
mod paper;
mod purpur;
mod spigot;
mod vanilla;

#[async_trait]
pub trait ServerProvider: Send + Sync {
    fn id(&self) -> ProviderId;
    fn info(&self) -> ProviderInfo;
    async fn list_minecraft_versions(&self) -> AppResult<Vec<MinecraftVersionInfo>>;
    async fn list_builds(&self, minecraft_version: &str) -> AppResult<Vec<BuildInfo>>;
    async fn resolve_download(
        &self,
        minecraft_version: &str,
        build: Option<&str>,
        loader_version: Option<&str>,
        installer_version: Option<&str>,
    ) -> AppResult<ArtifactDownload>;
    fn recommended_java(&self, minecraft_version: &str) -> u32 {
        recommended_java_major(minecraft_version)
    }
    fn launch_jar_name(&self) -> &'static str {
        "server.jar"
    }
    async fn post_install(
        &self,
        _server_dir: &Path,
        _java: &Path,
        _artifact: &Path,
        _minecraft_version: &str,
    ) -> AppResult<()> {
        Ok(())
    }
    fn validate_installation(&self, server_dir: &Path) -> AppResult<()> {
        let jar = server_dir.join(self.launch_jar_name());
        if jar.exists() {
            Ok(())
        } else {
            Err(AppError::new(
                "Installation incomplete",
                format!("Missing {} in the server folder.", self.launch_jar_name()),
            ))
        }
    }
    fn compatibility(&self, minecraft_version: &str, has_stable: bool) -> CompatibilityInfo {
        CompatibilityInfo {
            provider: self.id().as_str().to_string(),
            minecraft_version: minecraft_version.to_string(),
            supported: true,
            stable_build: has_stable,
            java_major: self.recommended_java(minecraft_version),
            notes: vec![],
        }
    }
}

pub fn all_providers(client: reqwest::Client) -> Vec<Arc<dyn ServerProvider>> {
    vec![
        Arc::new(vanilla::VanillaProvider::new(client.clone())),
        Arc::new(paper::PaperFamilyProvider::paper(client.clone())),
        Arc::new(spigot::SpigotProvider::new(client.clone())),
        Arc::new(purpur::PurpurProvider::new(client.clone())),
        Arc::new(folia::FoliaProvider::new(client.clone())),
        Arc::new(fabric::FabricProvider::new(client.clone())),
        Arc::new(forge::ForgeProvider::new(client.clone())),
        Arc::new(neoforge::NeoForgeProvider::new(client.clone())),
    ]
}

pub fn get_provider(client: reqwest::Client, id: &str) -> AppResult<Arc<dyn ServerProvider>> {
    let parsed = ProviderId::parse(id).ok_or_else(|| {
        AppError::new(
            "Unknown server platform",
            format!("'{id}' is not a supported Minecraft server platform."),
        )
    })?;
    all_providers(client)
        .into_iter()
        .find(|p| p.id() == parsed)
        .ok_or_else(|| AppError::message("Provider not registered"))
}

pub fn catalog() -> Vec<ProviderInfo> {
    // Static catalog so the UI can render immediately without a network round-trip.
    vec![
        vanilla::info(),
        paper::info(),
        spigot::info(),
        purpur::info(),
        folia::info(),
        fabric::info(),
        forge::info(),
        neoforge::info(),
    ]
}
