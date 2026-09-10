use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use walkdir::WalkDir;
use zip::write::SimpleFileOptions;

use crate::error::{AppError, AppResult};
use crate::models::{SharePackInfo, SharePackManifest, ServerRecord};
use crate::services::{servers, settings};
use crate::state::AppState;
use crate::utils::path::share_pack_path;

const FORMAT: &str = "serverforge.server";
const FORMAT_VERSION: u32 = 1;
const SKIP_DIRS: &[&str] = &["backups", "cache", ".buildtools", "logs", "crash-reports"];
const DEBOUNCE: Duration = Duration::from_secs(2);

pub fn schedule_share_pack(state: Arc<AppState>, server_id: String) {
    let next = {
        let mut entry = state.share_gens.entry(server_id.clone()).or_insert(0);
        *entry += 1;
        *entry
    };
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(DEBOUNCE).await;
        let current = state.share_gens.get(&server_id).map(|g| *g);
        if current != Some(next) {
            return;
        }
        if state.processes.contains_key(&server_id) {
            return;
        }
        if let Err(err) = write_share_pack(&state, &server_id).await {
            tracing::warn!(server_id = %server_id, error = %err, "share pack update failed");
        }
    });
}

pub async fn write_share_pack(state: &AppState, server_id: &str) -> AppResult<PathBuf> {
    let server = servers::get_server(&state.db, server_id).await?;
    let dest = share_pack_path(Path::new(&server.path), &server.name);
    let lock = state
        .share_locks
        .entry(server_id.to_string())
        .or_insert_with(|| Arc::new(tokio::sync::Mutex::new(())))
        .clone();
    let _guard = lock.lock().await;
    let src = PathBuf::from(&server.path);
    if !src.is_dir() {
        return Err(AppError::new(
            "Server folder missing",
            "The server folder is gone, so the shareable file could not be written.",
        ));
    }
    let dest_clone = dest.clone();
    let manifest = SharePackManifest::from_server(&server);
    tokio::task::spawn_blocking(move || write_pack(&src, &dest_clone, &manifest))
        .await
        .map_err(|e| AppError::message(e.to_string()))??;
    Ok(dest)
}

pub async fn ensure_share_pack(state: &AppState, server_id: &str) -> AppResult<SharePackInfo> {
    let server = servers::get_server(&state.db, server_id).await?;
    let dest = share_pack_path(Path::new(&server.path), &server.name);
    if !dest.exists() {
        if state.processes.contains_key(server_id) {
            return Err(AppError::new(
                "Share file not ready",
                "Stop the server once so ServerForge can create the shareable .server file.",
            ));
        }
        write_share_pack(state, server_id).await?;
    }
    pack_info(&dest)
}

pub async fn import_share_pack(
    state: &AppState,
    pack_path: &Path,
    dest_dir: Option<PathBuf>,
) -> AppResult<ServerRecord> {
    if !pack_path.exists() {
        return Err(AppError::new(
            "File not found",
            "That .server file could not be found.",
        ));
    }
    let pack = pack_path.to_path_buf();
    let manifest = tokio::task::spawn_blocking(move || read_manifest(&pack))
        .await
        .map_err(|e| AppError::message(e.to_string()))??;

    let settings = settings::load(&state.db).await?;
    let parent = dest_dir.unwrap_or_else(|| PathBuf::from(&settings.default_servers_dir));
    tokio::fs::create_dir_all(&parent).await?;
    let name = servers::unique_name(&state.db, &manifest.name).await?;
    let dest = servers::unique_install_path(&state.db, &parent, &name).await?;
    tokio::fs::create_dir_all(&dest).await?;

    let src = pack_path.to_path_buf();
    let dest_clone = dest.clone();
    tokio::task::spawn_blocking(move || extract_pack(&src, &dest_clone))
        .await
        .map_err(|e| AppError::message(e.to_string()))??;

    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO servers (id, name, path, provider, minecraft_version, loader_version, build, java_path, memory_min_mb, memory_max_mb, port, status, auto_start, eula_accepted, motd, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'offline', 0, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&name)
    .bind(dest.to_string_lossy().to_string())
    .bind(&manifest.provider)
    .bind(&manifest.minecraft_version)
    .bind(&manifest.loader_version)
    .bind(&manifest.build)
    .bind(&settings.preferred_java_path)
    .bind(manifest.memory_min_mb)
    .bind(manifest.memory_max_mb)
    .bind(manifest.port)
    .bind(if manifest.eula_accepted { 1 } else { 0 })
    .bind(&manifest.motd)
    .bind(&now)
    .bind(&now)
    .execute(&state.db)
    .await?;

    if let Err(err) = write_share_pack(state, &id).await {
        tracing::warn!(error = %err, "imported server but could not write local share pack");
    }
    servers::get_server(&state.db, &id).await
}

