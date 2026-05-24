use std::collections::HashMap;
use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use crate::csv_handler::SessionData;

#[derive(Debug, Clone)]
pub struct ExportRow {
    pub ev_name: String,
    pub index: usize,
    pub field_type: String,
    pub values: Vec<(String, String)>, // (lang, value) — ordered
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportFormat { Csv, Json, Yaml, Toml, Xml }

pub fn build_rows(session: &SessionData, langs: &[String]) -> Vec<ExportRow> {
    let mut pairs: Vec<(String, usize)> = Vec::new();
    for f in &session.files {
        if !langs.contains(&f.language) { continue; }
        for e in &f.entries {
            let key = (f.ev_name.clone(), e.index);
            if !pairs.contains(&key) { pairs.push(key); }
        }
    }
    pairs.iter().map(|(ev_name, index)| {
        let field_type = session.files.iter()
            .find(|f| &f.ev_name == ev_name && langs.contains(&f.language))
            .and_then(|f| f.entries.iter().find(|e| e.index == *index))
            .map(|e| e.field_type.clone())
            .unwrap_or_else(|| "string".to_string());
        let values = langs.iter().map(|lang| {
            let val = session.files.iter()
                .find(|f| &f.ev_name == ev_name && &f.language == lang)
                .and_then(|f| f.entries.iter().find(|e| e.index == *index))
                .map(|e| e.value.clone())
                .unwrap_or_default();
            (lang.clone(), val)
        }).collect();
        ExportRow { ev_name: ev_name.clone(), index: *index, field_type, values }
    }).collect()
}

pub fn format_from_extension(path: &str) -> ExportFormat {
    let ext = std::path::Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    match ext.as_str() {
        "json" => ExportFormat::Json,
        "yaml" | "yml" => ExportFormat::Yaml,
        "toml" => ExportFormat::Toml,
        "xml" => ExportFormat::Xml,
        _ => ExportFormat::Csv,
    }
}

pub fn file_extension(format: ExportFormat) -> &'static str {
    match format {
        ExportFormat::Csv  => "csv",
        ExportFormat::Json => "json",
        ExportFormat::Yaml => "yaml",
        ExportFormat::Toml => "toml",
        ExportFormat::Xml  => "xml",
    }
}

pub(crate) fn row_to_json(row: &ExportRow) -> serde_json::Value {
    let mut obj = serde_json::json!({
        "ev_name": row.ev_name,
        "index": row.index,
        "type": row.field_type,
    });
    for (lang, val) in &row.values {
        obj[lang] = serde_json::Value::String(val.clone());
    }
    obj
}

pub(crate) fn json_to_row(val: &serde_json::Value, known_langs: &[String]) -> Option<ExportRow> {
    let ev_name = val["ev_name"].as_str()?.to_string();
    let index = val["index"].as_u64()? as usize;
    let field_type = val["type"].as_str().unwrap_or("string").to_string();
    let values = known_langs.iter()
        .filter_map(|lang| {
            val[lang.as_str()].as_str().map(|v| (lang.clone(), v.to_string()))
        }).collect();
    Some(ExportRow { ev_name, index, field_type, values })
}

