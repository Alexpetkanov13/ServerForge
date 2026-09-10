use parking_lot::Mutex;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, Command};
use tokio::sync::Mutex as AsyncMutex;
use tokio::time::{sleep, Duration};

use crate::error::{AppError, AppResult};
use crate::models::{CrashInfo, ServerLogLine, ServerMetrics, ServerRecord, ServerStatus};
use crate::services::java::validate_java_executable;
use crate::services::servers;
use crate::services::share;
use crate::state::AppState;

pub struct ManagedServer {
    pub server_id: String,
    pub server_path: PathBuf,
    pub pid: AtomicU32,
    pub started_at: Mutex<Option<Instant>>,
    pub stdin: AsyncMutex<Option<ChildStdin>>,
    pub child: AsyncMutex<Option<Child>>,
    pub status: Mutex<ServerStatus>,
    pub players: AtomicU32,
    pub max_players: AtomicU32,
    pub tps: Mutex<Option<f32>>,
    pub last_line: Mutex<String>,
}

impl ManagedServer {
    pub fn new(server_id: String, server_path: PathBuf) -> Arc<Self> {
        Arc::new(Self {
            server_id,
            server_path,
            pid: AtomicU32::new(0),
            started_at: Mutex::new(None),
            stdin: AsyncMutex::new(None),
            child: AsyncMutex::new(None),
            status: Mutex::new(ServerStatus::Offline),
            players: AtomicU32::new(0),
            max_players: AtomicU32::new(0),
            tps: Mutex::new(None),
            last_line: Mutex::new(String::new()),
        })
    }
}

