use std::path::Path;

use crate::models::{DiskInfo, SystemInfo, WorldInfo};
use crate::utils::path::looks_like_server_dir;

pub fn system_info(offline: bool) -> SystemInfo {
    let mut sys = sysinfo::System::new();
    sys.refresh_memory();
    SystemInfo {
        total_memory_mb: sys.total_memory() / 1024 / 1024,
        available_memory_mb: sys.available_memory() / 1024 / 1024,
        cpu_count: sysinfo::System::physical_core_count().unwrap_or(1),
        os: sysinfo::System::long_os_version().unwrap_or_else(|| std::env::consts::OS.to_string()),
        arch: std::env::consts::ARCH.to_string(),
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        offline,
    }
}

pub fn inspect_directory(path: &Path) -> DiskInfo {
    let mut disks = sysinfo::Disks::new_with_refreshed_list();
    disks.refresh(true);
    let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let disk = disks.iter().find(|d| canonical.starts_with(d.mount_point()));
    let writable = path
        .parent()
        .map(|p| {
            let probe = p.join(".serverforge-write-test");
            match std::fs::write(&probe, b"ok") {
                Ok(_) => {
                    let _ = std::fs::remove_file(probe);
                    true
                }
                Err(_) => false,
            }
        })
        .or_else(|| {
            if path.exists() {
                let probe = path.join(".serverforge-write-test");
                match std::fs::write(&probe, b"ok") {
                    Ok(_) => {
                        let _ = std::fs::remove_file(&probe);
                        Some(true)
                    }
                    Err(_) => Some(false),
                }
            } else {
                Some(true)
            }
        })
        .unwrap_or(true);
    let empty = path
        .read_dir()
        .map(|mut rd| rd.next().is_none())
        .unwrap_or(true);
    DiskInfo {
        path: path.to_string_lossy().to_string(),
        total_bytes: disk.map(|d| d.total_space()).unwrap_or(0),
        available_bytes: disk.map(|d| d.available_space()).unwrap_or(0),
        writable,
        contains_server: looks_like_server_dir(path),
        empty,
    }
}

pub fn list_worlds(server_path: &Path) -> Vec<WorldInfo> {
    let mut worlds = Vec::new();
    let Ok(rd) = std::fs::read_dir(server_path) else {
        return worlds;
    };
    for entry in rd.flatten() {
        let path = entry.path();
        if path.join("level.dat").exists() {
            let name = entry.file_name().to_string_lossy().to_string();
            let kind = if name.contains("nether") {
                "nether"
            } else if name.contains("end") {
                "end"
            } else {
                "overworld"
            };
            worlds.push(WorldInfo {
                name,
                path: path.to_string_lossy().to_string(),
                size_bytes: dir_size(&path),
                kind: kind.into(),
            });
        }
    }
    worlds
}

fn dir_size(path: &Path) -> u64 {
    walkdir::WalkDir::new(path)
        .into_iter()
        .flatten()
        .filter(|e| e.file_type().is_file())
        .filter_map(|e| e.metadata().ok().map(|m| m.len()))
        .sum()
}

pub fn port_in_use(port: u16) -> bool {
    std::net::TcpListener::bind(("127.0.0.1", port)).is_err()
}

pub async fn probe_offline(client: &reqwest::Client) -> bool {
    client
        .get("https://piston-meta.mojang.com/mc/game/version_manifest_v2.json")
        .send()
        .await
        .ok()
        .map(|r| !r.status().is_success())
        .unwrap_or(true)
}
