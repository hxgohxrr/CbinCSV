use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;

use tauri::AppHandle;
use tauri::Emitter;

use crate::cfgbin::CfgBin;
use crate::csv_handler::{
    extract_ev_name_from_path, extract_language_from_path,
    CsvFormat, FileEntry, ParseMode, ProgressPayload, SessionData,
};
use crate::format_handler::{build_rows, serialize_rows, deserialize_rows, format_from_extension, file_extension, ExportFormat};
use crate::rdbn;

#[tauri::command]
pub fn parse_files(
    app: AppHandle,
    paths: Vec<String>,
    mode: ParseMode,
) -> Result<Vec<FileEntry>, String> {
    let total = paths.len();
    let mut files = Vec::new();

    for (i, path) in paths.iter().enumerate() {
        let _ = app.emit("progress", ProgressPayload {
            current: i,
            total,
            file: path.clone(),
        });

        let data = fs::read(path).map_err(|e| e.to_string())?;
        let ev_name = extract_ev_name_from_path(path)
            .unwrap_or_else(|| Path::new(path).file_stem().unwrap_or_default().to_string_lossy().to_string());
        let language = extract_language_from_path(path).unwrap_or_else(|| "??".to_string());

        if rdbn::is_rdbn(&data) {
            let rdbn_file = rdbn::RdbnFile::open(&data).map_err(|e| e.to_string())?;
            let entries = rdbn_file.extract_fields();
            files.push(FileEntry { path: path.clone(), ev_name, language, mode: ParseMode::Rdbn, entries, addresses: None });
            continue;
        }

        let (entries, addresses) = match mode {
            ParseMode::Standard | ParseMode::Rdbn => {
                let cfg = CfgBin::open(&data).map_err(|e| e.to_string())?;
                (cfg.extract_texts(), None)
            }
            ParseMode::Nnk => {
                let map = CfgBin::extract_texts_by_address(&data).map_err(|e| e.to_string())?;
                let addrs: Vec<u32> = map.keys().cloned().collect();
                let entries = map.into_iter().enumerate()
                    .map(|(i, (_, v))| crate::cfgbin::TextEntry {
                        index: i,
                        entry: String::new(),
                        variable_index: 0,
                        field_type: "string".to_string(),
                        value: v,
                    })
                    .collect();
                (entries, Some(addrs))
            }
        };

        files.push(FileEntry { path: path.clone(), ev_name, language, mode: mode.clone(), entries, addresses });
    }

    let _ = app.emit("progress", ProgressPayload { current: total, total, file: String::new() });
    Ok(files)
}

#[tauri::command]
pub fn parse_folder(
    app: AppHandle,
    folder: String,
    mode: ParseMode,
) -> Result<Vec<FileEntry>, String> {
    let paths: Vec<String> = fs::read_dir(&folder)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_name().to_string_lossy().ends_with(".cfg.bin")
        })
        .map(|e| e.path().to_string_lossy().to_string())
        .collect();

    parse_files(app, paths, mode)
}

#[tauri::command]
pub fn export_formatted(
    app: AppHandle,
    session: SessionData,
    output_path: String,
    langs: Vec<String>,
    format: CsvFormat,
    file_format: ExportFormat,
    separator: String,
) -> Result<(), String> {
    let sep = separator.chars().next().unwrap_or(';');
    let ext = file_extension(file_format);

    match format {
        CsvFormat::Single => {
            let rows = build_rows(&session, &langs);
            let content = serialize_rows(&rows, file_format, sep).map_err(|e| e.to_string())?;
            fs::write(&output_path, content.as_bytes()).map_err(|e| e.to_string())?;
        }
        CsvFormat::PerFile => {
            let ev_names: Vec<String> = {
                let mut seen = std::collections::HashSet::new();
                session.files.iter()
                    .filter(|f| seen.insert(f.ev_name.clone()))
                    .map(|f| f.ev_name.clone())
                    .collect()
            };
            let total = ev_names.len();
            for (i, ev_name) in ev_names.iter().enumerate() {
                let _ = app.emit("progress", ProgressPayload {
                    current: i, total, file: ev_name.clone(),
                });
                let sub = SessionData {
                    files: session.files.iter()
                        .filter(|f| &f.ev_name == ev_name)
                        .cloned()
                        .collect(),
                };
                let rows = build_rows(&sub, &langs);
                let content = serialize_rows(&rows, file_format, sep).map_err(|e| e.to_string())?;
                let out = Path::new(&output_path).join(format!("{}.{}", ev_name, ext));
                fs::write(out, content.as_bytes()).map_err(|e| e.to_string())?;
            }
            let _ = app.emit("progress", ProgressPayload { current: total, total, file: String::new() });
        }
    }
    Ok(())
}

