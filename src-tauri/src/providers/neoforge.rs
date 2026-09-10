use async_trait::async_trait;
use reqwest::Client;
use std::path::Path;
use tokio::process::Command;

use crate::error::{AppError, AppResult};
use crate::models::{
    ArtifactDownload, BuildInfo, Ecosystem, MinecraftVersionInfo, ProviderId, ProviderInfo,
};
use crate::utils::http::get_text;
use crate::utils::java_version::recommended_java_major;

use super::ServerProvider;

pub fn info() -> ProviderInfo {
    ProviderInfo {
        id: "neoforge".into(),
        name: "NeoForge".into(),
        description: "Modern Forge successor with an active 1.20.5+ ecosystem.".into(),
        recommended_use: "Current Forge-style modpacks on newer Minecraft versions.".into(),
        ecosystem: Ecosystem::Modded,
        supports_plugins: false,
        supports_mods: true,
        performance: "Good".into(),
    }
}

const METADATA: &str =
    "https://maven.neoforged.net/releases/net/neoforged/neoforge/maven-metadata.xml";

pub struct NeoForgeProvider {
    client: Client,
}

impl NeoForgeProvider {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    fn parse_versions(xml: &str) -> Vec<String> {
        xml.split("<version>")
            .skip(1)
            .filter_map(|chunk| chunk.split("</version>").next().map(|s| s.trim().to_string()))
            .filter(|s| !s.is_empty())
            .collect()
    }

    fn mc_from_neoforge(version: &str) -> String {
        // NeoForge versions are typically 21.1.x for Minecraft 1.21.1
        let mut parts = version.split('.');
        let major = parts.next().unwrap_or("21");
        let minor = parts.next().unwrap_or("0");
        if major == "20" {
            format!("1.20.{minor}")
        } else {
            format!("1.{major}.{minor}")
        }
    }
}

#[async_trait]
impl ServerProvider for NeoForgeProvider {
    fn id(&self) -> ProviderId {
        ProviderId::NeoForge
    }

    fn info(&self) -> ProviderInfo {
        info()
    }

    async fn list_minecraft_versions(&self) -> AppResult<Vec<MinecraftVersionInfo>> {
        let xml = get_text(&self.client, METADATA).await?;
        let versions = Self::parse_versions(&xml);
        let mut mc_versions: Vec<String> = versions.iter().map(|v| Self::mc_from_neoforge(v)).collect();
        mc_versions.sort();
        mc_versions.dedup();
        mc_versions.reverse();
        let latest = mc_versions.first().cloned().unwrap_or_default();
        Ok(mc_versions
            .into_iter()
            .map(|id| MinecraftVersionInfo {
                latest: id == latest,
                recommended: id == latest,
                java_major: recommended_java_major(&id),
                channel: "stable".into(),
                released: None,
                id,
            })
            .collect())
    }

    async fn list_builds(&self, minecraft_version: &str) -> AppResult<Vec<BuildInfo>> {
        let xml = get_text(&self.client, METADATA).await?;
        let versions = Self::parse_versions(&xml);
        let mut builds: Vec<BuildInfo> = versions
            .into_iter()
            .filter(|v| Self::mc_from_neoforge(v) == minecraft_version)
            .map(|id| BuildInfo {
                label: format!("NeoForge {id}"),
                loader_version: Some(id.clone()),
                installer_version: Some(id.clone()),
                channel: "stable".into(),
                recommended: false,
                released: None,
                id,
            })
            .collect();
        builds.reverse();
        if let Some(first) = builds.first_mut() {
            first.recommended = true;
            first.label = format!("{} (recommended)", first.label);
        }
        if builds.is_empty() {
            return Err(AppError::new(
                "Version unavailable",
                format!("No NeoForge installer was published for Minecraft {minecraft_version}."),
            ));
        }
        Ok(builds)
    }

    async fn resolve_download(
        &self,
        minecraft_version: &str,
        build: Option<&str>,
        loader_version: Option<&str>,
        _installer_version: Option<&str>,
    ) -> AppResult<ArtifactDownload> {
        let builds = self.list_builds(minecraft_version).await?;
        let selected = build
            .or(loader_version)
            .and_then(|id| builds.iter().find(|b| b.id == id))
            .or_else(|| builds.iter().find(|b| b.recommended))
            .or_else(|| builds.first())
            .ok_or_else(|| AppError::message("NeoForge version not found"))?;
        Ok(ArtifactDownload {
            url: format!(
                "https://maven.neoforged.net/releases/net/neoforged/neoforge/{ver}/neoforge-{ver}-installer.jar",
                ver = selected.id
            ),
            file_name: format!("neoforge-{}-installer.jar", selected.id),
            sha256: None,
            sha1: None,
            size: None,
            kind: "installer".into(),
        })
    }

    async fn post_install(
        &self,
        server_dir: &Path,
        java: &Path,
        artifact: &Path,
        _minecraft_version: &str,
    ) -> AppResult<()> {
        let mut cmd = Command::new(java);
        cmd.current_dir(server_dir)
            .arg("-jar")
            .arg(artifact)
            .arg("--installServer");
        crate::utils::process::capture_hidden(&mut cmd);
        let output = cmd.output().await?;
        if !output.status.success() {
            let log = String::from_utf8_lossy(&output.stderr);
            return Err(AppError::new(
                "NeoForge installer failed",
                "The official NeoForge installer did not complete.",
            )
            .with_technical(format!("exit={:?}\n{log}", output.status)));
        }
        Ok(())
    }

    fn validate_installation(&self, server_dir: &Path) -> AppResult<()> {
        let win = server_dir.join("run.bat");
        let unix = server_dir.join("run.sh");
        if unix.exists() || win.exists() || server_dir.join("user_jvm_args.txt").exists() {
            Ok(())
        } else {
            Err(AppError::new(
                "Installation incomplete",
                "NeoForge server files were not found after the installer ran.",
            ))
        }
    }
}