pub fn serialize_rows(rows: &[ExportRow], format: ExportFormat, separator: char) -> Result<String> {
    match format {
        ExportFormat::Csv => {
            let mut wtr = csv::WriterBuilder::new()
                .delimiter(separator as u8)
                .from_writer(vec![]);
            if let Some(first) = rows.first() {
                let mut header = vec!["EV_NAME".to_string(), "TYPE".to_string(), "INDEX".to_string()];
                for (lang, _) in &first.values { header.push(lang.to_uppercase()); }
                wtr.write_record(&header)?;
            }
            for row in rows {
                let mut record = vec![row.ev_name.clone(), row.field_type.clone(), row.index.to_string()];
                for (_, val) in &row.values { record.push(val.clone()); }
                wtr.write_record(&record)?;
            }
            Ok(String::from_utf8(wtr.into_inner()?)?)
        }
        ExportFormat::Json => {
            let arr: Vec<serde_json::Value> = rows.iter().map(row_to_json).collect();
            Ok(serde_json::to_string_pretty(&arr)?)
        }
        ExportFormat::Yaml => {
            let arr: Vec<serde_json::Value> = rows.iter().map(row_to_json).collect();
            Ok(serde_yaml::to_string(&arr)?)
        }
        ExportFormat::Toml => {
            #[derive(Serialize)]
            struct TomlRow {
                ev_name: String,
                index: usize,
                #[serde(rename = "type")]
                field_type: String,
                values: HashMap<String, String>,
            }
            #[derive(Serialize)]
            struct TomlDoc { entry: Vec<TomlRow> }
            let entries = rows.iter().map(|r| TomlRow {
                ev_name: r.ev_name.clone(),
                index: r.index,
                field_type: r.field_type.clone(),
                values: r.values.iter().cloned().collect(),
            }).collect();
            Ok(toml::to_string_pretty(&TomlDoc { entry: entries })?)
        }
        ExportFormat::Xml => {
            let mut out = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<entries>\n");
            for row in rows {
                out.push_str(&format!(
                    "  <entry ev_name=\"{}\" index=\"{}\" type=\"{}\">\n",
                    xml_escape(&row.ev_name), row.index, xml_escape(&row.field_type)
                ));
                for (lang, val) in &row.values {
                    out.push_str(&format!(
                        "    <lang code=\"{}\">{}</lang>\n",
                        xml_escape(lang), xml_escape(val)
                    ));
                }
                out.push_str("  </entry>\n");
            }
            out.push_str("</entries>");
            Ok(out)
        }
    }
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
     .replace('<', "&lt;")
     .replace('>', "&gt;")
     .replace('"', "&quot;")
     .replace('\'', "&apos;")
}

pub fn deserialize_rows(content: &str, format: ExportFormat, separator: char) -> Result<Vec<ExportRow>> {
    match format {
        ExportFormat::Csv => {
            let mut rdr = csv::ReaderBuilder::new()
                .delimiter(separator as u8)
                .from_reader(content.as_bytes());
            let headers: Vec<String> = rdr.headers()?.iter().map(|s| s.to_string()).collect();
            if headers.is_empty() || headers[0] != "EV_NAME" {
                bail!("CSV header must start with EV_NAME");
            }
            let has_type = headers.get(1).map(|s| s == "TYPE").unwrap_or(false);
            let idx_col = if has_type { 2 } else { 1 };
            let lang_start = idx_col + 1;
            let langs: Vec<String> = headers[lang_start..].iter().map(|s| s.to_lowercase()).collect();
            let mut rows = Vec::new();
            for result in rdr.records() {
                let rec = result?;
                let ev_name = rec.get(0).unwrap_or("").to_string();
                let field_type = if has_type { rec.get(1).unwrap_or("string").to_string() } else { "string".to_string() };
                let index: usize = rec.get(idx_col).unwrap_or("0").parse().unwrap_or(0);
                let values = langs.iter().enumerate()
                    .map(|(i, lang)| (lang.clone(), rec.get(lang_start + i).unwrap_or("").to_string()))
                    .collect();
                rows.push(ExportRow { ev_name, index, field_type, values });
            }
            Ok(rows)
        }
        ExportFormat::Json => {
            let arr: Vec<serde_json::Value> = serde_json::from_str(content)?;
            let langs = detect_langs_from_json(&arr);
            Ok(arr.iter().filter_map(|v| json_to_row(v, &langs)).collect())
        }
        ExportFormat::Yaml => {
            let arr: Vec<serde_json::Value> = serde_yaml::from_str(content)?;
            let langs = detect_langs_from_json(&arr);
            Ok(arr.iter().filter_map(|v| json_to_row(v, &langs)).collect())
        }
        ExportFormat::Toml => {
            #[derive(Deserialize)]
            struct TomlRow {
                ev_name: String,
                index: usize,
                #[serde(rename = "type")]
                field_type: String,
                values: HashMap<String, String>,
            }
            #[derive(Deserialize)]
            struct TomlDoc { entry: Vec<TomlRow> }
            let doc: TomlDoc = toml::from_str(content)?;
            Ok(doc.entry.into_iter().map(|r| {
                let mut v: Vec<(String, String)> = r.values.into_iter().collect();
                v.sort_by(|a, b| a.0.cmp(&b.0));
                ExportRow { ev_name: r.ev_name, index: r.index, field_type: r.field_type, values: v }
            }).collect())
        }
        ExportFormat::Xml => parse_xml(content),
    }
}

fn detect_langs_from_json(arr: &[serde_json::Value]) -> Vec<String> {
    let reserved = ["ev_name", "index", "type"];
    if let Some(first) = arr.first() {
        if let Some(obj) = first.as_object() {
            return obj.keys()
                .filter(|k| !reserved.contains(&k.as_str()))
                .cloned().collect();
        }
    }
    Vec::new()
}