#[tauri::command]
pub fn import_formatted(
    file_path: String,
    separator: String,
) -> Result<Vec<(String, String, usize, std::collections::HashMap<String, String>)>, String> {
    let sep = separator.chars().next().unwrap_or(';');
    let file_format = format_from_extension(&file_path);
    let content = fs::read_to_string(&file_path).map_err(|e| e.to_string())?;
    let rows = deserialize_rows(&content, file_format, sep).map_err(|e| e.to_string())?;
    Ok(rows.into_iter().map(|row| {
        let lang_map: std::collections::HashMap<String, String> = row.values.into_iter().collect();
        (row.ev_name, row.field_type, row.index, lang_map)
    }).collect())
}

#[tauri::command]
pub fn write_cfgbin(
    app: AppHandle,
    session: SessionData,
    output_dir: String,
) -> Result<Vec<String>, String> {
    let total = session.files.len();
    let mut written = Vec::new();

    for (i, file) in session.files.iter().enumerate() {
        let _ = app.emit("progress", ProgressPayload {
            current: i, total, file: file.path.clone(),
        });

        let data = fs::read(&file.path).map_err(|e| e.to_string())?;

        let output = match file.mode {
            ParseMode::Standard => {
                let mut cfg = CfgBin::open(&data).map_err(|e| e.to_string())?;
                cfg.update_texts(&file.entries);
                cfg.save()
            }
            ParseMode::Rdbn => {
                let mut rdbn_file = rdbn::RdbnFile::open(&data).map_err(|e| e.to_string())?;
                rdbn_file.update_fields(&file.entries);
                rdbn_file.save()
            }
            ParseMode::Nnk => {
                let addrs = file.addresses.as_ref()
                    .ok_or("NNK mode requires addresses")?;
                let map: BTreeMap<u32, String> = addrs.iter()
                    .zip(file.entries.iter())
                    .map(|(&addr, e)| (addr, e.value.clone()))
                    .collect();
                CfgBin::patch_texts_by_address_in_place(&data, &map)
                    .map_err(|e| e.to_string())?
            }
        };

        let filename = Path::new(&file.path)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy();
        let out_path = Path::new(&output_dir).join(filename.as_ref());
        fs::write(&out_path, &output).map_err(|e| e.to_string())?;
        written.push(out_path.to_string_lossy().to_string());
    }

    let _ = app.emit("progress", ProgressPayload { current: total, total, file: String::new() });
    Ok(written)
}

#[tauri::command]
pub fn get_system_locale() -> String {
    sys_locale::get_locale()
        .map(|l| l.to_lowercase())
        .unwrap_or_else(|| "es".to_string())
}

#[tauri::command]
pub async fn sync_session(
    session_state: tauri::State<'_, Arc<tokio::sync::RwLock<crate::csv_handler::SessionData>>>,
    session: crate::csv_handler::SessionData,
) -> Result<(), String> {
    let mut guard = session_state.write().await;
    *guard = session;
    Ok(())
}

#[tauri::command]
pub async fn start_mcp_server(
    app: tauri::AppHandle,
    port: u16,
    session_state: tauri::State<'_, Arc<tokio::sync::RwLock<crate::csv_handler::SessionData>>>,
    mcp_handle: tauri::State<'_, crate::McpHandle>,
) -> Result<(), String> {
    {
        let mut guard = mcp_handle.0.lock().await;
        if let Some(h) = guard.take() {
            h.abort();
        }
    }
    let state = crate::mcp_server::McpState {
        session: session_state.inner().clone(),
        app_handle: app,
    };
    let handle = crate::mcp_server::start(port, state).await;
    let mut guard = mcp_handle.0.lock().await;
    *guard = Some(handle);
    Ok(())
}

#[tauri::command]
pub async fn stop_mcp_server(
    mcp_handle: tauri::State<'_, crate::McpHandle>,
) -> Result<(), String> {
    let mut guard = mcp_handle.0.lock().await;
    if let Some(h) = guard.take() {
        h.abort();
    }
    Ok(())
}
