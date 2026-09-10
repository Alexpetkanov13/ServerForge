use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tauri::{AppHandle, State};

use crate::error::AppResult;
use crate::models::*;
use crate::providers::{catalog, get_provider};
use crate::services::{addons, backup, discovery, filesystem, installation, java, process, servers, settings, share, system};
use crate::state::AppState;
use crate::utils::java_version::recommended_java_major;
use crate::utils::memory::{clamp_memory_mb, memory_warning};
use crate::utils::path::validate_server_name;
use crate::utils::properties::{read_properties_file, write_properties_file};

fn queue_share(state: &State<'_, Arc<AppState>>, id: &str) {
    share::schedule_share_pack(state.inner().clone(), id.to_string());
}

#[tauri::command]
pub fn provider_catalog() -> Vec<ProviderInfo> {
    catalog()
}

#[tauri::command]
pub async fn list_minecraft_versions(state: State<'_, Arc<AppState>>, provider: String) -> AppResult<Vec<MinecraftVersionInfo>> {
    get_provider(state.http.clone(), &provider)?
        .list_minecraft_versions()
        .await
}

#[tauri::command]
pub async fn list_builds(
    state: State<'_, Arc<AppState>>,
    provider: String,
    minecraft_version: String,
) -> AppResult<Vec<BuildInfo>> {
    get_provider(state.http.clone(), &provider)?
        .list_builds(&minecraft_version)
        .await
}

#[tauri::command]
pub async fn provider_compatibility(
    state: State<'_, Arc<AppState>>,
    provider: String,
    minecraft_version: String,
) -> AppResult<CompatibilityInfo> {
    let p = get_provider(state.http.clone(), &provider)?;
    let builds = p.list_builds(&minecraft_version).await.unwrap_or_default();
    Ok(p.compatibility(&minecraft_version, builds.iter().any(|b| b.recommended || b.channel == "stable")))
}

#[tauri::command]
pub async fn list_servers(state: State<'_, Arc<AppState>>) -> AppResult<Vec<ServerRecord>> {
    servers::list_servers(&state.db).await
}

#[tauri::command]
pub async fn get_server(state: State<'_, Arc<AppState>>, id: String) -> AppResult<ServerRecord> {
    servers::get_server(&state.db, &id).await
}

#[tauri::command]
pub fn validate_name(name: String) -> AppResult<()> {
    validate_server_name(&name)
}

#[tauri::command]
pub fn inspect_directory(path: String) -> DiskInfo {
    system::inspect_directory(PathBuf::from(path).as_path())
}

#[tauri::command]
pub async fn create_server(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    request: CreateServerRequest,
) -> AppResult<String> {
    installation::install_server(app, &state, request).await
}

#[tauri::command]
pub async fn cleanup_installation(
    state: State<'_, Arc<AppState>>,
    server_id: String,
    delete_files: bool,
) -> AppResult<()> {
    installation::cleanup_failed(&state, &server_id, delete_files).await
}

#[tauri::command]
pub async fn start_server(app: AppHandle, state: State<'_, Arc<AppState>>, id: String) -> AppResult<()> {
    let server = servers::get_server(&state.db, &id).await?;
    process::start_server(app, &state, &server).await
}

#[tauri::command]
pub async fn stop_server(state: State<'_, Arc<AppState>>, id: String, force: bool) -> AppResult<()> {
    process::stop_server(&state, &id, force).await?;
    if force {
        share::schedule_share_pack(state.inner().clone(), id);
    }
    Ok(())
}

#[tauri::command]
pub async fn restart_server(app: AppHandle, state: State<'_, Arc<AppState>>, id: String) -> AppResult<()> {
    process::stop_server(&state, &id, false).await?;
    let server = servers::get_server(&state.db, &id).await?;
    process::start_server(app, &state, &server).await
}

#[tauri::command]
pub async fn send_command(state: State<'_, Arc<AppState>>, id: String, command: String) -> AppResult<()> {
    process::send_command(&state, &id, &command).await
}

