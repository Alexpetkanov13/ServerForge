use futures_util::StreamExt;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;
use tokio::io::AsyncWriteExt;
use tokio::sync::watch;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::DownloadProgress;
use crate::utils::hash::{sha1_file, sha256_file, verify_hash};

#[derive(Clone)]
pub struct DownloadService {
    inner: Arc<Mutex<HashMap<String, DownloadProgress>>>,
}

impl DownloadService {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn list(&self) -> Vec<DownloadProgress> {
        self.inner.lock().values().cloned().collect()
    }

    pub fn get(&self, id: &str) -> Option<DownloadProgress> {
        self.inner.lock().get(id).cloned()
    }

    pub async fn download_file(
        &self,
        client: &reqwest::Client,
        url: &str,
        dest: &Path,
        label: &str,
        sha256: Option<&str>,
        sha1: Option<&str>,
        on_progress: impl Fn(DownloadProgress),
    ) -> AppResult<PathBuf> {
        let id = Uuid::new_v4().to_string();
        if let Some(parent) = dest.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        let mut progress = DownloadProgress {
            id: id.clone(),
            label: label.to_string(),
            status: "downloading".into(),
            bytes_downloaded: 0,
            bytes_total: None,
            speed_bps: 0,
            eta_seconds: None,
            error: None,
        };
        self.inner.lock().insert(id.clone(), progress.clone());
        on_progress(progress.clone());

        let response = client.get(url).send().await?;
        if !response.status().is_success() {
            progress.status = "error".into();
            progress.error = Some(format!("HTTP {}", response.status()));
            self.inner.lock().insert(id.clone(), progress.clone());
            return Err(AppError::new(
                "Unable to download server software",
                "The download request was rejected by the provider.",
            )
            .with_causes(vec![
                "Internet connection unavailable",
                "Provider API unavailable",
                "Selected version is unavailable",
            ])
            .with_technical(format!("{} -> {}", url, response.status())));
        }
        progress.bytes_total = response.content_length();
        let mut file = tokio::fs::File::create(dest).await?;
        let mut stream = response.bytes_stream();
        let started = Instant::now();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;
            progress.bytes_downloaded += chunk.len() as u64;
            let elapsed = started.elapsed().as_secs_f64().max(0.05);
            progress.speed_bps = (progress.bytes_downloaded as f64 / elapsed) as u64;
            if let Some(total) = progress.bytes_total {
                if progress.speed_bps > 0 {
                    progress.eta_seconds = Some(total.saturating_sub(progress.bytes_downloaded) / progress.speed_bps);
                }
            }
            self.inner.lock().insert(id.clone(), progress.clone());
            on_progress(progress.clone());
        }
        file.flush().await?;
        drop(file);

        if let Some(expected) = sha256 {
            let actual = sha256_file(dest).await?;
            verify_hash(&actual, expected)?;
        } else if let Some(expected) = sha1 {
            let actual = sha1_file(dest).await?;
            verify_hash(&actual, expected)?;
        }

        progress.status = "completed".into();
        progress.eta_seconds = Some(0);
        self.inner.lock().insert(id, progress.clone());
        on_progress(progress);
        Ok(dest.to_path_buf())
    }
}

pub fn progress_channel() -> (watch::Sender<DownloadProgress>, watch::Receiver<DownloadProgress>) {
    watch::channel(DownloadProgress {
        id: String::new(),
        label: String::new(),
        status: "idle".into(),
        bytes_downloaded: 0,
        bytes_total: None,
        speed_bps: 0,
        eta_seconds: None,
        error: None,
    })
}