pub async fn server_id_for_child_path(db: &sqlx::SqlitePool, child: &Path) -> AppResult<Option<String>> {
    let servers = servers::list_servers(db).await?;
    let child_key = normalize_path_key(child);
    Ok(servers.into_iter().find_map(|server| {
        let parent_key = normalize_path_key(Path::new(&server.path));
        if child_key.starts_with(&parent_key) {
            Some(server.id)
        } else {
            None
        }
    }))
}

fn normalize_path_key(path: &Path) -> String {
    path.to_string_lossy().replace('/', "\\").trim_end_matches('\\').to_lowercase()
}

fn pack_info(path: &Path) -> AppResult<SharePackInfo> {
    let meta = std::fs::metadata(path)?;
    let updated_at = meta
        .modified()
        .ok()
        .map(chrono::DateTime::<chrono::Utc>::from)
        .map(|t| t.to_rfc3339())
        .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());
    Ok(SharePackInfo {
        path: path.to_string_lossy().to_string(),
        size_bytes: meta.len() as i64,
        updated_at,
    })
}

fn write_pack(src: &Path, dest: &Path, manifest: &SharePackManifest) -> AppResult<u64> {
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let tmp = dest.with_file_name(format!(
        "{}.tmp",
        dest.file_name().unwrap_or_default().to_string_lossy()
    ));
    if tmp.exists() {
        std::fs::remove_file(&tmp)?;
    }
    let file = File::create(&tmp)?;
    let mut zip = zip::ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    zip.start_file("manifest.json", options)?;
    zip.write_all(serde_json::to_vec_pretty(manifest)?.as_slice())?;

    let src = src.canonicalize()?;
    for entry in WalkDir::new(&src).into_iter().flatten() {
        let path = entry.path();
        let rel = path.strip_prefix(&src).unwrap_or(path);
        if rel.components().any(|c| SKIP_DIRS.iter().any(|s| c.as_os_str() == *s)) {
            continue;
        }
        if path.is_dir() {
            continue;
        }
        let name = rel.to_string_lossy().replace('\\', "/");
        if name.is_empty() || name.ends_with(".server") || name.ends_with(".tmp") {
            continue;
        }
        zip.start_file(format!("server/{name}"), options)?;
        let mut f = File::open(path)?;
        let mut buf = Vec::new();
        f.read_to_end(&mut buf)?;
        zip.write_all(&buf)?;
    }
    zip.finish()?;
    if dest.exists() {
        std::fs::remove_file(dest)?;
    }
    std::fs::rename(&tmp, dest)?;
    Ok(std::fs::metadata(dest)?.len())
}

fn read_manifest(zip_path: &Path) -> AppResult<SharePackManifest> {
    let file = File::open(zip_path)?;
    let mut archive = zip::ZipArchive::new(file).map_err(|_| invalid_pack())?;
    let mut manifest_file = archive.by_name("manifest.json").map_err(|_| invalid_pack())?;
    let mut buf = String::new();
    manifest_file.read_to_string(&mut buf)?;
    drop(manifest_file);
    let manifest: SharePackManifest = serde_json::from_str(&buf).map_err(|_| invalid_pack())?;
    if manifest.format != FORMAT {
        return Err(invalid_pack());
    }
    if manifest.format_version == 0 || manifest.format_version > FORMAT_VERSION {
        return Err(AppError::new(
            "Unsupported server file",
            "This .server file was made with a newer version of ServerForge.",
        ));
    }
    if manifest.name.trim().is_empty() {
        return Err(invalid_pack());
    }
    Ok(manifest)
}