#[tauri::command]
pub async fn delete_server(state: State<'_, Arc<AppState>>, id: String, delete_files: bool) -> AppResult<()> {
    let _ = process::stop_server(&state, &id, true).await;
    servers::delete_server(&state.db, &id, delete_files).await
}

#[tauri::command]
pub async fn detect_java() -> AppResult<Vec<JavaRuntime>> {
    java::detect_java().await
}

#[tauri::command]
pub async fn test_java(path: String) -> AppResult<JavaTestResult> {
    java::test_java(PathBuf::from(path).as_path()).await
}

#[tauri::command]
pub fn java_download_url(major: u32) -> String {
    java::adoptium_download_url(major)
}

#[tauri::command]
pub async fn system_snapshot(state: State<'_, Arc<AppState>>) -> AppResult<SystemInfo> {
    let offline = system::probe_offline(&state.http).await;
    Ok(system::system_info(offline))
}

#[tauri::command]
pub fn recommended_java(minecraft_version: String) -> u32 {
    recommended_java_major(&minecraft_version)
}

#[tauri::command]
pub fn memory_advice(allocated_mb: u32, total_system_mb: u64) -> Option<String> {
    memory_warning(allocated_mb, total_system_mb).map(|s| s.to_string())
}

#[tauri::command]
pub fn clamp_memory(requested: u32, total_system_mb: u64) -> u32 {
    clamp_memory_mb(requested, total_system_mb)
}

#[tauri::command]
pub fn port_in_use(port: u16) -> bool {
    system::port_in_use(port)
}

#[tauri::command]
pub async fn list_files(state: State<'_, Arc<AppState>>, id: String, rel: String) -> AppResult<Vec<FileEntry>> {
    let server = servers::get_server(&state.db, &id).await?;
    filesystem::list_dir(PathBuf::from(server.path).as_path(), &rel).await
}

#[tauri::command]
pub async fn read_file(state: State<'_, Arc<AppState>>, id: String, rel: String) -> AppResult<String> {
    let server = servers::get_server(&state.db, &id).await?;
    filesystem::read_text(PathBuf::from(server.path).as_path(), &rel).await
}

#[tauri::command]
pub async fn write_file(state: State<'_, Arc<AppState>>, id: String, rel: String, contents: String) -> AppResult<()> {
    let server = servers::get_server(&state.db, &id).await?;
    filesystem::write_text(PathBuf::from(server.path).as_path(), &rel, &contents).await?;
    queue_share(&state, &id);
    Ok(())
}

#[tauri::command]
pub async fn create_folder(state: State<'_, Arc<AppState>>, id: String, rel: String) -> AppResult<()> {
    let server = servers::get_server(&state.db, &id).await?;
    filesystem::create_dir(PathBuf::from(server.path).as_path(), &rel).await?;
    queue_share(&state, &id);
    Ok(())
}

#[tauri::command]
pub async fn rename_file(state: State<'_, Arc<AppState>>, id: String, from: String, to: String) -> AppResult<()> {
    let server = servers::get_server(&state.db, &id).await?;
    filesystem::rename(PathBuf::from(server.path).as_path(), &from, &to).await?;
    queue_share(&state, &id);
    Ok(())
}

#[tauri::command]
pub async fn delete_file(state: State<'_, Arc<AppState>>, id: String, rel: String) -> AppResult<()> {
    let server = servers::get_server(&state.db, &id).await?;
    filesystem::remove(PathBuf::from(server.path).as_path(), &rel).await?;
    queue_share(&state, &id);
    Ok(())
}

#[tauri::command]
pub async fn search_files(state: State<'_, Arc<AppState>>, id: String, query: String) -> AppResult<Vec<FileEntry>> {
    let server = servers::get_server(&state.db, &id).await?;
    Ok(filesystem::search(PathBuf::from(server.path).as_path(), &query))
}

