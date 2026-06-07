use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Representa cualquier mensaje válido del protocolo MCP bajo JSON-RPC 2.0.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum McpMessage {
    Request(McpRequest),
    Response(McpResponse),
    Notification(McpNotification),
}

/// Una solicitud JSON-RPC 2.0 enviada por el cliente.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct McpRequest {
    pub jsonrpc: String,
    pub id: Value,
    pub method: String,
    #[serde(default)]
    pub params: Option<Value>,
}

/// Una respuesta JSON-RPC 2.0 enviada por el servidor.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct McpResponse {
    pub jsonrpc: String,
    pub id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<McpError>,
}

/// Una notificación JSON-RPC 2.0 (no requiere respuesta).
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct McpNotification {
    pub jsonrpc: String,
    pub method: String,
    #[serde(default)]
    pub params: Option<Value>,
}

/// Estructura de error estándar en JSON-RPC 2.0.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct McpError {
    pub code: i64,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl McpError {
    pub fn new(code: i64, message: &str) -> Self {
        Self {
            code,
            message: message.to_string(),
            data: None,
        }
    }
}

// Codigos de Error estándar JSON-RPC
pub const PARSE_ERROR: i64 = -32700;
pub const INVALID_REQUEST: i64 = -32600;
pub const METHOD_NOT_FOUND: i64 = -32601;
pub const INVALID_PARAMS: i64 = -32602;
pub const INTERNAL_ERROR: i64 = -32603;

// Estructuras específicas para el handshake 'initialize'
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InitializeResult {
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    pub capabilities: ServerCapabilities,
    #[serde(rename = "serverInfo")]
    pub server_info: ServerInfo,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompts: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
}

// Estructuras de respuesta para tools/list
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ToolsListResult {
    pub tools: Vec<McpToolDefinition>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct McpToolDefinition {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

// Estructura de respuesta para tools/call
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ToolCallResult {
    pub content: Vec<McpContent>,
    #[serde(rename = "isError", skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum McpContent {
    #[serde(rename = "text")]
    Text { text: String },
}
