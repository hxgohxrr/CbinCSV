mod cfgbin;
mod crc32;
mod csv_handler;
mod rdbn;
mod commands;
mod format_handler;
mod mcp_server;

use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use crate::csv_handler::SessionData;

pub struct McpHandle(pub Arc<Mutex<Option<tokio::task::AbortHandle>>>);

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(Arc::new(RwLock::new(SessionData { files: vec![] })) as Arc<RwLock<SessionData>>)
        .manage(McpHandle(Arc::new(Mutex::new(None))))
        .invoke_handler(tauri::generate_handler![
            commands::parse_files,
            commands::parse_folder,
            commands::export_formatted,
            commands::import_formatted,
            commands::write_cfgbin,
            commands::get_system_locale,
            commands::sync_session,
            commands::start_mcp_server,
            commands::stop_mcp_server,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