#[tauri::command]
pub async fn upload_into(
    state: State<'_, Arc<AppState>>,
    id: String,
    rel: String,
    sources: Vec<String>,
) -> AppResult<u32> {
    let server = servers::get_server(&state.db, &id).await?;
    let count = filesystem::import_paths(PathBuf::from(server.path).as_path(), &rel, sources).await?;
    queue_share(&state, &id);
    Ok(count)
}

#[tauri::command]
pub async fn get_properties(state: State<'_, Arc<AppState>>, id: String) -> AppResult<ServerProperties> {
    let server = servers::get_server(&state.db, &id).await?;
    read_properties_file(&PathBuf::from(server.path).join("server.properties")).await
}

#[tauri::command]
pub async fn save_properties(
    state: State<'_, Arc<AppState>>,
    id: String,
    updates: BTreeMap<String, String>,
) -> AppResult<()> {
    let server = servers::get_server(&state.db, &id).await?;
    write_properties_file(&PathBuf::from(&server.path).join("server.properties"), &updates).await?;
    if let Some(port) = updates.get("server-port").and_then(|p| p.parse::<i64>().ok()) {
        sqlx::query("UPDATE servers SET port = ?, updated_at = ? WHERE id = ?")
            .bind(port)
            .bind(chrono::Utc::now().to_rfc3339())
            .bind(&id)
            .execute(&state.db)
            .await?;
    }
    if let Some(motd) = updates.get("motd") {
        sqlx::query("UPDATE servers SET motd = ?, updated_at = ? WHERE id = ?")
            .bind(motd)
            .bind(chrono::Utc::now().to_rfc3339())
            .bind(&id)
            .execute(&state.db)
            .await?;
    }
    // Push a changed player cap straight into the running server handle so the
    // Overview player denominator updates within seconds — no restart needed
    // for the UI (the game server itself picks it up on next start).
    if let Some(max) = updates.get("max-players").and_then(|v| v.parse::<u32>().ok()) {
        if let Some(handle) = state.processes.get(&id) {
            handle.max_players.store(max, Ordering::SeqCst);
        }
    }
    queue_share(&state, &id);
    Ok(())
}

#[tauri::command]
pub async fn list_backups(state: State<'_, Arc<AppState>>, id: String) -> AppResult<Vec<BackupRecord>> {
    backup::list_backups(&state.db, &id).await
}

#[tauri::command]
pub async fn create_backup(state: State<'_, Arc<AppState>>, id: String, trigger: String) -> AppResult<BackupRecord> {
    let server = servers::get_server(&state.db, &id).await?;
    backup::create_backup(&state.db, &id, PathBuf::from(server.path).as_path(), &trigger, None).await
}

#[tauri::command]
pub async fn restore_backup(state: State<'_, Arc<AppState>>, id: String, backup_id: String) -> AppResult<()> {
    let _ = process::stop_server(&state, &id, true).await;
    let server = servers::get_server(&state.db, &id).await?;
    let backups = backup::list_backups(&state.db, &id).await?;
    let item = backups
        .into_iter()
        .find(|b| b.id == backup_id)
        .ok_or_else(|| crate::error::AppError::new("Backup not found", "That backup no longer exists."))?;
    backup::restore_backup(PathBuf::from(server.path).as_path(), PathBuf::from(item.path).as_path()).await?;
    queue_share(&state, &id);
    Ok(())
}

#[tauri::command]
pub async fn delete_backup(state: State<'_, Arc<AppState>>, backup_id: String) -> AppResult<()> {
    backup::delete_backup(&state.db, &backup_id).await
}

#[tauri::command]
pub async fn list_worlds(state: State<'_, Arc<AppState>>, id: String) -> AppResult<Vec<WorldInfo>> {
    let server = servers::get_server(&state.db, &id).await?;
    Ok(system::list_worlds(PathBuf::from(server.path).as_path()))
}

