use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use walkdir::WalkDir;
use zip::write::SimpleFileOptions;

use crate::error::{AppError, AppResult};
use crate::models::BackupRecord;
use crate::utils::path::is_path_inside;

pub async fn create_backup(
    db: &sqlx::SqlitePool,
    server_id: &str,
    server_path: &Path,
    trigger: &str,
    label: Option<String>,
) -> AppResult<BackupRecord> {
    let backups_dir = server_path.join("backups");
    tokio::fs::create_dir_all(&backups_dir).await?;
    let stamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
    let dest = backups_dir.join(format!("backup-{stamp}.zip"));
    let dest_clone = dest.clone();
    let src = server_path.to_path_buf();
    let size = tokio::task::spawn_blocking(move || zip_dir(&src, &dest_clone, &["backups", "cache", ".buildtools"]))
        .await
        .map_err(|e| AppError::message(e.to_string()))??;
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO backups (id, server_id, label, path, size_bytes, trigger, created_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(server_id)
    .bind(&label)
    .bind(dest.to_string_lossy().to_string())
    .bind(size as i64)
    .bind(trigger)
    .bind(&now)
    .execute(db)
    .await?;
    prune(db, server_id, 10).await?;
    Ok(BackupRecord {
        id,
        server_id: server_id.to_string(),
        label,
        path: dest.to_string_lossy().to_string(),
        size_bytes: size as i64,
        trigger: trigger.to_string(),
        created_at: now,
    })
}

pub async fn list_backups(db: &sqlx::SqlitePool, server_id: &str) -> AppResult<Vec<BackupRecord>> {
    Ok(sqlx::query_as::<_, BackupRecord>(
        "SELECT * FROM backups WHERE server_id = ? ORDER BY created_at DESC",
    )
    .bind(server_id)
    .fetch_all(db)
    .await?)
}

pub async fn restore_backup(server_path: &Path, zip_path: &Path) -> AppResult<()> {
    if !is_path_inside(server_path, zip_path) && !zip_path.starts_with(server_path.join("backups")) {
        // still allow restore from the backups folder even if canonicalize fails
    }
    let dest = server_path.to_path_buf();
    let src = zip_path.to_path_buf();
    tokio::task::spawn_blocking(move || unzip_to(&src, &dest))
        .await
        .map_err(|e| AppError::message(e.to_string()))??;
    Ok(())
}

pub async fn delete_backup(db: &sqlx::SqlitePool, id: &str) -> AppResult<()> {
    let row: Option<(String,)> = sqlx::query_as("SELECT path FROM backups WHERE id = ?")
        .bind(id)
        .fetch_optional(db)
        .await?;
    if let Some((path,)) = row {
        let _ = tokio::fs::remove_file(path).await;
    }
    sqlx::query("DELETE FROM backups WHERE id = ?")
        .bind(id)
        .execute(db)
        .await?;
    Ok(())
}

async fn prune(db: &sqlx::SqlitePool, server_id: &str, keep: i64) -> AppResult<()> {
    let extras: Vec<(String, String)> = sqlx::query_as(
        "SELECT id, path FROM backups WHERE server_id = ? ORDER BY created_at DESC LIMIT -1 OFFSET ?",
    )
    .bind(server_id)
    .bind(keep)
    .fetch_all(db)
    .await?;
    for (id, path) in extras {
        let _ = tokio::fs::remove_file(path).await;
        sqlx::query("DELETE FROM backups WHERE id = ?").bind(id).execute(db).await?;
    }
    Ok(())
}

fn zip_dir(src: &Path, dest: &Path, skip: &[&str]) -> AppResult<u64> {
    let file = File::create(dest)?;
    let mut zip = zip::ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    let src = src.canonicalize()?;
    for entry in WalkDir::new(&src).into_iter().flatten() {
        let path = entry.path();
        let rel = path.strip_prefix(&src).unwrap_or(path);
        if rel.components().any(|c| skip.iter().any(|s| c.as_os_str() == *s)) {
            continue;
        }
        if path.is_dir() {
            continue;
        }
        let name = rel.to_string_lossy().replace('\\', "/");
        zip.start_file(name, options)?;
        let mut f = File::open(path)?;
        let mut buf = Vec::new();
        f.read_to_end(&mut buf)?;
        zip.write_all(&buf)?;
    }
    zip.finish()?;
    Ok(std::fs::metadata(dest)?.len())
}

fn unzip_to(zip_path: &Path, dest: &Path) -> AppResult<()> {
    let file = File::open(zip_path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = match file.enclosed_name() {
            Some(p) => dest.join(p),
            None => continue,
        };
        if file.is_dir() {
            std::fs::create_dir_all(&outpath)?;
        } else {
            if let Some(parent) = outpath.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut outfile = File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }
    Ok(())
}
