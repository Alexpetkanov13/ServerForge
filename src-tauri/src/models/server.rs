use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ServerRecord {
    pub id: String,
    pub name: String,
    pub path: String,
    pub provider: String,
    pub minecraft_version: String,
    pub loader_version: Option<String>,
    pub build: Option<String>,
    pub java_path: Option<String>,
    pub memory_min_mb: i64,
    pub memory_max_mb: i64,
    pub port: i64,
    pub status: String,
    pub auto_start: i64,
    pub eula_accepted: i64,
    pub motd: Option<String>,
    pub pid: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
    pub last_started_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ServerSettingsRecord {
    pub jvm_args: Option<String>,
    pub optimized_flags: bool,
    pub backup_retention: i64,
    pub auto_backup: bool,
    pub backup_interval_minutes: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ServerStatus {
    Offline,
    Starting,
    Online,
    Stopping,
    Crashed,
    Installing,
    Updating,
    Error,
}

impl ServerStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Offline => "offline",
            Self::Starting => "starting",
            Self::Online => "online",
            Self::Stopping => "stopping",
            Self::Crashed => "crashed",
            Self::Installing => "installing",
            Self::Updating => "updating",
            Self::Error => "error",
        }
    }

    pub fn parse(value: &str) -> Self {
        match value {
            "starting" => Self::Starting,
            "online" => Self::Online,
            "stopping" => Self::Stopping,
            "crashed" => Self::Crashed,
            "installing" => Self::Installing,
            "updating" => Self::Updating,
            "error" => Self::Error,
            _ => Self::Offline,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateServerRequest {
    pub name: String,
    pub path: String,
    pub provider: String,
    pub minecraft_version: String,
    pub build: Option<String>,
    pub loader_version: Option<String>,
    pub installer_version: Option<String>,
    pub java_path: Option<String>,
    pub memory_min_mb: u32,
    pub memory_max_mb: u32,
    pub port: u16,
    pub motd: Option<String>,
    pub eula_accepted: bool,
    pub start_after_install: bool,
    pub generate_world: bool,
    pub optimized_flags: bool,
    pub difficulty: Option<String>,
    pub gamemode: Option<String>,
    pub max_players: Option<u32>,
    pub online_mode: Option<bool>,
    pub pvp: Option<bool>,
    pub view_distance: Option<u32>,
    pub simulation_distance: Option<u32>,
    pub allow_flight: Option<bool>,
    pub template_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SharePackManifest {
    pub format: String,
    pub format_version: u32,
    pub name: String,
    pub provider: String,
    pub minecraft_version: String,
    pub loader_version: Option<String>,
    pub build: Option<String>,
    pub memory_min_mb: i64,
    pub memory_max_mb: i64,
    pub port: i64,
    pub motd: Option<String>,
    pub eula_accepted: bool,
    pub auto_start: bool,
    pub exported_at: String,
}

impl SharePackManifest {
    pub fn from_server(server: &ServerRecord) -> Self {
        Self {
            format: "serverforge.server".into(),
            format_version: 1,
            name: server.name.clone(),
            provider: server.provider.clone(),
            minecraft_version: server.minecraft_version.clone(),
            loader_version: server.loader_version.clone(),
            build: server.build.clone(),
            memory_min_mb: server.memory_min_mb,
            memory_max_mb: server.memory_max_mb,
            port: server.port,
            motd: server.motd.clone(),
            eula_accepted: server.eula_accepted == 1,
            auto_start: server.auto_start == 1,
            exported_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SharePackInfo {
    pub path: String,
    pub size_bytes: i64,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveredServer {
    pub path: String,
    pub name: String,
    pub provider: Option<String>,
    pub minecraft_version: Option<String>,
    pub plugin_count: u32,
    pub mod_count: u32,
    pub world_count: u32,
    pub has_eula: bool,
    pub port: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerLogLine {
    pub server_id: String,
    pub stream: String,
    pub line: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrashInfo {
    pub server_id: String,
    pub reason: String,
    pub suggestion: String,
    pub report_path: Option<String>,
}
