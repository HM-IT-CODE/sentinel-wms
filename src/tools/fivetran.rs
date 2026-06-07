use serde_json::{json, Value};
use crate::mcp::jsonrpc::McpToolDefinition;

/// Herramientas relacionadas a la orquestación de datos con Fivetran
pub fn get_fivetran_tools_definition() -> Vec<McpToolDefinition> {
    vec![
        McpToolDefinition {
            name: "FivetranConnectorStatus".to_string(),
            description: "Obtiene el estado actual de un conector en Fivetran.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "connector_id": {
                        "type": "string",
                        "description": "El ID único del conector de Fivetran."
                    }
                },
                "required": ["connector_id"]
            }),
        },
        McpToolDefinition {
            name: "FivetranForceSync".to_string(),
            description: "Fuerza una sincronización inmediata de un conector en Fivetran.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "connector_id": {
                        "type": "string",
                        "description": "El ID único del conector de Fivetran."
                    }
                },
                "required": ["connector_id"]
            }),
        },
    ]
}

pub async fn dispatch_fivetran_tool(name: &str, arguments: Value) -> Result<Value, String> {
    // 1. Construir cliente HTTP
    let client = crate::fivetran::client::build_client()
        .map_err(|e| format!("Error construyendo cliente Fivetran: {}", e))?;

    match name {
        "FivetranConnectorStatus" => {
            let connector_id = arguments.get("connector_id")
                .and_then(|v| v.as_str())
                .ok_or("Falta el argumento requerido 'connector_id'")?;

            let status = crate::fivetran::client::get_connector_status(&client, connector_id)
                .await
                .map_err(|e| format!("Error consultando estado Fivetran: {}", e))?;

            Ok(json!(status))
        },
        "FivetranForceSync" => {
            let connector_id = arguments.get("connector_id")
                .and_then(|v| v.as_str())
                .ok_or("Falta el argumento requerido 'connector_id'")?;

            let sync_res = crate::fivetran::client::force_sync(&client, connector_id)
                .await
                .map_err(|e| format!("Error forzando sync Fivetran: {}", e))?;

            Ok(json!(sync_res))
        },
        _ => Err(format!("Herramienta Fivetran no implementada: {}", name)),
    }
}
