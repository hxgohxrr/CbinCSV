use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use tauri::AppHandle;
use tauri::Emitter;

use crate::cfgbin::CfgBin;
use crate::csv_handler::{
    build_csv, extract_ev_name_from_path, extract_language_from_path,
    parse_csv_rows, CsvFormat, FileEntry, ParseMode, ProgressPayload, SessionData,
};

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

        let (entries, addresses) = match mode {
            ParseMode::Standard => {
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
pub fn export_csv(
    app: AppHandle,
    session: SessionData,
    output_path: String,
    langs: Vec<String>,
    format: CsvFormat,
    separator: String,
) -> Result<(), String> {
    let sep = separator.chars().next().unwrap_or(';');
    let total = session.files.iter().map(|f| &f.ev_name).collect::<std::collections::HashSet<_>>().len();

    match format {
        CsvFormat::Single => {
            let csv = build_csv(&session, &langs, sep).map_err(|e| e.to_string())?;
            fs::write(&output_path, csv.as_bytes()).map_err(|e| e.to_string())?;
        }
        CsvFormat::PerFile => {
            let ev_names: Vec<String> = {
                let mut seen = std::collections::HashSet::new();
                session.files.iter()
                    .filter(|f| seen.insert(f.ev_name.clone()))
                    .map(|f| f.ev_name.clone())
                    .collect()
            };
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
                let csv = build_csv(&sub, &langs, sep).map_err(|e| e.to_string())?;
                let out = Path::new(&output_path).join(format!("{}.csv", ev_name));
                fs::write(out, csv.as_bytes()).map_err(|e| e.to_string())?;
            }
        }
    }
    let _ = app.emit("progress", ProgressPayload { current: total, total, file: String::new() });
    Ok(())
}

#[tauri::command]
pub fn import_csv_rows(
    csv_path: String,
    separator: String,
) -> Result<Vec<(String, usize, std::collections::HashMap<String, String>)>, String> {
    let sep = separator.chars().next().unwrap_or(';');
    let content = fs::read_to_string(&csv_path).map_err(|e| e.to_string())?;
    parse_csv_rows(&content, sep).map_err(|e| e.to_string())
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
