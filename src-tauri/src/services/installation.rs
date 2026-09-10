use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Emitter};
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::{CreateServerRequest, InstallationProgress, ServerStatus};
use crate::providers::get_provider;
use crate::services::java::{pick_java, required_java_for, validate_java_executable};
use crate::services::process::{self, set_status};
use crate::state::AppState;
use crate::utils::path::{folder_name_from_server, looks_like_server_dir, validate_server_name};
use crate::utils::properties::write_properties_file;

const STEPS: &[&str] = &[
    "prepare_directory",
    "resolve_java",
    "resolve_version",
    "download_artifact",
    "verify_artifact",
    "install_artifact",
    "generate_configuration",
    "accept_eula",
    "first_startup",
    "finalize",
];

pub async fn install_server(
    app: AppHandle,
    state: &AppState,
    req: CreateServerRequest,
) -> AppResult<String> {
    validate_server_name(&req.name)?;
    if !req.eula_accepted {
        return Err(AppError::new(
            "EULA not accepted",
            "You must accept the Minecraft EULA before ServerForge can create the server.",
        ));
    }
    let existing: Option<(String,)> = sqlx::query_as("SELECT id FROM servers WHERE name = ?")
        .bind(&req.name)
        .fetch_optional(&state.db)
        .await?;
    if existing.is_some() {
        return Err(AppError::new(
            "Duplicate server name",
            "A server with this name already exists.",
        ));
    }

    let id = Uuid::new_v4().to_string();
    let task_id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let mut logs: Vec<String> = Vec::new();

    sqlx::query(
        "INSERT INTO servers (id, name, path, provider, minecraft_version, loader_version, build, java_path, memory_min_mb, memory_max_mb, port, status, auto_start, eula_accepted, motd, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 0, 1, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&req.name)
    .bind(&req.path)
    .bind(&req.provider)
    .bind(&req.minecraft_version)
    .bind(&req.loader_version)
    .bind(&req.build)
    .bind(&req.java_path)
    .bind(req.memory_min_mb as i64)
    .bind(req.memory_max_mb as i64)
    .bind(req.port as i64)
    .bind(ServerStatus::Installing.as_str())
    .bind(&req.motd)
    .bind(&now)
    .bind(&now)
    .execute(&state.db)
    .await?;

    sqlx::query(
        "INSERT INTO installation_tasks (id, server_id, payload_json, current_step, status, created_at, updated_at)
         VALUES (?, ?, ?, ?, 'running', ?, ?)",
    )
    .bind(&task_id)
    .bind(&id)
    .bind(serde_json::to_string(&req)?)
    .bind(STEPS[0])
    .bind(&now)
    .bind(&now)
    .execute(&state.db)
    .await?;

    let result = run_pipeline(&app, state, &id, &task_id, &req, &mut logs).await;
    match result {
        Ok(()) => {
            emit_progress(&app, &task_id, Some(&id), "finalize", 9, "complete", "Server is ready!", 100, &logs, None);
            set_status(&state.db, &id, ServerStatus::Offline).await?;
            sqlx::query("UPDATE installation_tasks SET status = 'complete', current_step = 'finalize', updated_at = ? WHERE id = ?")
                .bind(chrono::Utc::now().to_rfc3339())
                .bind(&task_id)
                .execute(&state.db)
                .await?;
            if let Err(err) = crate::services::share::write_share_pack(state, &id).await {
                tracing::warn!(server_id = %id, error = %err, "could not write initial share pack");
            }
            Ok(id)
        }
        Err(err) => {
            emit_progress(&app, &task_id, Some(&id), "error", 0, "error", &err.message, 0, &logs, Some(err.clone()));
            set_status(&state.db, &id, ServerStatus::Error).await?;
            sqlx::query("UPDATE installation_tasks SET status = 'error', error = ?, updated_at = ? WHERE id = ?")
                .bind(err.to_string())
                .bind(chrono::Utc::now().to_rfc3339())
                .bind(&task_id)
                .execute(&state.db)
                .await?;
            Err(err)
        }
    }
}

async fn run_pipeline(
    app: &AppHandle,
    state: &AppState,
    server_id: &str,
    task_id: &str,
    req: &CreateServerRequest,
    logs: &mut Vec<String>,
) -> AppResult<()> {
    let dir = PathBuf::from(&req.path);
    step(app, task_id, server_id, 0, "Creating directory", logs);
    tokio::fs::create_dir_all(&dir).await?;
    if looks_like_server_dir(&dir) && !dir.join("server.properties").exists() {
        // empty or new is fine
    }
    let existing_jar = std::fs::read_dir(&dir).ok().and_then(|rd| {
        rd.flatten().find(|e| e.path().extension().and_then(|s| s.to_str()) == Some("jar"))
    });
    if existing_jar.is_some() && dir.join("eula.txt").exists() {
        return Err(AppError::new(
            "Folder already contains a server",
            "Choose an empty folder or import the existing server instead.",
        ));
    }

    step(app, task_id, server_id, 1, "Checking Java", logs);
    let required = required_java_for(&req.minecraft_version);
    let java = pick_java(required, req.java_path.as_deref()).await?;
    validate_java_executable(Path::new(&java.path))?;
    sqlx::query("UPDATE servers SET java_path = ? WHERE id = ?")
        .bind(&java.path)
        .bind(server_id)
        .execute(&state.db)
        .await?;

    step(app, task_id, server_id, 2, "Resolving version", logs);
    let provider = get_provider(state.http.clone(), &req.provider)?;
    let artifact = provider
        .resolve_download(
            &req.minecraft_version,
            req.build.as_deref(),
            req.loader_version.as_deref(),
            req.installer_version.as_deref(),
        )
        .await?;

    step(app, task_id, server_id, 3, &format!("Downloading {}", artifact.file_name), logs);
    let dest = dir.join(&artifact.file_name);
    let app2 = app.clone();
    let downloads = state.downloads.clone();
    downloads
        .download_file(
            &state.http,
            &artifact.url,
            &dest,
            &format!("{} {}", req.provider, req.minecraft_version),
            artifact.sha256.as_deref(),
            artifact.sha1.as_deref(),
            move |p| {
                let _ = app2.emit("download-progress", p);
            },
        )
        .await?;

    step(app, task_id, server_id, 4, "Verifying download", logs);
    // Hash verification happens inside download_file when checksums exist.

    step(app, task_id, server_id, 5, "Installing server files", logs);
    if artifact.kind == "server" && artifact.file_name != "server.jar" {
        let target = dir.join("server.jar");
        if dest != target {
            tokio::fs::copy(&dest, &target).await?;
        }
    }
    provider
        .post_install(&dir, Path::new(&java.path), &dest, &req.minecraft_version)
        .await?;
    for folder in folders_for_provider(&req.provider) {
        tokio::fs::create_dir_all(dir.join(folder)).await?;
    }

    step(app, task_id, server_id, 6, "Generating configuration", logs);
    let mut props = BTreeMap::new();
    props.insert("motd".into(), req.motd.clone().unwrap_or_else(|| req.name.clone()));
    props.insert("server-port".into(), req.port.to_string());
    if let Some(v) = &req.difficulty {
        props.insert("difficulty".into(), v.clone());
    }
    if let Some(v) = &req.gamemode {
        props.insert("gamemode".into(), v.clone());
    }
    if let Some(v) = req.max_players {
        props.insert("max-players".into(), v.to_string());
    }
    if let Some(v) = req.online_mode {
        props.insert("online-mode".into(), v.to_string());
    }
    if let Some(v) = req.pvp {
        props.insert("pvp".into(), v.to_string());
    }
    if let Some(v) = req.view_distance {
        props.insert("view-distance".into(), v.to_string());
    }
    if let Some(v) = req.simulation_distance {
        props.insert("simulation-distance".into(), v.to_string());
    }
    if let Some(v) = req.allow_flight {
        props.insert("allow-flight".into(), v.to_string());
    }
    write_properties_file(&dir.join("server.properties"), &props).await?;

    step(app, task_id, server_id, 7, "Writing EULA", logs);
    tokio::fs::write(
        dir.join("eula.txt"),
        "# Accepted in ServerForge after explicit user confirmation.\n# https://aka.ms/MinecraftEULA\neula=true\n",
    )
    .await?;

    step(app, task_id, server_id, 8, "First startup", logs);
    if req.generate_world || req.start_after_install {
        let record = crate::services::servers::get_server(&state.db, server_id).await?;
        process::start_server(app.clone(), state, &record).await?;
        if !req.start_after_install {
            wait_until_online(state, server_id).await;
            let _ = process::stop_server(state, server_id, false).await;
        }
    }

    step(app, task_id, server_id, 9, "Installation complete", logs);
    sqlx::query("UPDATE servers SET updated_at = ? WHERE id = ?")
        .bind(chrono::Utc::now().to_rfc3339())
        .bind(server_id)
        .execute(&state.db)
        .await?;
    let _ = folder_name_from_server(&req.name);
    Ok(())
}

fn folders_for_provider(provider: &str) -> Vec<&'static str> {
    let mut folders = vec!["world", "crash-reports", "logs", "backups"];
    match provider {
        "paper" | "spigot" | "purpur" | "folia" => folders.insert(0, "plugins"),
        "fabric" | "forge" | "neoforge" => folders.insert(0, "mods"),
        _ => {}
    }
    folders
}

