use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::error::{AppError, AppResult};
use crate::models::ServerRecord;
use crate::utils::path::{folder_name_from_server, share_pack_path};

pub async fn list_servers(db: &sqlx::SqlitePool) -> AppResult<Vec<ServerRecord>> {
    let rows = sqlx::query_as::<_, ServerRecord>("SELECT * FROM servers ORDER BY created_at DESC")
        .fetch_all(db)
        .await?;
    Ok(rows)
}

pub async fn get_server(db: &sqlx::SqlitePool, id: &str) -> AppResult<ServerRecord> {
    sqlx::query_as::<_, ServerRecord>("SELECT * FROM servers WHERE id = ?")
        .bind(id)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| crate::error::AppError::new("Server not found", "That server is no longer in ServerForge."))
}

pub async fn delete_server(db: &sqlx::SqlitePool, id: &str, delete_files: bool) -> AppResult<()> {
    let server = get_server(db, id).await?;
    let pack = share_pack_path(Path::new(&server.path), &server.name);
    if delete_files {
        let path = PathBuf::from(&server.path);
        if path.exists() {
            remove_dir_with_retry(&path).await?;
        }
        if pack.exists() {
            let _ = tokio::fs::remove_file(&pack).await;
        }
    }
    sqlx::query("DELETE FROM servers WHERE id = ?")
        .bind(id)
        .execute(db)
        .await?;
    Ok(())
}

async fn remove_dir_with_retry(path: &Path) -> AppResult<()> {
    let mut last = String::new();
    for attempt in 0..10 {
        match tokio::fs::remove_dir_all(path).await {
            Ok(()) => return Ok(()),
            Err(err) => {
                last = err.to_string();
                if !path.exists() {
                    return Ok(());
                }
                if attempt < 9 {
                    tokio::time::sleep(Duration::from_millis(250)).await;
                }
            }
        }
    }
    Err(AppError::new(
        "Could not delete server files",
        "The folder is still in use. Stop the server, close File Explorer windows on that folder, then try again.",
    )
    .with_technical(last))
}

pub async fn update_server_memory(db: &sqlx::SqlitePool, id: &str, min_mb: i64, max_mb: i64) -> AppResult<()> {
    sqlx::query("UPDATE servers SET memory_min_mb = ?, memory_max_mb = ?, updated_at = ? WHERE id = ?")
        .bind(min_mb)
        .bind(max_mb)
        .bind(chrono::Utc::now().to_rfc3339())
        .bind(id)
        .execute(db)
        .await?;
    Ok(())
}

pub async fn set_auto_start(db: &sqlx::SqlitePool, id: &str, auto_start: bool) -> AppResult<()> {
    sqlx::query("UPDATE servers SET auto_start = ?, updated_at = ? WHERE id = ?")
        .bind(if auto_start { 1 } else { 0 })
        .bind(chrono::Utc::now().to_rfc3339())
        .bind(id)
        .execute(db)
        .await?;
    Ok(())
}

pub async fn import_server(
    db: &sqlx::SqlitePool,
    discovered: crate::models::DiscoveredServer,
    java_path: Option<String>,
) -> AppResult<ServerRecord> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let name = unique_name(db, &discovered.name).await?;
    sqlx::query(
        "INSERT INTO servers (id, name, path, provider, minecraft_version, java_path, memory_min_mb, memory_max_mb, port, status, auto_start, eula_accepted, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, 1024, 4096, ?, 'offline', 0, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&name)
    .bind(&discovered.path)
    .bind(discovered.provider.unwrap_or_else(|| "vanilla".into()))
    .bind(discovered.minecraft_version.unwrap_or_else(|| "unknown".into()))
    .bind(&java_path)
    .bind(discovered.port.unwrap_or(25565) as i64)
    .bind(if discovered.has_eula { 1 } else { 0 })
    .bind(&now)
    .bind(&now)
    .execute(db)
    .await?;
    get_server(db, &id).await
}

pub(crate) async fn unique_name(db: &sqlx::SqlitePool, base: &str) -> AppResult<String> {
    let mut name = base.to_string();
    let mut i = 2;
    loop {
        let exists: Option<(String,)> = sqlx::query_as("SELECT id FROM servers WHERE name = ?")
            .bind(&name)
            .fetch_optional(db)
            .await?;
        if exists.is_none() {
            return Ok(name);
        }
        name = format!("{base} ({i})");
        i += 1;
    }
}

pub(crate) async fn unique_install_path(
    db: &sqlx::SqlitePool,
    parent: &Path,
    name: &str,
) -> AppResult<PathBuf> {
    let folder = folder_name_from_server(name);
    let mut dest = parent.join(&folder);
    let mut i = 2;
    loop {
        let path_str = dest.to_string_lossy().to_string();
        let on_disk = dest.exists();
        let in_db: Option<(String,)> = sqlx::query_as("SELECT id FROM servers WHERE path = ?")
            .bind(&path_str)
            .fetch_optional(db)
            .await?;
        if !on_disk && in_db.is_none() {
            return Ok(dest);
        }
        dest = parent.join(format!("{folder}-{i}"));
        i += 1;
        if i > 999 {
            return Err(crate::error::AppError::new(
                "Could not pick a folder",
                "Too many servers already use this name. Choose a different name or location.",
            ));
        }
    }
}
