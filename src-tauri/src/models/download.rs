use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadProgress {
    pub id: String,
    pub label: String,
    pub status: String,
    pub bytes_downloaded: u64,
    pub bytes_total: Option<u64>,
    pub speed_bps: u64,
    pub eta_seconds: Option<u64>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallationProgress {
    pub id: String,
    pub server_id: Option<String>,
    pub step: String,
    pub step_index: u32,
    pub step_count: u32,
    pub status: String,
    pub message: String,
    pub percent: u32,
    pub logs: Vec<String>,
    pub error: Option<crate::error::AppError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddonInfo {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub description: String,
    pub author: String,
    pub downloads: u64,
    pub icon_url: Option<String>,
    pub platform: String,
    pub source: String,
    #[serde(default)]
    pub game_versions: Vec<String>,
    #[serde(default)]
    pub compatible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddonVersion {
    pub id: String,
    pub name: String,
    pub version_number: String,
    pub game_versions: Vec<String>,
    pub loaders: Vec<String>,
    pub download_url: Option<String>,
    pub file_name: Option<String>,
    pub sha512: Option<String>,
    pub required_dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledAddon {
    pub name: String,
    pub file_name: String,
    pub enabled: bool,
    pub version: Option<String>,
    pub path: String,
}
