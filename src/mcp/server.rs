use axum::{
    extract::{Query, State},
    response::sse::{Event, Sse},
    routing::{get, post},
    Json, Router,
};
use futures::stream::{self, Stream};
use std::{collections::HashMap, convert::Infallible, sync::Arc};
use tokio::sync::{mpsc, RwLock};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};

use crate::mcp::jsonrpc::{McpMessage, McpResponse, McpError, PARSE_ERROR, INVALID_REQUEST, INTERNAL_ERROR};
use crate::mcp::router::handle_mcp_request;
use crate::db::connection::DbPool;

// ==========================================
// STDIO SERVER
// ==========================================

/// Inicia el bucle de procesamiento MCP sobre Stdio (stdin / stdout).
/// Todo log y depuración se escribe en stderr para no contaminar el canal de comunicación.
pub async fn start_stdio_transport(pool: DbPool) -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("⚙️ Iniciando transporte Stdio para servidor MCP...");
    
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut reader = BufReader::new(stdin).lines();

    while let Some(line) = reader.next_line().await? {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        eprintln!("📥 Mensaje Recibido (Stdio): {}", trimmed);

        let response = match serde_json::from_str::<McpMessage>(trimmed) {
            Ok(msg) => match msg {
                McpMessage::Request(req) => {
                    let res = handle_mcp_request(&pool, req).await;
                    Some(res)
                }
                McpMessage::Notification(notif) => {
                    eprintln!("🔔 Notificación recibida: {}", notif.method);
                    None
                }
                McpMessage::Response(res) => {
                    eprintln!("📤 Respuesta del cliente recibida para ID: {:?}", res.id);
                    None
                }
            },
            Err(e) => {
                eprintln!("⚠️ Error parseando JSON: {}", e);
                let err_res = McpResponse {
                    jsonrpc: "2.0".to_string(),
                    id: serde_json::Value::Null,
                    result: None,
                    error: Some(McpError::new(PARSE_ERROR, &format!("Error de Parseo JSON: {}", e))),
                };
                Some(err_res)
            }
        };

        if let Some(res) = response {
            match serde_json::to_string(&res) {
                Ok(res_str) => {
                    eprintln!("📤 Enviando Respuesta: {}", res_str);
                    stdout.write_all(res_str.as_bytes()).await?;
                    stdout.write_all(b"\n").await?;
                    stdout.flush().await?;
                }
                Err(e) => {
                    eprintln!("🚨 Error serializando respuesta: {}", e);
                }
            }
        }
    }

    eprintln!("🛑 Fin del flujo Stdio.");
    Ok(())
}

// ==========================================
// SSE SERVER
// ==========================================

/// Generador de IDs de sesión thread-safe
static SESSION_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Estado global para el servidor SSE
#[derive(Clone)]
pub struct SseServerState {
    pub db_pool: DbPool,
    pub sessions: Arc<RwLock<HashMap<String, mpsc::UnboundedSender<Event>>>>,
}

#[derive(Deserialize)]
pub struct MessageParams {
    pub session_id: String,
}

/// Construye las rutas de Axum para el transporte SSE
pub fn create_sse_router(state: SseServerState) -> Router {
    Router::new()
        .route("/sse", get(sse_handler))
        .route("/message", post(message_handler))
        .with_state(state)
}

/// Manejador de la conexión SSE (GET /sse)
async fn sse_handler(
    State(state): State<SseServerState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let session_id = format!("session-{}", SESSION_ID_COUNTER.fetch_add(1, Ordering::SeqCst));
    eprintln!("🔌 Nueva conexión SSE iniciada. Asignando session_id: {}", session_id);

    let (tx, rx) = mpsc::unbounded_channel::<Event>();

    {
        let mut sessions = state.sessions.write().await;
        sessions.insert(session_id.clone(), tx.clone());
    }

    let endpoint_url = format!("/message?session_id={}", session_id);
    let init_event = Event::default()
        .event("endpoint")
        .data(endpoint_url);
    
    if let Err(e) = tx.send(init_event) {
        eprintln!("🚨 Error enviando evento inicial 'endpoint': {}", e);
    }

    let clean_session_id = session_id.clone();
    let clean_state = state.clone();
    let stream = stream::unfold(rx, move |mut rx| {
        let clean_session_id = clean_session_id.clone();
        let clean_state = clean_state.clone();
        async move {
            match rx.recv().await {
                Some(event) => Some((Ok(event), rx)),
                None => {
                    eprintln!("🔌 Conexión SSE cerrada para session_id: {}", clean_session_id);
                    let mut sessions = clean_state.sessions.write().await;
                    sessions.remove(&clean_session_id);
                    None
                }
            }
        }
    });

    Sse::new(stream)
}

/// Manejador de mensajes entrantes (POST /message?session_id=XXX)
async fn message_handler(
    State(state): State<SseServerState>,
    Query(params): Query<MessageParams>,
    Json(payload): Json<Value>,
) -> Json<Value> {
    let session_id = params.session_id;
    eprintln!("📥 Mensaje Recibido (SSE - {}): {:?}", session_id, payload);

    let tx = {
        let sessions = state.sessions.read().await;
        sessions.get(&session_id).cloned()
    };

    let tx = match tx {
        Some(sender) => sender,
        None => {
            return Json(json!({
                "jsonrpc": "2.0",
                "id": Value::Null,
                "error": McpError::new(INVALID_REQUEST, &format!("Sesión activa no encontrada: {}", session_id))
            }));
        }
    };

    let response = match serde_json::from_value::<McpMessage>(payload) {
        Ok(msg) => match msg {
            McpMessage::Request(req) => {
                let res = handle_mcp_request(&state.db_pool, req).await;
                Some(res)
            }
            McpMessage::Notification(notif) => {
                eprintln!("🔔 Notificación recibida vía SSE: {}", notif.method);
                None
            }
            McpMessage::Response(res) => {
                eprintln!("📤 Respuesta del cliente recibida vía SSE para ID: {:?}", res.id);
                None
            }
        },
        Err(e) => {
            eprintln!("⚠️ Error parseando JSON en SSE: {}", e);
            let err_res = McpResponse {
                jsonrpc: "2.0".to_string(),
                id: Value::Null,
                result: None,
                error: Some(McpError::new(INVALID_REQUEST, &format!("Error de Parseo JSON: {}", e))),
            };
            Some(err_res)
        }
    };

    if let Some(res) = response {
        match serde_json::to_string(&res) {
            Ok(res_str) => {
                let event = Event::default()
                    .event("message")
                    .data(res_str);
                if let Err(e) = tx.send(event) {
                    eprintln!("🚨 Error enviando evento a la sesión SSE: {}", e);
                    return Json(json!({
                        "jsonrpc": "2.0",
                        "id": res.id,
                        "error": McpError::new(INTERNAL_ERROR, "Canal de sesión SSE cerrado de forma inesperada")
                    }));
                }
            }
            Err(e) => {
                eprintln!("🚨 Error serializando respuesta SSE: {}", e);
            }
        }
    }

    Json(json!({ "status": "accepted" }))
}