#[tauri::command]
pub async fn list_addons(state: State<'_, Arc<AppState>>, id: String, kind: String) -> AppResult<Vec<InstalledAddon>> {
    let server = servers::get_server(&state.db, &id).await?;
    addons::list_installed(PathBuf::from(server.path).as_path(), &kind).await
}

#[tauri::command]
pub async fn search_plugins(
    state: State<'_, Arc<AppState>>,
    query: String,
    minecraft_version: String,
    provider: String,
) -> AppResult<Vec<AddonInfo>> {
    addons::search_plugins(&state.http, &query, &minecraft_version, &provider).await
}

#[tauri::command]
pub async fn search_mods(
    state: State<'_, Arc<AppState>>,
    query: String,
    loader: String,
    minecraft_version: Option<String>,
) -> AppResult<Vec<AddonInfo>> {
    addons::search_mods(&state.http, &query, &loader, minecraft_version.as_deref().unwrap_or("")).await
}

#[tauri::command]
pub async fn plugin_versions(
    state: State<'_, Arc<AppState>>,
    author: String,
    slug: String,
    minecraft_version: Option<String>,
    platform: Option<String>,
) -> AppResult<Vec<AddonVersion>> {
    addons::plugin_versions(
        &state.http,
        &author,
        &slug,
        minecraft_version.as_deref().unwrap_or(""),
        platform.as_deref().unwrap_or("PAPER"),
    )
    .await
}

#[tauri::command]
pub async fn resolve_plugin_download(
    state: State<'_, Arc<AppState>>,
    source: String,
    id: String,
    author: String,
    slug: String,
    minecraft_version: String,
    provider: String,
) -> AppResult<AddonVersion> {
    addons::resolve_plugin_download(&state.http, &source, &id, &author, &slug, &minecraft_version, &provider).await
}

#[tauri::command]
pub async fn mod_versions(
    state: State<'_, Arc<AppState>>,
    id: String,
    loader: String,
    game: String,
) -> AppResult<Vec<AddonVersion>> {
    addons::mod_versions(&state.http, &id, &loader, &game).await
}

#[tauri::command]
pub async fn install_addon(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    server_id: String,
    kind: String,
    url: String,
    file_name: String,
    label: String,
) -> AppResult<()> {
    let server = servers::get_server(&state.db, &server_id).await?;
    let folder = if kind == "mods" { "mods" } else { "plugins" };
    let dest = PathBuf::from(server.path).join(folder).join(file_name);
    addons::install_from_url(app, &state.downloads, &state.http, &url, &dest, &label).await?;
    queue_share(&state, &server_id);
    Ok(())
}

#[tauri::command]
pub async fn toggle_addon(state: State<'_, Arc<AppState>>, path: String, enabled: bool) -> AppResult<()> {
    addons::set_enabled(PathBuf::from(&path).as_path(), enabled).await?;
    if let Some(id) = share::server_id_for_child_path(&state.db, PathBuf::from(&path).as_path()).await? {
        queue_share(&state, &id);
    }
    Ok(())
}

#[tauri::command]
pub async fn remove_addon(state: State<'_, Arc<AppState>>, path: String) -> AppResult<()> {
    let child = PathBuf::from(&path);
    let id = share::server_id_for_child_path(&state.db, child.as_path()).await?;
    addons::remove_addon(child.as_path()).await?;
    if let Some(id) = id {
        queue_share(&state, &id);
    }
    Ok(())
}

#[tauri::command]
pub async fn scan_servers(path: String) -> AppResult<Vec<DiscoveredServer>> {
    discovery::scan(PathBuf::from(path).as_path()).await
}

#[tauri::command]
pub async fn import_server(
    state: State<'_, Arc<AppState>>,
    discovered: DiscoveredServer,
) -> AppResult<ServerRecord> {
    let settings = settings::load(&state.db).await?;
    let server = servers::import_server(&state.db, discovered, settings.preferred_java_path).await?;
    queue_share(&state, &server.id);
    Ok(server)
}