pub async fn start_server(app: AppHandle, state: &AppState, server: &ServerRecord) -> AppResult<()> {
    if state.processes.get(&server.id).is_some() {
        return Err(AppError::new(
            "Server already running",
            "Stop the server before starting it again.",
        ));
    }
    let java = server
        .java_path
        .as_deref()
        .ok_or_else(|| AppError::new("Java not configured", "Choose a Java runtime before starting this server."))?;
    let java_path = PathBuf::from(java);
    validate_java_executable(&java_path)?;

    let cwd = PathBuf::from(&server.path);
    if !cwd.exists() {
        return Err(AppError::new(
            "Server folder missing",
            "The server directory no longer exists.",
        ));
    }

    let args = launch_args(server, &cwd)?;
    let mut cmd = Command::new(&java_path);
    cmd.current_dir(&cwd)
        .args(&args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(false);
    crate::utils::process::hide_window(&mut cmd);

    let mut child = cmd.spawn().map_err(|e| {
        AppError::new(
            "Unable to start server",
            "Java could not be launched.",
        )
        .with_technical(e.to_string())
    })?;
    let pid = child.id().unwrap_or_default();
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let stdin = child.stdin.take();

    let handle = ManagedServer::new(server.id.clone(), cwd.clone());
    handle.pid.store(pid, Ordering::SeqCst);
    *handle.started_at.lock() = Some(Instant::now());
    *handle.status.lock() = ServerStatus::Starting;
    *handle.stdin.lock().await = stdin;
    *handle.child.lock().await = Some(child);
    state.processes.insert(server.id.clone(), handle.clone());
    // Seed the player-cap from server.properties so the Overview denominator is
    // correct immediately — without this it stayed 0 until a `/list` log line
    // arrived and the UI fell back to a hardcoded 20.
    let props_path = cwd.join("server.properties");
    let max_players = tokio::fs::read_to_string(&props_path)
        .await
        .map(|raw| crate::utils::properties::get_max_players(&raw))
        .unwrap_or(20);
    handle.max_players.store(max_players, Ordering::SeqCst);
    set_status(&state.db, &server.id, ServerStatus::Starting).await?;

    if let Some(out) = stdout {
        spawn_reader(app.clone(), handle.clone(), state.db.clone(), server.id.clone(), out, "stdout");
    }
    if let Some(err) = stderr {
        spawn_reader(app.clone(), handle.clone(), state.db.clone(), server.id.clone(), err, "stderr");
    }

    let waiter = handle.clone();
    let db = state.db.clone();
    let app2 = app.clone();
    let sid = server.id.clone();
    tokio::spawn(async move {
        loop {
            let finished = {
                let mut child_slot = waiter.child.lock().await;
                match child_slot.as_mut() {
                    None => true,
                    Some(child) => match child.try_wait() {
                        Ok(Some(_)) | Err(_) => {
                            *child_slot = None;
                            true
                        }
                        Ok(None) => false,
                    },
                }
            };
            if finished {
                break;
            }
            sleep(Duration::from_millis(100)).await;
        }
        let current = waiter.status.lock().clone();
        if current == ServerStatus::Stopping {
            *waiter.status.lock() = ServerStatus::Offline;
            let _ = set_status(&db, &sid, ServerStatus::Offline).await;
        } else if current != ServerStatus::Offline {
            *waiter.status.lock() = ServerStatus::Crashed;
            let _ = set_status(&db, &sid, ServerStatus::Crashed).await;
            let last = waiter.last_line.lock().clone();
            let crash = classify_crash(&last);
            let _ = app2.emit(
                "server-crash",
                CrashInfo {
                    server_id: sid.clone(),
                    reason: crash.0,
                    suggestion: crash.1,
                    report_path: find_crash_report(Path::new(&sid)).map(|p| p.to_string_lossy().to_string()),
                },
            );
        }
        *waiter.stdin.lock().await = None;
        *waiter.child.lock().await = None;
        waiter.pid.store(0, Ordering::SeqCst);
    });

    Ok(())
}

pub async fn send_command(state: &AppState, server_id: &str, command: &str) -> AppResult<()> {
    let Some(handle) = state.processes.get(server_id).map(|h| h.clone()) else {
        return Err(AppError::new(
            "Server is offline",
            "Start the server before sending console commands.",
        ));
    };
    let mut stdin = handle.stdin.lock().await;
    let Some(stdin) = stdin.as_mut() else {
        return Err(AppError::new(
            "Console unavailable",
            "The server process is not accepting commands.",
        ));
    };
    let line = if command.ends_with('\n') {
        command.to_string()
    } else {
        format!("{command}\n")
    };
    stdin.write_all(line.as_bytes()).await?;
    stdin.flush().await?;
    Ok(())
}

pub async fn stop_server(state: &AppState, server_id: &str, force: bool) -> AppResult<()> {
    let path = servers::get_server(&state.db, server_id)
        .await
        .ok()
        .map(|s| PathBuf::from(s.path));
    let handle = state.processes.get(server_id).map(|h| h.clone());
    let path = path.or_else(|| handle.as_ref().map(|h| h.server_path.clone()));

    if let Some(handle) = handle {
        *handle.status.lock() = ServerStatus::Stopping;
        set_status(&state.db, server_id, ServerStatus::Stopping).await?;
        if !force {
            let sent = send_stop_command(&handle).await;
            if sent {
                for _ in 0..60 {
                    sleep(Duration::from_millis(250)).await;
                    if !server_still_running(&handle, path.as_deref()) {
                        return finish_stop(state, server_id, true).await;
                    }
                }
            }
        }
        force_kill(&handle, path.as_deref()).await;
        return finish_stop(state, server_id, false).await;
    }

    if let Some(dir) = path.as_deref() {
        crate::utils::process::kill_server_processes(0, Some(dir));
        wait_until_dead(0, Some(dir)).await;
    }
    set_status(&state.db, server_id, ServerStatus::Offline).await?;
    Ok(())
}

async fn send_stop_command(handle: &ManagedServer) -> bool {
    let mut stdin = handle.stdin.lock().await;
    let Some(stdin) = stdin.as_mut() else {
        return false;
    };
    for payload in ["stop\r\n"] {
        if stdin.write_all(payload.as_bytes()).await.is_err() {
            return false;
        }
    }
    stdin.flush().await.is_ok()
}

fn server_still_running(handle: &ManagedServer, path: Option<&Path>) -> bool {
    let pid = handle.pid.load(Ordering::SeqCst);
    if crate::utils::process::process_alive(pid) {
        return true;
    }
    path.is_some_and(crate::utils::process::java_running_in_dir)
}

async fn force_kill(handle: &ManagedServer, path: Option<&Path>) {
    let pid = handle.pid.load(Ordering::SeqCst);
    if let Some(mut child) = handle.child.lock().await.take() {
        let _ = tokio::time::timeout(Duration::from_secs(2), child.kill()).await;
    }
    crate::utils::process::kill_server_processes(pid, path);
    *handle.stdin.lock().await = None;
    handle.pid.store(0, Ordering::SeqCst);
    wait_until_dead(pid, path).await;
    crate::utils::process::kill_server_processes(pid, path);
}

async fn wait_until_dead(pid: u32, path: Option<&Path>) {
    for i in 0..40 {
        let pid_alive = crate::utils::process::process_alive(pid);
        let java_alive = path.is_some_and(crate::utils::process::java_running_in_dir);
        if !pid_alive && !java_alive {
            return;
        }
        if i % 5 == 4 {
            crate::utils::process::kill_server_processes(pid, path);
        }
        sleep(Duration::from_millis(150)).await;
    }
}

async fn finish_stop(state: &AppState, server_id: &str, write_pack: bool) -> AppResult<()> {
    state.processes.remove(server_id);
    set_status(&state.db, server_id, ServerStatus::Offline).await?;
    if write_pack {
        if let Err(err) = share::write_share_pack(state, server_id).await {
            tracing::warn!(server_id = %server_id, error = %err, "share pack update after stop failed");
        }
    }
    Ok(())
}

pub fn metrics(handle: &ManagedServer, memory_max_mb: i64) -> ServerMetrics {
    let uptime = handle
        .started_at
        .lock()
        .map(|t| t.elapsed().as_secs())
        .unwrap_or(0);
    let mut cpu = 0.0;
    let mut memory = 0;
    let pid = handle.pid.load(Ordering::SeqCst);
    if pid > 0 {
        let mut sys = sysinfo::System::new();
        sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
        if let Some(proc) = sys.process(sysinfo::Pid::from_u32(pid)) {
            cpu = proc.cpu_usage();
            memory = proc.memory();
        }
    }
    ServerMetrics {
        server_id: handle.server_id.clone(),
        cpu_percent: cpu,
        memory_bytes: memory,
        memory_max_bytes: (memory_max_mb.max(0) as u64) * 1024 * 1024,
        uptime_seconds: uptime,
        player_count: Some(handle.players.load(Ordering::SeqCst)),
        max_players: Some(handle.max_players.load(Ordering::SeqCst)),
        tps: *handle.tps.lock(),
        status: handle.status.lock().as_str().to_string(),
    }
}

fn spawn_reader<R: tokio::io::AsyncRead + Unpin + Send + 'static>(
    app: AppHandle,
    handle: Arc<ManagedServer>,
    db: sqlx::SqlitePool,
    server_id: String,
    reader: R,
    stream: &'static str,
) {
    tokio::spawn(async move {
        let mut lines = BufReader::new(reader).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            *handle.last_line.lock() = line.clone();
            interpret_line(&handle, &line);
            let payload = ServerLogLine {
                server_id: server_id.clone(),
                stream: stream.to_string(),
                line: line.clone(),
                timestamp: chrono::Utc::now(),
            };
            let _ = app.emit("server-log", payload);
            if handle.status.lock().clone() == ServerStatus::Starting && is_ready(&line) {
                *handle.status.lock() = ServerStatus::Online;
                let _ = set_status(&db, &server_id, ServerStatus::Online).await;
                let _ = app.emit("server-status", serde_json::json!({ "serverId": server_id, "status": "online" }));
            }
        }
    });
}

