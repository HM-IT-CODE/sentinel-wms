mod config;
mod mcp;
mod tools;
mod db;
mod fivetran;

use std::env;
use std::net::SocketAddr;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use db::connection::init_pool;
use mcp::server::{start_stdio_transport, create_sse_router, SseServerState};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Carga centralizada de configuración
    config::env::load_env();

    // 2. Determinar tipo de transporte mediante argumentos de línea de comando.
    let args: Vec<String> = env::args().collect();

    // Modo de prueba rápida de la integración con Fivetran (no levanta el servidor).
    // Uso: cargo run -- --test-fivetran
    if args.iter().any(|a| a == "--test-fivetran") {
        let connector_id = env::var("FIVETRAN_CONNECTOR_ID").unwrap_or_default();
        eprintln!("🔌 Probando FivetranConnectorStatus para connector_id = '{}'", connector_id);
        let client = fivetran::client::build_client().map_err(|e| e.to_string())?;
        let status = fivetran::client::get_connector_status(&client, &connector_id)
            .await
            .map_err(|e| e.to_string())?;
        println!("{}", serde_json::to_string_pretty(&status).map_err(|e| e.to_string())?);
        eprintln!("✅ Integración Rust ↔ Fivetran OK");
        return Ok(());
    }

    let mut transport = "stdio";

    for i in 0..args.len() {
        if (args[i] == "--transport" || args[i] == "-t") && i + 1 < args.len() {
            transport = &args[i + 1];
        }
    }

    if transport == "stdio" {
        // En modo STDIO, todo log debe enviarse a stderr
        eprintln!("🚀 Iniciando servidor MCP SQL Sentinel en modo STDIO...");
        
        let pool = init_pool().await;
        // Validar conexion activa al arrancar
        match pool.get().await {
            Ok(_) => eprintln!("✅ Conexion validada y activa con SQL Server."),
            Err(e) => {
                eprintln!("🚨 Error critico conectando a la base de datos: {}", e);
                std::process::exit(1);
            }
        }
        
        start_stdio_transport(pool).await?;
    } else if transport == "sse" {
        println!("🚀 Iniciando servidor MCP SQL Sentinel en modo SSE (Servidor Web)...");
        
        let pool = init_pool().await;
        match pool.get().await {
            Ok(_) => println!("✅ Conexion validada y activa con SQL Server."),
            Err(e) => {
                println!("🚨 Error critico conectando a la base de datos: {}", e);
                std::process::exit(1);
            }
        }

        let state = SseServerState {
            db_pool: pool,
            sessions: Arc::new(RwLock::new(HashMap::new())),
        };

        let app = create_sse_router(state);

        let port_str = config::env::get_var_or("PORT", "3000");
        let port: u16 = port_str.parse().unwrap_or(3000);
        let addr = SocketAddr::from(([0, 0, 0, 0], port));

        println!("👂 Servidor MCP SSE escuchando en: http://localhost:{}", port);
        
        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await?;
    } else {
        eprintln!("🚨 Transporte no soportado: '{}'. Usa 'stdio' o 'sse'.", transport);
        std::process::exit(1);
    }

    Ok(())
}