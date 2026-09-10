use tauri::{AppHandle, Emitter};

use crate::services::process::metrics;
use crate::state::AppState;

pub fn spawn_metrics_loop(app: AppHandle, state: std::sync::Arc<AppState>) {
    tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            for entry in state.processes.iter() {
                let server_id = entry.key().clone();
                let handle = entry.value().clone();
                let max = sqlx::query_as::<_, (i64,)>("SELECT memory_max_mb FROM servers WHERE id = ?")
                    .bind(&server_id)
                    .fetch_optional(&state.db)
                    .await
                    .ok()
                    .flatten()
                    .map(|r| r.0)
                    .unwrap_or(4096);
                let payload = metrics(&handle, max);
                let _ = app.emit("server-metrics", payload);
            }
        }
    });
}