fn interpret_line(handle: &ManagedServer, line: &str) {
    if let Some(caps) = player_count(line) {
        handle.players.store(caps.0, Ordering::SeqCst);
        handle.max_players.store(caps.1, Ordering::SeqCst);
    }
    if let Some(tps) = parse_tps(line) {
        *handle.tps.lock() = Some(tps);
    }
}

fn is_ready(line: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    lower.contains("done (") || lower.contains("done!") || lower.contains("for help, type")
}

fn player_count(line: &str) -> Option<(u32, u32)> {
    // There are 3 of a max of 20 players online
    let lower = line.to_ascii_lowercase();
    if lower.contains("of a max of") && lower.contains("players") {
        let nums: Vec<u32> = lower
            .split(|c: char| !c.is_ascii_digit())
            .filter_map(|s| s.parse().ok())
            .collect();
        if nums.len() >= 2 {
            return Some((nums[0], nums[1]));
        }
    }
    None
}

fn parse_tps(line: &str) -> Option<f32> {
    let lower = line.to_ascii_lowercase();
    if lower.contains("tps") {
        line.split_whitespace()
            .rev()
            .find_map(|part| part.trim_matches(',').parse::<f32>().ok().filter(|v| *v > 0.0 && *v <= 21.0))
    } else {
        None
    }
}

fn classify_crash(last_line: &str) -> (String, String) {
    let lower = last_line.to_ascii_lowercase();
    if lower.contains("outofmemory") || lower.contains("java heap space") {
        (
            "OutOfMemoryError".into(),
            "Increase allocated RAM for this server.".into(),
        )
    } else if lower.contains("bind") || lower.contains("address already in use") || lower.contains("failed to bind") {
        (
            "Port already in use".into(),
            "Change the server port or stop the other process using it.".into(),
        )
    } else if lower.contains("unsupportedclassversion") || lower.contains("java.lang.class") {
        (
            "Incompatible Java version".into(),
            "Install or select the Java version recommended for this Minecraft release.".into(),
        )
    } else if last_line.trim().is_empty() {
        (
            "Unexpected process termination".into(),
            "Check the latest crash report in the server folder.".into(),
        )
    } else {
        (
            "Server crashed".into(),
            "Open the console and crash-reports folder for details.".into(),
        )
    }
}

