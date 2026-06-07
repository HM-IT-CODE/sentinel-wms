use crate::mcp::jsonrpc::{
    McpRequest, McpResponse, McpError, InitializeResult, ServerCapabilities, ServerInfo,
    ToolsListResult, ToolCallResult, McpContent,
    METHOD_NOT_FOUND, INVALID_PARAMS, INTERNAL_ERROR
};
use crate::tools::inventory::{dispatch_mcp_tool, get_mcp_tools_definition};
use crate::db::connection::DbPool;
use serde_json::json;

/// Procesa una solicitud MCP JSON-RPC 2.0 y genera la respuesta correspondiente.
pub async fn handle_mcp_request(pool: &DbPool, req: McpRequest) -> McpResponse {
    let method = req.method.as_str();
    let id = req.id.clone();

    let result = match method {
        "initialize" => {
            let res = InitializeResult {
                protocol_version: "2024-11-05".to_string(),
                capabilities: ServerCapabilities {
                    tools: Some(json!({})),
                    resources: None,
                    prompts: None,
                },
                server_info: ServerInfo {
                    name: "mcp-sql-sentinel".to_string(),
                    version: "0.1.0".to_string(),
                },
            };
            serde_json::to_value(res)
                .map_err(|e| McpError::new(INTERNAL_ERROR, &format!("Error serializando initialize: {}", e)))
        }
        "tools/list" => {
            let mut tools = get_mcp_tools_definition();
            let mut fivetran_tools = crate::tools::fivetran::get_fivetran_tools_definition();
            tools.append(&mut fivetran_tools);
            let res = ToolsListResult { tools };
            serde_json::to_value(res)
                .map_err(|e| McpError::new(INTERNAL_ERROR, &format!("Error serializando tools/list: {}", e)))
        }
        "tools/call" => {
            let name = req.params.as_ref()
                .and_then(|p| p.get("name"))
                .and_then(|n| n.as_str())
                .unwrap_or_default();
            
            let arguments = req.params.as_ref()
                .and_then(|p| p.get("arguments"))
                .cloned()
                .unwrap_or(json!({}));
            
            if name.is_empty() {
                Err(McpError::new(INVALID_PARAMS, "El nombre de la herramienta es requerido"))
            } else {
                let execution_result = if name.starts_with("Fivetran") {
                    crate::tools::fivetran::dispatch_fivetran_tool(name, arguments).await
                } else {
                    dispatch_mcp_tool(pool, name, arguments).await
                };

                match execution_result {
                    Ok(val) => {
                        let content = vec![McpContent::Text { text: val.to_string() }];
                        let res = ToolCallResult {
                            content,
                            is_error: Some(false),
                        };
                        serde_json::to_value(res)
                            .map_err(|e| McpError::new(INTERNAL_ERROR, &format!("Error serializando respuesta: {}", e)))
                    }
                    Err(e) => {
                        // Los errores de ejecución interna de las tools los devolvemos dentro del esquema MCP como exitosos pero con is_error = true
                        let content = vec![McpContent::Text { text: e }];
                        let res = ToolCallResult {
                            content,
                            is_error: Some(true),
                        };
                        serde_json::to_value(res)
                            .map_err(|e| McpError::new(INTERNAL_ERROR, &format!("Error serializando error de ejecución: {}", e)))
                    }
                }
            }
        }
        _ => {
            Err(McpError::new(METHOD_NOT_FOUND, &format!("Método no soportado: {}", method)))
        }
    };

    match result {
        Ok(res_val) => McpResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(res_val),
            error: None,
        },
        Err(mcp_err) => McpResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(mcp_err),
        },
    }
}