async fn wait_until_online(state: &AppState, server_id: &str) {
    for _ in 0..180 {
        if let Some(handle) = state.processes.get(server_id) {
            if *handle.status.lock() == ServerStatus::Online {
                return;
            }
            if *handle.status.lock() == ServerStatus::Crashed {
                return;
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}

fn step(app: &AppHandle, task_id: &str, server_id: &str, index: u32, message: &str, logs: &mut Vec<String>) {
    logs.push(message.to_string());
    emit_progress(
        app,
        task_id,
        Some(server_id),
        STEPS.get(index as usize).copied().unwrap_or("unknown"),
        index,
        "running",
        message,
        ((index as f32 / STEPS.len() as f32) * 100.0) as u32,
        logs,
        None,
    );
}

fn emit_progress(
    app: &AppHandle,
    id: &str,
    server_id: Option<&str>,
    step: &str,
    step_index: u32,
    status: &str,
    message: &str,
    percent: u32,
    logs: &[String],
    error: Option<AppError>,
) {
    let _ = app.emit(
        "installation-progress",
        InstallationProgress {
            id: id.to_string(),
            server_id: server_id.map(|s| s.to_string()),
            step: step.to_string(),
            step_index,
            step_count: STEPS.len() as u32,
            status: status.to_string(),
            message: message.to_string(),
            percent,
            logs: logs.to_vec(),
            error,
        },
    );
}

pub async fn cleanup_failed(state: &AppState, server_id: &str, delete_files: bool) -> AppResult<()> {
    let row: Option<(String,)> = sqlx::query_as("SELECT path FROM servers WHERE id = ?")
        .bind(server_id)
        .fetch_optional(&state.db)
        .await?;
    if delete_files {
        if let Some((path,)) = row {
            let path = PathBuf::from(path);
            if path.exists() {
                tokio::fs::remove_dir_all(path).await.ok();
            }
        }
    }
    sqlx::query("DELETE FROM servers WHERE id = ?")
        .bind(server_id)
        .execute(&state.db)
        .await?;
    Ok(())
}
