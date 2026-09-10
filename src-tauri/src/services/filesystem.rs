use std::path::{Path, PathBuf};
use tokio::fs;

use crate::error::{AppError, AppResult};
use crate::models::FileEntry;
use crate::utils::path::is_path_inside;

pub fn assert_inside(root: &Path, target: &Path) -> AppResult<PathBuf> {
    let root_c = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let target_c = if target.exists() {
        target.canonicalize().unwrap_or_else(|_| target.to_path_buf())
    } else {
        target.to_path_buf()
    };
    if target_c == root_c || target_c.starts_with(&root_c) || is_path_inside(&root_c, &target_c) {
        return Ok(target_c);
    }
    // Allow creating files under root even if they don't exist yet.
    if target.starts_with(root) {
        return Ok(target.to_path_buf());
    }
    Err(AppError::new(
        "Path not allowed",
        "File operations are limited to this server's directory.",
    ))
}

pub async fn list_dir(root: &Path, rel: &str) -> AppResult<Vec<FileEntry>> {
    let path = if rel.is_empty() || rel == "." {
        root.to_path_buf()
    } else {
        root.join(rel)
    };
    let path = assert_inside(root, &path)?;
    let mut rd = fs::read_dir(&path).await?;
    let mut out = Vec::new();
    while let Some(entry) = rd.next_entry().await? {
        let meta = entry.metadata().await?;
        out.push(FileEntry {
            name: entry.file_name().to_string_lossy().to_string(),
            path: entry.path().to_string_lossy().to_string(),
            is_dir: meta.is_dir(),
            size: meta.len(),
            modified: meta.modified().ok().map(chrono::DateTime::<chrono::Utc>::from),
        });
    }
    out.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then(a.name.to_lowercase().cmp(&b.name.to_lowercase())));
    Ok(out)
}

pub async fn read_text(root: &Path, rel: &str) -> AppResult<String> {
    let path = assert_inside(root, &root.join(rel))?;
    Ok(fs::read_to_string(path).await?)
}

pub async fn write_text(root: &Path, rel: &str, contents: &str) -> AppResult<()> {
    let path = root.join(rel);
    let _ = assert_inside(root, &path)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).await?;
    }
    fs::write(path, contents).await?;
    Ok(())
}

pub async fn create_dir(root: &Path, rel: &str) -> AppResult<()> {
    let path = root.join(rel);
    let _ = assert_inside(root, &path)?;
    fs::create_dir_all(path).await?;
    Ok(())
}

pub async fn rename(root: &Path, from: &str, to: &str) -> AppResult<()> {
    let src = assert_inside(root, &root.join(from))?;
    let dest = root.join(to);
    let _ = assert_inside(root, &dest)?;
    fs::rename(src, dest).await?;
    Ok(())
}

pub async fn remove(root: &Path, rel: &str) -> AppResult<()> {
    let path = assert_inside(root, &root.join(rel))?;
    let meta = fs::metadata(&path).await?;
    if meta.is_dir() {
        fs::remove_dir_all(path).await?;
    } else {
        fs::remove_file(path).await?;
    }
    Ok(())
}

pub async fn import_paths(root: &Path, dest_rel: &str, sources: Vec<String>) -> AppResult<u32> {
    let dest_dir = if dest_rel.is_empty() || dest_rel == "." {
        root.to_path_buf()
    } else {
        root.join(dest_rel)
    };
    let _ = assert_inside(root, &dest_dir)?;
    fs::create_dir_all(&dest_dir).await?;
    tokio::task::spawn_blocking(move || {
        let mut count = 0u32;
        for src in sources {
            let src = PathBuf::from(&src);
            if !src.exists() {
                continue;
            }
            let Some(name) = src.file_name() else {
                continue;
            };
            let dest = dest_dir.join(name);
            if src.is_dir() {
                copy_tree(&src, &dest)?;
            } else {
                if let Some(parent) = dest.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::copy(&src, &dest)?;
            }
            count += 1;
        }
        Ok(count)
    })
    .await
    .map_err(|e| AppError::message(e.to_string()))?
}

fn copy_tree(src: &Path, dest: &Path) -> AppResult<()> {
    std::fs::create_dir_all(dest)?;
    for entry in walkdir::WalkDir::new(src).into_iter().flatten() {
        let path = entry.path();
        let rel = path.strip_prefix(src).unwrap_or(path);
        if rel.as_os_str().is_empty() {
            continue;
        }
        let out = dest.join(rel);
        if path.is_dir() {
            std::fs::create_dir_all(&out)?;
        } else if let Some(parent) = out.parent() {
            std::fs::create_dir_all(parent)?;
            std::fs::copy(path, &out)?;
        }
    }
    Ok(())
}

pub fn search(root: &Path, query: &str) -> Vec<FileEntry> {
    let q = query.to_ascii_lowercase();
    walkdir::WalkDir::new(root)
        .max_depth(8)
        .into_iter()
        .flatten()
        .filter(|e| e.file_name().to_string_lossy().to_ascii_lowercase().contains(&q))
        .take(200)
        .filter_map(|e| {
            let meta = e.metadata().ok()?;
            Some(FileEntry {
                name: e.file_name().to_string_lossy().to_string(),
                path: e.path().to_string_lossy().to_string(),
                is_dir: meta.is_dir(),
                size: meta.len(),
                modified: meta.modified().ok().map(chrono::DateTime::<chrono::Utc>::from),
            })
        })
        .collect()
}