fn extract_pack(zip_path: &Path, dest: &Path) -> AppResult<()> {
    let file = File::open(zip_path)?;
    let mut archive = zip::ZipArchive::new(file).map_err(|_| invalid_pack())?;
    let mut extracted = 0u32;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let Some(enclosed) = file.enclosed_name() else {
            continue;
        };
        let name = enclosed.to_string_lossy().replace('\\', "/");
        if name == "manifest.json" {
            continue;
        }
        let Some(rel) = name.strip_prefix("server/") else {
            continue;
        };
        if rel.is_empty() {
            continue;
        }
        let outpath = dest.join(rel);
        if file.is_dir() {
            std::fs::create_dir_all(&outpath)?;
            continue;
        }
        if let Some(parent) = outpath.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut outfile = File::create(&outpath)?;
        std::io::copy(&mut file, &mut outfile)?;
        extracted += 1;
    }
    if extracted == 0 {
        return Err(AppError::new(
            "Empty server file",
            "This .server file does not contain any server files.",
        ));
    }
    Ok(())
}

fn invalid_pack() -> AppError {
    AppError::new(
        "Not a ServerForge server",
        "That file is not a valid .server pack. Ask the sender to export it from ServerForge.",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ServerRecord;

    fn sample_server(path: &Path) -> ServerRecord {
        ServerRecord {
            id: "id".into(),
            name: "Testing Server".into(),
            path: path.to_string_lossy().to_string(),
            provider: "paper".into(),
            minecraft_version: "1.21.1".into(),
            loader_version: None,
            build: Some("123".into()),
            java_path: None,
            memory_min_mb: 1024,
            memory_max_mb: 4096,
            port: 25565,
            status: "offline".into(),
            auto_start: 0,
            eula_accepted: 1,
            motd: Some("Hello".into()),
            pid: None,
            created_at: "now".into(),
            updated_at: "now".into(),
            last_started_at: None,
        }
    }

    #[test]
    fn roundtrip_pack_keeps_server_files() {
        let root = tempfile::tempdir().unwrap();
        let server_dir = root.path().join("testing-server");
        std::fs::create_dir_all(server_dir.join("plugins")).unwrap();
        std::fs::write(server_dir.join("eula.txt"), "eula=true").unwrap();
        std::fs::write(server_dir.join("server.properties"), "motd=Hello").unwrap();
        std::fs::write(server_dir.join("plugins").join("luckperms.jar"), b"jar").unwrap();
        std::fs::create_dir_all(server_dir.join("logs")).unwrap();
        std::fs::write(server_dir.join("logs").join("latest.log"), "noise").unwrap();

        let dest = share_pack_path(&server_dir, "Testing Server");
        let manifest = SharePackManifest::from_server(&sample_server(&server_dir));
        write_pack(&server_dir, &dest, &manifest).unwrap();
        assert_eq!(dest, server_dir.join("testing-server.server"));

        let read = read_manifest(&dest).unwrap();
        assert_eq!(read.name, "Testing Server");
        assert_eq!(read.provider, "paper");

        let imported = root.path().join("imported");
        std::fs::create_dir_all(&imported).unwrap();
        extract_pack(&dest, &imported).unwrap();
        assert_eq!(std::fs::read_to_string(imported.join("eula.txt")).unwrap(), "eula=true");
        assert!(imported.join("plugins").join("luckperms.jar").exists());
        assert!(!imported.join("logs").join("latest.log").exists());
    }

    #[test]
    fn rejects_random_zip() {
        let dir = tempfile::tempdir().unwrap();
        let zip_path = dir.path().join("fake.server");
        let file = File::create(&zip_path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        zip.start_file("readme.txt", SimpleFileOptions::default()).unwrap();
        zip.write_all(b"nope").unwrap();
        zip.finish().unwrap();
        assert!(read_manifest(&zip_path).is_err());
    }
}
