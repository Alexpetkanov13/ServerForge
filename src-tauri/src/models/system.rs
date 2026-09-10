use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemInfo {
    pub total_memory_mb: u64,
    pub available_memory_mb: u64,
    pub cpu_count: usize,
    pub os: String,
    pub arch: String,
    pub app_version: String,
    pub offline: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskInfo {
    pub path: String,
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub writable: bool,
    pub contains_server: bool,
    pub empty: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerMetrics {
    pub server_id: String,
    pub cpu_percent: f32,
    pub memory_bytes: u64,
    pub memory_max_bytes: u64,
    pub uptime_seconds: u64,
    pub player_count: Option<u32>,
    pub max_players: Option<u32>,
    pub tps: Option<f32>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldInfo {
    pub name: String,
    pub path: String,
    pub size_bytes: u64,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub default_servers_dir: String,
    pub preferred_java_path: Option<String>,
    pub theme: String,
    pub reduced_motion: bool,
    pub font_scale: f32,
    pub developer_mode: bool,
    pub backup_retention: u32,
    pub auto_updates: bool,
    pub onboarded: bool,
    pub app_auto_update: bool,
    pub app_auto_install: bool,
    pub skipped_app_version: Option<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            default_servers_dir: default_servers_dir(),
            preferred_java_path: None,
            theme: "dark".into(),
            reduced_motion: false,
            font_scale: 1.0,
            developer_mode: false,
            backup_retention: 10,
            auto_updates: false,
            onboarded: false,
            app_auto_update: true,
            app_auto_install: false,
            skipped_app_version: None,
        }
    }
}

pub fn default_servers_dir() -> String {
    dirs::document_dir()
        .or_else(dirs::home_dir)
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("Minecraft")
        .join("Servers")
        .to_string_lossy()
        .to_string()
}
