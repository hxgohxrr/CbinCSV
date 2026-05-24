use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tauri::Emitter;
use tokio::sync::RwLock;

use crate::csv_handler::SessionData;

#[derive(Clone)]
pub struct McpState {
    pub session: Arc<RwLock<SessionData>>,
    pub app_handle: tauri::AppHandle,
}

#[derive(Deserialize)]
pub struct JsonRpcRequest {
    pub id: Option<serde_json::Value>,
    pub method: String,
    pub params: Option<serde_json::Value>,
}

#[derive(Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: &'static str,
    pub id: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Serialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
}

pub fn router(state: McpState) -> axum::Router {
    use axum::routing::{get, post};
    use tower_http::cors::{CorsLayer, Any};

    let cors = CorsLayer::new()
        .allow_origin([
            "http://localhost".parse().unwrap(),
            "http://127.0.0.1".parse().unwrap(),
        ])
        .allow_methods(Any)
        .allow_headers(Any);

    axum::Router::new()
        .route("/mcp", post(handle_jsonrpc))
        .route("/mcp/sse", get(handle_sse))
        .with_state(state)
        .layer(cors)
}

pub async fn start(port: u16, state: McpState) -> tokio::task::AbortHandle {
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind MCP port");
    let app = router(state);
    let handle = tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    handle.abort_handle()
}

async fn handle_sse() -> impl axum::response::IntoResponse {
    use axum::response::sse::{Event, Sse};
    use futures_util::stream;
    use std::convert::Infallible;
    use tokio::time::Duration;

    let stream = stream::unfold((), |_| async {
        tokio::time::sleep(Duration::from_secs(30)).await;
        let event = Event::default().data("heartbeat");
        Some((Ok::<_, Infallible>(event), ()))
    });
    Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::default())
}

async fn handle_jsonrpc(
    axum::extract::State(state): axum::extract::State<McpState>,
    axum::extract::Json(req): axum::extract::Json<JsonRpcRequest>,
) -> axum::response::Json<JsonRpcResponse> {
    let result = match req.method.as_str() {
        "list_entries" => handle_list_entries(&state, req.params).await,
        "get_entry" => handle_get_entry(&state, req.params).await,
        "set_entry" => handle_set_entry(&state, req.params).await,
        "apply_translations" => handle_apply_translations(&state, req.params).await,
        "get_session_info" => handle_get_session_info(&state, req.params).await,
        _ => Err(JsonRpcError {
            code: -32601,
            message: "Method not found".to_string(),
        }),
    };
    axum::response::Json(match result {
        Ok(r) => JsonRpcResponse {
            jsonrpc: "2.0",
            id: req.id,
            result: Some(r),
            error: None,
        },
        Err(e) => JsonRpcResponse {
            jsonrpc: "2.0",
            id: req.id,
            result: None,
            error: Some(e),
        },
    })
}

async fn handle_list_entries(
    state: &McpState,
    params: Option<serde_json::Value>,
) -> Result<serde_json::Value, JsonRpcError> {
    let filter_ev = params.as_ref().and_then(|p| p["ev_name"].as_str().map(|s| s.to_string()));
    let filter_lang = params.as_ref().and_then(|p| p["lang"].as_str().map(|s| s.to_string()));

    let session = state.session.read().await;

    let mut pairs: Vec<(String, usize)> = Vec::new();
    for f in &session.files {
        if let Some(ref ev) = filter_ev {
            if &f.ev_name != ev {
                continue;
            }
        }
        if let Some(ref lang) = filter_lang {
            if &f.language != lang {
                continue;
            }
        }
        for e in &f.entries {
            let key = (f.ev_name.clone(), e.index);
            if !pairs.contains(&key) {
                pairs.push(key);
            }
        }
    }

    let entries: Vec<serde_json::Value> = pairs
        .iter()
        .map(|(ev_name, index)| {
            let field_type = session
                .files
                .iter()
                .find(|f| &f.ev_name == ev_name)
                .and_then(|f| f.entries.iter().find(|e| e.index == *index))
                .map(|e| e.field_type.clone())
                .unwrap_or_else(|| "string".to_string());

            let mut values = serde_json::Map::new();
            for f in &session.files {
                if &f.ev_name != ev_name {
                    continue;
                }
                if let Some(ref lang) = filter_lang {
                    if &f.language != lang {
                        continue;
                    }
                }
                if let Some(entry) = f.entries.iter().find(|e| e.index == *index) {
                    values.insert(f.language.clone(), serde_json::Value::String(entry.value.clone()));
                }
            }

            serde_json::json!({
                "ev_name": ev_name,
                "index": index,
                "type": field_type,
                "values": values,
            })
        })
        .collect();

    Ok(serde_json::Value::Array(entries))
}

