mod commands;
mod database;
mod error;
mod models;
mod providers;
mod services;
mod state;
mod utils;

use std::sync::Arc;
use tauri::Manager;
use tracing_subscriber::EnvFilter;

use crate::services::monitoring;
use crate::state::AppState;
use crate::utils::http::http_client;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let handle = app.handle().clone();
            let data_dir = handle.path().app_data_dir()?;
            let logs_dir = data_dir.join("logs");
            std::fs::create_dir_all(&logs_dir)?;
            let db_path = data_dir.join("serverforge.db");
            let http = http_client().expect("http client");
            let db = tauri::async_runtime::block_on(database::connect(&db_path))?;
            let state = Arc::new(AppState::new(db, http, data_dir, logs_dir));
            monitoring::spawn_metrics_loop(handle.clone(), state.clone());
            let autostart_state = state.clone();
            let autostart_handle = handle.clone();
            tauri::async_runtime::spawn(async move {
                if let Ok(servers) = services::servers::list_servers(&autostart_state.db).await {
                    for server in servers.into_iter().filter(|s| s.auto_start == 1) {
                        let _ = services::process::start_server(
                            autostart_handle.clone(),
                            &autostart_state,
                            &server,
                        )
                        .await;
                    }
                }
            });
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::provider_catalog,
            commands::list_minecraft_versions,
            commands::list_builds,
            commands::provider_compatibility,
            commands::list_servers,
            commands::get_server,
            commands::validate_name,
            commands::inspect_directory,
            commands::create_server,
            commands::cleanup_installation,
            commands::start_server,
            commands::stop_server,
            commands::restart_server,
            commands::send_command,
            commands::delete_server,
            commands::detect_java,
            commands::test_java,
            commands::java_download_url,
            commands::system_snapshot,
            commands::recommended_java,
            commands::memory_advice,
            commands::clamp_memory,
            commands::port_in_use,
            commands::list_files,
            commands::read_file,
            commands::write_file,
            commands::create_folder,
            commands::rename_file,
            commands::delete_file,
            commands::search_files,
            commands::upload_into,
            commands::get_properties,
            commands::save_properties,
            commands::list_backups,
            commands::create_backup,
            commands::restore_backup,
            commands::delete_backup,
            commands::list_worlds,
            commands::list_addons,
            commands::search_plugins,
            commands::search_mods,
            commands::plugin_versions,
            commands::resolve_plugin_download,
            commands::mod_versions,
            commands::install_addon,
            commands::toggle_addon,
            commands::remove_addon,
            commands::scan_servers,
            commands::import_server,
            commands::import_share_pack,
            commands::get_share_pack,
            commands::load_settings,
            commands::save_settings,
            commands::set_auto_start,
            commands::update_memory,
            commands::list_downloads,
            commands::open_path,
            commands::open_logs,
            commands::diagnostic_report,
            commands::default_server_path,
        ])
        .run(tauri::generate_context!())
        .expect("error while running ServerForge");
}