#[tauri::command]
pub async fn import_share_pack(
    state: State<'_, Arc<AppState>>,
    pack_path: String,
    dest_dir: Option<String>,
) -> AppResult<ServerRecord> {
    share::import_share_pack(&state, PathBuf::from(pack_path).as_path(), dest_dir.map(PathBuf::from)).await
}

#[tauri::command]
pub async fn get_share_pack(state: State<'_, Arc<AppState>>, id: String) -> AppResult<SharePackInfo> {
    share::ensure_share_pack(&state, &id).await
}

#[tauri::command]
pub async fn load_settings(state: State<'_, Arc<AppState>>) -> AppResult<AppSettings> {
    settings::load(&state.db).await
}

#[tauri::command]
pub async fn save_settings(state: State<'_, Arc<AppState>>, value: AppSettings) -> AppResult<()> {
    settings::save(&state.db, &value).await
}

#[tauri::command]
pub async fn set_auto_start(state: State<'_, Arc<AppState>>, id: String, auto_start: bool) -> AppResult<()> {
    servers::set_auto_start(&state.db, &id, auto_start).await?;
    queue_share(&state, &id);
    Ok(())
}

#[tauri::command]
pub async fn update_memory(state: State<'_, Arc<AppState>>, id: String, min_mb: u32, max_mb: u32) -> AppResult<()> {
    servers::update_server_memory(&state.db, &id, min_mb as i64, max_mb as i64).await?;
    queue_share(&state, &id);
    Ok(())
}

#[tauri::command]
pub fn list_downloads(state: State<'_, Arc<AppState>>) -> Vec<DownloadProgress> {
    state.downloads.list()
}

#[tauri::command]
pub async fn open_path(path: String) -> AppResult<()> {
    let as_path = PathBuf::from(&path);
    let mut cmd = if cfg!(target_os = "windows") {
        let mut c = std::process::Command::new("explorer");
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            c.creation_flags(0x0800_0000);
        }
        if as_path.is_file() {
            c.arg(format!("/select,{path}"));
        } else {
            c.arg(&path);
        }
        c
    } else if cfg!(target_os = "macos") {
        let mut c = std::process::Command::new("open");
        if as_path.is_file() {
            c.args(["-R", &path]);
        } else {
            c.arg(&path);
        }
        c
    } else {
        let mut c = std::process::Command::new("xdg-open");
        if as_path.is_file() {
            if let Some(parent) = as_path.parent() {
                c.arg(parent);
            } else {
                c.arg(&path);
            }
        } else {
            c.arg(&path);
        }
        c
    };
    cmd.spawn()?;
    Ok(())
}

#[tauri::command]
pub async fn open_logs(state: State<'_, Arc<AppState>>) -> AppResult<String> {
    Ok(state.logs_dir.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn diagnostic_report(state: State<'_, Arc<AppState>>) -> AppResult<String> {
    let servers = servers::list_servers(&state.db).await?;
    let javas = java::detect_java().await.unwrap_or_default();
    let info = system::system_info(false);
    let report = serde_json::json!({
        "appVersion": info.app_version,
        "os": info.os,
        "arch": info.arch,
        "java": javas.iter().map(|j| serde_json::json!({
            "path": j.path, "version": j.version, "vendor": j.vendor
        })).collect::<Vec<_>>(),
        "servers": servers.iter().map(|s| serde_json::json!({
            "id": s.id, "name": s.name, "provider": s.provider, "version": s.minecraft_version, "status": s.status
        })).collect::<Vec<_>>(),
    });
    Ok(serde_json::to_string_pretty(&report)?)
}

#[tauri::command]
pub async fn default_server_path(state: State<'_, Arc<AppState>>, name: String) -> AppResult<String> {
    let settings = settings::load(&state.db).await?;
    Ok(discovery::suggested_path(PathBuf::from(settings.default_servers_dir).as_path(), &name)
        .to_string_lossy()
        .to_string())
}