async fn handle_get_entry(
    state: &McpState,
    params: Option<serde_json::Value>,
) -> Result<serde_json::Value, JsonRpcError> {
    let params = params.ok_or_else(|| JsonRpcError {
        code: -32602,
        message: "Missing params".to_string(),
    })?;

    let ev_name = params["ev_name"].as_str().ok_or_else(|| JsonRpcError {
        code: -32602,
        message: "Missing ev_name".to_string(),
    })?;
    let index = params["index"].as_u64().ok_or_else(|| JsonRpcError {
        code: -32602,
        message: "Missing index".to_string(),
    })? as usize;
    let lang = params["lang"].as_str().ok_or_else(|| JsonRpcError {
        code: -32602,
        message: "Missing lang".to_string(),
    })?;

    let session = state.session.read().await;

    let file = session
        .files
        .iter()
        .find(|f| f.ev_name == ev_name && f.language == lang)
        .ok_or_else(|| JsonRpcError {
            code: -32602,
            message: format!("No file found for ev_name={} lang={}", ev_name, lang),
        })?;

    let entry = file
        .entries
        .iter()
        .find(|e| e.index == index)
        .ok_or_else(|| JsonRpcError {
            code: -32602,
            message: format!("No entry at index {}", index),
        })?;

    Ok(serde_json::json!({
        "value": entry.value,
        "type": entry.field_type,
    }))
}

async fn handle_set_entry(
    state: &McpState,
    params: Option<serde_json::Value>,
) -> Result<serde_json::Value, JsonRpcError> {
    let params = params.ok_or_else(|| JsonRpcError {
        code: -32602,
        message: "Missing params".to_string(),
    })?;

    let ev_name = params["ev_name"].as_str().ok_or_else(|| JsonRpcError {
        code: -32602,
        message: "Missing ev_name".to_string(),
    })?.to_string();
    let index = params["index"].as_u64().ok_or_else(|| JsonRpcError {
        code: -32602,
        message: "Missing index".to_string(),
    })? as usize;
    let lang = params["lang"].as_str().ok_or_else(|| JsonRpcError {
        code: -32602,
        message: "Missing lang".to_string(),
    })?.to_string();
    let value = params["value"].as_str().ok_or_else(|| JsonRpcError {
        code: -32602,
        message: "Missing value".to_string(),
    })?.to_string();

    {
        let mut session = state.session.write().await;
        let file = session
            .files
            .iter_mut()
            .find(|f| f.ev_name == ev_name && f.language == lang)
            .ok_or_else(|| JsonRpcError {
                code: -32602,
                message: format!("No file found for ev_name={} lang={}", ev_name, lang),
            })?;

        let entry = file
            .entries
            .iter_mut()
            .find(|e| e.index == index)
            .ok_or_else(|| JsonRpcError {
                code: -32602,
                message: format!("No entry at index {}", index),
            })?;

        entry.value = value.clone();
    }

    let _ = state.app_handle.emit(
        "mcp_entry_changed",
        serde_json::json!({
            "ev_name": ev_name,
            "index": index,
            "lang": lang,
            "value": value,
        }),
    );

    Ok(serde_json::json!({ "success": true }))
}

async fn handle_apply_translations(
    state: &McpState,
    params: Option<serde_json::Value>,
) -> Result<serde_json::Value, JsonRpcError> {
    let params = params.ok_or_else(|| JsonRpcError {
        code: -32602,
        message: "Missing params".to_string(),
    })?;

    let entries = params["entries"].as_array().ok_or_else(|| JsonRpcError {
        code: -32602,
        message: "Missing entries array".to_string(),
    })?;

    let mut applied = 0usize;

    for entry_val in entries {
        let ev_name = match entry_val["ev_name"].as_str() {
            Some(s) => s.to_string(),
            None => continue,
        };
        let index = match entry_val["index"].as_u64() {
            Some(n) => n as usize,
            None => continue,
        };
        let lang = match entry_val["lang"].as_str() {
            Some(s) => s.to_string(),
            None => continue,
        };
        let value = match entry_val["value"].as_str() {
            Some(s) => s.to_string(),
            None => continue,
        };

        let updated = {
            let mut session = state.session.write().await;
            if let Some(file) = session
                .files
                .iter_mut()
                .find(|f| f.ev_name == ev_name && f.language == lang)
            {
                if let Some(e) = file.entries.iter_mut().find(|e| e.index == index) {
                    e.value = value.clone();
                    true
                } else {
                    false
                }
            } else {
                false
            }
        };

        if updated {
            let _ = state.app_handle.emit(
                "mcp_entry_changed",
                serde_json::json!({
                    "ev_name": ev_name,
                    "index": index,
                    "lang": lang,
                    "value": value,
                }),
            );
            applied += 1;
        }
    }

    Ok(serde_json::json!({ "applied": applied }))
}

async fn handle_get_session_info(
    state: &McpState,
    _params: Option<serde_json::Value>,
) -> Result<serde_json::Value, JsonRpcError> {
    let session = state.session.read().await;

    let mut languages: Vec<String> = session
        .files
        .iter()
        .map(|f| f.language.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    languages.sort();

    let mut seen_ev_names = std::collections::HashSet::new();
    let files: Vec<serde_json::Value> = session
        .files
        .iter()
        .filter(|f| seen_ev_names.insert(f.ev_name.clone()))
        .map(|f| {
            serde_json::json!({
                "ev_name": f.ev_name,
                "mode": format!("{:?}", f.mode).to_lowercase(),
            })
        })
        .collect();

    Ok(serde_json::json!({
        "languages": languages,
        "files": files,
        "game_profile": "level5",
    }))
}