fn parse_xml(content: &str) -> Result<Vec<ExportRow>> {
    let mut rows = Vec::new();
    let mut current: Option<ExportRow> = None;
    let mut current_lang: Option<String> = None;
    let mut text_buf = String::new();

    let mut reader = quick_xml::Reader::from_str(content);
    reader.config_mut().trim_text(true);

    loop {
        match reader.read_event()? {
            quick_xml::events::Event::Start(e) => {
                match e.name().as_ref() {
                    b"entry" => {
                        let mut ev_name = String::new();
                        let mut index = 0usize;
                        let mut field_type = "string".to_string();
                        for attr in e.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"ev_name" => ev_name = String::from_utf8_lossy(&attr.value).to_string(),
                                b"index"   => index = String::from_utf8_lossy(&attr.value).parse().unwrap_or(0),
                                b"type"    => field_type = String::from_utf8_lossy(&attr.value).to_string(),
                                _ => {}
                            }
                        }
                        current = Some(ExportRow { ev_name, index, field_type, values: Vec::new() });
                    }
                    b"lang" => {
                        current_lang = e.attributes().flatten()
                            .find(|a| a.key.as_ref() == b"code")
                            .map(|a| String::from_utf8_lossy(&a.value).to_string());
                        text_buf.clear();
                    }
                    _ => {}
                }
            }
            quick_xml::events::Event::Text(e) => {
                text_buf.push_str(&e.unescape()?);
            }
            quick_xml::events::Event::End(e) => {
                match e.name().as_ref() {
                    b"lang" => {
                        if let (Some(row), Some(lang)) = (current.as_mut(), current_lang.take()) {
                            row.values.push((lang, text_buf.clone()));
                        }
                        text_buf.clear();
                    }
                    b"entry" => {
                        if let Some(row) = current.take() { rows.push(row); }
                    }
                    _ => {}
                }
            }
            quick_xml::events::Event::Eof => break,
            _ => {}
        }
    }
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::csv_handler::{FileEntry, ParseMode};
    use crate::cfgbin::TextEntry;

    fn sample_session() -> SessionData {
        SessionData { files: vec![
            FileEntry {
                path: "ev00_0010_ja.cfg.bin".into(),
                ev_name: "ev00_0010".into(),
                language: "ja".into(),
                mode: ParseMode::Standard,
                addresses: None,
                entries: vec![
                    TextEntry { index: 0, entry: "A".into(), variable_index: 0, field_type: "string".into(), value: "日本語".into() },
                    TextEntry { index: 1, entry: "B".into(), variable_index: 0, field_type: "int".into(), value: "42".into() },
                ],
            },
            FileEntry {
                path: "ev00_0010_en.cfg.bin".into(),
                ev_name: "ev00_0010".into(),
                language: "en".into(),
                mode: ParseMode::Standard,
                addresses: None,
                entries: vec![
                    TextEntry { index: 0, entry: "A".into(), variable_index: 0, field_type: "string".into(), value: "Japanese".into() },
                    TextEntry { index: 1, entry: "B".into(), variable_index: 0, field_type: "int".into(), value: "42".into() },
                ],
            },
        ]}
    }

    fn roundtrip(format: ExportFormat) {
        let session = sample_session();
        let langs = vec!["ja".to_string(), "en".to_string()];
        let rows = build_rows(&session, &langs);
        let serialized = serialize_rows(&rows, format, ';').unwrap();
        let parsed = deserialize_rows(&serialized, format, ';').unwrap();
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].ev_name, "ev00_0010");
        assert_eq!(parsed[0].index, 0);
        let ja = parsed[0].values.iter().find(|(l, _)| l == "ja").map(|(_, v)| v.as_str()).unwrap_or("");
        let en = parsed[0].values.iter().find(|(l, _)| l == "en").map(|(_, v)| v.as_str()).unwrap_or("");
        assert_eq!(ja, "日本語");
        assert_eq!(en, "Japanese");
    }

    #[test] fn roundtrip_csv()  { roundtrip(ExportFormat::Csv); }
    #[test] fn roundtrip_json() { roundtrip(ExportFormat::Json); }
    #[test] fn roundtrip_yaml() { roundtrip(ExportFormat::Yaml); }
    #[test] fn roundtrip_toml() { roundtrip(ExportFormat::Toml); }
    #[test] fn roundtrip_xml()  { roundtrip(ExportFormat::Xml); }
}