fn find_crash_report(_server_id: &Path) -> Option<PathBuf> {
    None
}

pub async fn set_status(db: &sqlx::SqlitePool, server_id: &str, status: ServerStatus) -> AppResult<()> {
    sqlx::query("UPDATE servers SET status = ?, updated_at = ? WHERE id = ?")
        .bind(status.as_str())
        .bind(chrono::Utc::now().to_rfc3339())
        .bind(server_id)
        .execute(db)
        .await?;
    Ok(())
}

fn launch_args(server: &ServerRecord, cwd: &Path) -> AppResult<Vec<String>> {
    let mut args = vec![
        format!("-Xms{}M", server.memory_min_mb),
        format!("-Xmx{}M", server.memory_max_mb),
    ];
    if let Ok(Some(row)) = futures_executor_disabled_settings() {
        let _ = row;
    }
    args.extend(aikar_flags(server.memory_max_mb as u32));
    if let Some(argfile) = find_argfile(cwd) {
        let jvm = cwd.join("user_jvm_args.txt");
        if jvm.exists() {
            args.push(format!("@{}", jvm.display()));
        }
        args.push(format!("@{}", argfile.display()));
        args.push("nogui".into());
        return Ok(args);
    }
    let jar = find_launch_jar(cwd).ok_or_else(|| {
        AppError::new(
            "Server jar missing",
            "No launchable server jar was found in the server folder.",
        )
    })?;
    args.push("-jar".into());
    args.push(jar.file_name().unwrap().to_string_lossy().to_string());
    args.push("nogui".into());
    Ok(args)
}

fn aikar_flags(max_mb: u32) -> Vec<String> {
    let region = if max_mb >= 12000 { "16M" } else { "8M" };
    vec![
        "-XX:+UseG1GC".into(),
        "-XX:+ParallelRefProcEnabled".into(),
        "-XX:MaxGCPauseMillis=200".into(),
        "-XX:+UnlockExperimentalVMOptions".into(),
        "-XX:+DisableExplicitGC".into(),
        "-XX:+AlwaysPreTouch".into(),
        "-XX:G1NewSizePercent=30".into(),
        "-XX:G1MaxNewSizePercent=40".into(),
        format!("-XX:G1HeapRegionSize={region}"),
        "-XX:G1ReservePercent=20".into(),
        "-XX:G1HeapWastePercent=5".into(),
        "-XX:G1MixedGCCountTarget=4".into(),
        "-XX:InitiatingHeapOccupancyPercent=15".into(),
        "-XX:G1MixedGCLiveThresholdPercent=90".into(),
        "-XX:G1RSetUpdatingPauseTimePercent=5".into(),
        "-XX:SurvivorRatio=32".into(),
        "-XX:+PerfDisableSharedMem".into(),
        "-XX:MaxTenuringThreshold=1".into(),
        "-Dusing.aikars.flags=https://mcflags.emc.gs".into(),
        "-Daikars.new.flags=true".into(),
    ]
}

fn find_launch_jar(cwd: &Path) -> Option<PathBuf> {
    let preferred = ["server.jar", "fabric-server-launch.jar"];
    for name in preferred {
        let p = cwd.join(name);
        if p.exists() {
            return Some(p);
        }
    }
    std::fs::read_dir(cwd).ok()?.flatten().map(|e| e.path()).find(|p| {
        p.extension().and_then(|s| s.to_str()) == Some("jar")
            && !p.file_name().unwrap_or_default().to_string_lossy().contains("installer")
            && !p.file_name().unwrap_or_default().to_string_lossy().contains("BuildTools")
    })
}

fn find_argfile(cwd: &Path) -> Option<PathBuf> {
    let needle = if cfg!(windows) {
        "win_args.txt"
    } else {
        "unix_args.txt"
    };
    walkdir::WalkDir::new(cwd.join("libraries"))
        .max_depth(12)
        .into_iter()
        .flatten()
        .map(|e| e.into_path())
        .find(|p| p.file_name().and_then(|s| s.to_str()) == Some(needle))
}

fn futures_executor_disabled_settings() -> Result<Option<()>, ()> {
    Ok(None)
}
