use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum ProviderId {
    Vanilla,
    Paper,
    Spigot,
    Purpur,
    Folia,
    Fabric,
    Forge,
    NeoForge,
}

impl ProviderId {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Vanilla => "vanilla",
            Self::Paper => "paper",
            Self::Spigot => "spigot",
            Self::Purpur => "purpur",
            Self::Folia => "folia",
            Self::Fabric => "fabric",
            Self::Forge => "forge",
            Self::NeoForge => "neoforge",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.to_ascii_lowercase().as_str() {
            "vanilla" => Some(Self::Vanilla),
            "paper" => Some(Self::Paper),
            "spigot" => Some(Self::Spigot),
            "purpur" => Some(Self::Purpur),
            "folia" => Some(Self::Folia),
            "fabric" => Some(Self::Fabric),
            "forge" => Some(Self::Forge),
            "neoforge" => Some(Self::NeoForge),
            _ => None,
        }
    }

    pub fn all() -> &'static [ProviderId] {
        &[
            Self::Vanilla,
            Self::Paper,
            Self::Spigot,
            Self::Purpur,
            Self::Folia,
            Self::Fabric,
            Self::Forge,
            Self::NeoForge,
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Ecosystem {
    Vanilla,
    Bukkit,
    Modded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub recommended_use: String,
    pub ecosystem: Ecosystem,
    pub supports_plugins: bool,
    pub supports_mods: bool,
    pub performance: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MinecraftVersionInfo {
    pub id: String,
    pub channel: String,
    pub recommended: bool,
    pub latest: bool,
    pub java_major: u32,
    pub released: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildInfo {
    pub id: String,
    pub label: String,
    pub channel: String,
    pub recommended: bool,
    pub released: Option<String>,
    pub loader_version: Option<String>,
    pub installer_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactDownload {
    pub url: String,
    pub file_name: String,
    pub sha256: Option<String>,
    pub sha1: Option<String>,
    pub size: Option<u64>,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompatibilityInfo {
    pub provider: String,
    pub minecraft_version: String,
    pub supported: bool,
    pub stable_build: bool,
    pub java_major: u32,
    pub notes: Vec<String>,
}
