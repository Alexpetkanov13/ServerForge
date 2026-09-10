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
use super::vanilla::VanillaProvider;

pub fn info() -> ProviderInfo {
    ProviderInfo {
        id: "forge".into(),
        name: "Forge".into(),
        description: "The classic Minecraft modding platform.".into(),
        recommended_use: "Large modpacks and Forge-only communities.".into(),
        ecosystem: Ecosystem::Modded,
        supports_plugins: false,
        supports_mods: true,
        performance: "Good".into(),
    }
}

const METADATA: &str =
    "https://maven.minecraftforge.net/net/minecraftforge/forge/maven-metadata.xml";

pub struct ForgeProvider {
    client: Client,
    vanilla: VanillaProvider,
}

impl ForgeProvider {
    pub fn new(client: Client) -> Self {
        Self {
            vanilla: VanillaProvider::new(client.clone()),
            client,
        }
    }

    fn parse_versions(xml: &str) -> Vec<String> {
        xml.split("<version>")
            .skip(1)
            .filter_map(|chunk| chunk.split("</version>").next().map(|s| s.trim().to_string()))
            .filter(|s| !s.is_empty())
            .collect()
    }
}

#[async_trait]
impl ServerProvider for ForgeProvider {
    fn id(&self) -> ProviderId {
        ProviderId::Forge
    }

    fn info(&self) -> ProviderInfo {
        info()
    }

    async fn list_minecraft_versions(&self) -> AppResult<Vec<MinecraftVersionInfo>> {
        let xml = get_text(&self.client, METADATA).await?;
        let versions = Self::parse_versions(&xml);
        let mut mc_versions: Vec<String> = versions
            .iter()
            .filter_map(|v| v.split('-').next().map(|s| s.to_string()))
            .collect();
        mc_versions.sort();
        mc_versions.dedup();
        mc_versions.reverse();
        if mc_versions.is_empty() {
            return self.vanilla.list_minecraft_versions().await;
        }
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
            .filter(|v| v.starts_with(&format!("{minecraft_version}-")))
            .map(|id| {
                let forge = id.split_once('-').map(|(_, rest)| rest.to_string());
                BuildInfo {
                    label: format!("Forge {}", forge.clone().unwrap_or_else(|| id.clone())),
                    loader_version: forge,
                    installer_version: Some(id.clone()),
                    channel: "stable".into(),
                    recommended: false,
                    released: None,
                    id,
                }
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
                format!("No Forge installer was published for Minecraft {minecraft_version}."),
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
            .and_then(|id| {
                builds.iter().find(|b| {
                    b.id == id || b.loader_version.as_deref() == Some(id)
                })
            })
            .or_else(|| builds.iter().find(|b| b.recommended))
            .or_else(|| builds.first())
            .ok_or_else(|| AppError::message("Forge version not found"))?;
        Ok(ArtifactDownload {
            url: format!(
                "https://maven.minecraftforge.net/net/minecraftforge/forge/{ver}/forge-{ver}-installer.jar",
                ver = selected.id
            ),
            file_name: format!("forge-{}-installer.jar", selected.id),
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
                "Forge installer failed",
                "The official Forge installer did not complete.",
            )
            .with_technical(format!("exit={:?}\n{log}", output.status)));
        }
        Ok(())
    }

    fn launch_jar_name(&self) -> &'static str {
        "run.bat"
    }

    fn validate_installation(&self, server_dir: &Path) -> AppResult<()> {
        let unix = server_dir.join("run.sh");
        let win = server_dir.join("run.bat");
        let forge_jar = std::fs::read_dir(server_dir).ok().and_then(|rd| {
            rd.flatten().find(|e| {
                let n = e.file_name().to_string_lossy().to_string();
                n.starts_with("forge-") && n.ends_with(".jar") && !n.contains("installer")
            })
        });
        if unix.exists() || win.exists() || forge_jar.is_some() || server_dir.join("user_jvm_args.txt").exists() {
            Ok(())
        } else {
            Err(AppError::new(
                "Installation incomplete",
                "Forge server files were not found after the installer ran.",
            ))
        }
    }
}
