mod cfgbin;
mod crc32;
mod csv_handler;
mod commands;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::parse_files,
            commands::parse_folder,
            commands::export_csv,
            commands::import_csv_rows,
            commands::write_cfgbin,
            commands::get_system_locale,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
