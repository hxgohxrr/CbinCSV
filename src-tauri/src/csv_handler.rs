use std::collections::HashMap;
use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use crate::cfgbin::TextEntry;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ParseMode { Standard, Nnk }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub path: String,
    pub ev_name: String,
    pub language: String,
    pub mode: ParseMode,
    pub entries: Vec<TextEntry>,
    pub addresses: Option<Vec<u32>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    pub files: Vec<FileEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CsvFormat { Single, PerFile }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressPayload {
    pub current: usize,
    pub total: usize,
    pub file: String,
}

pub fn extract_language_from_path(path: &str) -> Option<String> {
    let stem = std::path::Path::new(path)
        .file_name()?
        .to_str()?;
    let without_cfg_bin = stem.strip_suffix(".cfg.bin")?;
    let lang = without_cfg_bin.split('_').last()?;
    if lang.len() == 2 || lang.len() == 3 {
        Some(lang.to_lowercase())
    } else {
        None
    }
}

pub fn extract_ev_name_from_path(path: &str) -> Option<String> {
    let stem = std::path::Path::new(path)
        .file_name()?
        .to_str()?;
    let without_cfg_bin = stem.strip_suffix(".cfg.bin")?;
    let parts: Vec<&str> = without_cfg_bin.rsplitn(2, '_').collect();
    if parts.len() == 2 {
        Some(parts[1].to_string())
    } else {
        None
    }
}

pub fn build_csv(
    session: &SessionData,
    langs: &[String],
    separator: char,
) -> Result<String> {
    let mut wtr = csv::WriterBuilder::new()
        .delimiter(separator as u8)
        .from_writer(vec![]);

    let mut header = vec!["EV_NAME".to_string(), "INDEX".to_string()];
    for lang in langs {
        header.push(lang.to_uppercase());
    }
    wtr.write_record(&header)?;

    let mut pairs: Vec<(String, usize)> = Vec::new();
    for f in &session.files {
        if !langs.contains(&f.language) { continue; }
        for e in &f.entries {
            let pair = (f.ev_name.clone(), e.index);
            if !pairs.contains(&pair) {
                pairs.push(pair);
            }
        }
    }

    for (ev_name, index) in &pairs {
        let mut row = vec![ev_name.clone(), index.to_string()];
        for lang in langs {
            let val = session.files.iter()
                .find(|f| &f.ev_name == ev_name && &f.language == lang)
                .and_then(|f| f.entries.get(*index))
                .map(|e| e.value.clone())
                .unwrap_or_default();
            row.push(val);
        }
        wtr.write_record(&row)?;
    }

    let bytes = wtr.into_inner()?;
    Ok(String::from_utf8(bytes)?)
}

pub fn parse_csv_rows(
    csv_str: &str,
    separator: char,
) -> Result<Vec<(String, usize, HashMap<String, String>)>> {
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(separator as u8)
        .from_reader(csv_str.as_bytes());

    let headers: Vec<String> = rdr.headers()?
        .iter()
        .map(|s| s.to_string())
        .collect();

    if headers.len() < 3 || headers[0] != "EV_NAME" || headers[1] != "INDEX" {
        bail!("CSV header must start with EV_NAME;INDEX");
    }

    let lang_cols: Vec<String> = headers[2..].iter()
        .map(|s| s.to_lowercase())
        .collect();

    let mut rows = Vec::new();
    for result in rdr.records() {
        let record = result?;
        let ev_name = record.get(0).unwrap_or("").to_string();
        let index: usize = record.get(1).unwrap_or("0").parse()?;
        let mut map = HashMap::new();
        for (i, lang) in lang_cols.iter().enumerate() {
            let val = record.get(i + 2).unwrap_or("").to_string();
            map.insert(lang.clone(), val);
        }
        rows.push((ev_name, index, map));
    }
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_language_ja() {
        assert_eq!(
            extract_language_from_path("ev00_0010_ja.cfg.bin"),
            Some("ja".to_string())
        );
    }

    #[test]
    fn extracts_language_es() {
        assert_eq!(
            extract_language_from_path("C:/files/ev00_1010_es.cfg.bin"),
            Some("es".to_string())
        );
    }

    #[test]
    fn extracts_ev_name() {
        assert_eq!(
            extract_ev_name_from_path("ev00_0010_ja.cfg.bin"),
            Some("ev00_0010".to_string())
        );
    }

    #[test]
    fn build_and_parse_csv_roundtrip() {
        let session = SessionData {
            files: vec![
                FileEntry {
                    path: "ev00_0010_ja.cfg.bin".to_string(),
                    ev_name: "ev00_0010".to_string(),
                    language: "ja".to_string(),
                    mode: ParseMode::Standard,
                    entries: vec![
                        TextEntry { index: 0, entry: "A".to_string(), variable_index: 0, value: "日本語".to_string() },
                        TextEntry { index: 1, entry: "B".to_string(), variable_index: 0, value: "テスト".to_string() },
                    ],
                    addresses: None,
                },
                FileEntry {
                    path: "ev00_0010_es.cfg.bin".to_string(),
                    ev_name: "ev00_0010".to_string(),
                    language: "es".to_string(),
                    mode: ParseMode::Standard,
                    entries: vec![
                        TextEntry { index: 0, entry: "A".to_string(), variable_index: 0, value: "Hola".to_string() },
                        TextEntry { index: 1, entry: "B".to_string(), variable_index: 0, value: "".to_string() },
                    ],
                    addresses: None,
                },
            ],
        };

        let csv = build_csv(&session, &["ja".to_string(), "es".to_string()], ';').unwrap();
        let rows = parse_csv_rows(&csv, ';').unwrap();

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].0, "ev00_0010");
        assert_eq!(rows[0].1, 0);
        assert_eq!(rows[0].2["ja"], "日本語");
        assert_eq!(rows[0].2["es"], "Hola");
        assert_eq!(rows[1].2["es"], "");
    }
}
