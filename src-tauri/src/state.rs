use dashmap::DashMap;
use sqlx::SqlitePool;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::models::CreateServerRequest;
use crate::services::download::DownloadService;
use crate::services::process::ManagedServer;

pub struct AppState {
    pub db: SqlitePool,
    pub http: reqwest::Client,
    pub data_dir: std::path::PathBuf,
    pub logs_dir: std::path::PathBuf,
    pub processes: DashMap<String, Arc<ManagedServer>>,
    pub downloads: Arc<DownloadService>,
    pub installations: DashMap<String, Mutex<CreateServerRequest>>,
    pub share_gens: DashMap<String, u64>,
    pub share_locks: DashMap<String, Arc<Mutex<()>>>,
}

impl AppState {
    pub fn new(db: SqlitePool, http: reqwest::Client, data_dir: std::path::PathBuf, logs_dir: std::path::PathBuf) -> Self {
        Self {
            db,
            http,
            data_dir,
            logs_dir,
            processes: DashMap::new(),
            downloads: Arc::new(DownloadService::new()),
            installations: DashMap::new(),
            share_gens: DashMap::new(),
            share_locks: DashMap::new(),
        }
    }
}
