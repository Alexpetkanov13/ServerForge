use async_trait::async_trait;
use reqwest::Client;
use std::path::Path;
use tokio::process::Command;

use crate::error::{AppError, AppResult};
use crate::models::{
    ArtifactDownload, BuildInfo, Ecosystem, MinecraftVersionInfo, ProviderId, ProviderInfo,
};
use crate::utils::java_version::{channel_for_version, recommended_java_major};

use super::ServerProvider;
use super::vanilla::VanillaProvider;

pub fn info() -> ProviderInfo {
    ProviderInfo {
        id: "spigot".into(),
        name: "Spigot".into(),
        description: "Classic Bukkit-compatible server built with official BuildTools.".into(),
        recommended_use: "Legacy plugin servers that specifically require Spigot.".into(),
        ecosystem: Ecosystem::Bukkit,
        supports_plugins: true,
        supports_mods: false,
        performance: "Good".into(),
    }
}

const BUILDTOOLS: &str =
    "https://hub.spigotmc.org/jenkins/job/BuildTools/lastSuccessfulBuild/artifact/target/BuildTools.jar";

pub struct SpigotProvider {
    client: Client,
    vanilla: VanillaProvider,
}

impl SpigotProvider {
    pub fn new(client: Client) -> Self {
        Self {
            vanilla: VanillaProvider::new(client.clone()),
            client,
        }
    }
}

#[async_trait]
impl ServerProvider for SpigotProvider {
    fn id(&self) -> ProviderId {
        ProviderId::Spigot
    }

    fn info(&self) -> ProviderInfo {
        info()
    }

    async fn list_minecraft_versions(&self) -> AppResult<Vec<MinecraftVersionInfo>> {
        let mut versions = self.vanilla.list_minecraft_versions().await?;
        versions.retain(|v| v.channel != "snapshot");
        Ok(versions)
    }

    async fn list_builds(&self, minecraft_version: &str) -> AppResult<Vec<BuildInfo>> {
        Ok(vec![BuildInfo {
            id: minecraft_version.to_string(),
            label: format!("BuildTools ({minecraft_version})"),
            channel: channel_for_version(minecraft_version).into(),
            recommended: true,
            released: None,
            loader_version: None,
            installer_version: None,
        }])
    }

    async fn resolve_download(
        &self,
        _minecraft_version: &str,
        _build: Option<&str>,
        _loader_version: Option<&str>,
        _installer_version: Option<&str>,
    ) -> AppResult<ArtifactDownload> {
        Ok(ArtifactDownload {
            url: BUILDTOOLS.to_string(),
            file_name: "BuildTools.jar".into(),
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
        minecraft_version: &str,
    ) -> AppResult<()> {
        self.post_install_with_version(server_dir, java, artifact, minecraft_version)
            .await
    }

    fn recommended_java(&self, minecraft_version: &str) -> u32 {
        recommended_java_major(minecraft_version)
    }
}

impl SpigotProvider {
    pub async fn post_install_with_version(
        &self,
        server_dir: &Path,
        java: &Path,
        artifact: &Path,
        version: &str,
    ) -> AppResult<()> {
        if which::which("git").is_err() {
            return Err(AppError::new(
                "Git is required for Spigot",
                "Spigot is compiled locally with official BuildTools, which requires Git.",
            )
            .with_causes(vec![
                "Install Git and restart ServerForge",
                "Or create a Paper server instead — it is plugin-compatible and faster to install",
            ]));
        }
        let work = server_dir.join(".buildtools");
        tokio::fs::create_dir_all(&work).await?;
        let copied = work.join("BuildTools.jar");
        tokio::fs::copy(artifact, &copied).await?;
        let mut cmd = Command::new(java);
        cmd.current_dir(&work)
            .arg("-jar")
            .arg(&copied)
            .arg("--rev")
            .arg(version)
            .arg("--compile")
            .arg("SPIGOT");
        crate::utils::process::capture_hidden(&mut cmd);
        let output = cmd.output().await?;
        if !output.status.success() {
            let log = String::from_utf8_lossy(&output.stderr);
            return Err(AppError::new(
                "Spigot build failed",
                "BuildTools did not finish successfully.",
            )
            .with_technical(format!("exit={:?}\n{log}", output.status)));
        }
        let mut found = None;
        let mut rd = tokio::fs::read_dir(&work).await?;
        while let Some(entry) = rd.next_entry().await? {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("spigot-") && name.ends_with(".jar") {
                found = Some(entry.path());
                break;
            }
        }
        let src = found.ok_or_else(|| {
            AppError::new(
                "Spigot build failed",
                "BuildTools finished but no spigot jar was produced.",
            )
        })?;
        tokio::fs::copy(&src, server_dir.join("server.jar")).await?;
        Ok(())
    }
}
