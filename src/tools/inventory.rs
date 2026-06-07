use serde_json::{json, Value};
use serde::Deserialize;
use crate::db::connection::DbPool;
use tiberius::{Client, Config, AuthMethod};
use tokio::net::TcpStream;
use tokio_util::compat::TokioAsyncWriteCompatExt;
use futures::io::{AsyncRead, AsyncWrite};
use crate::mcp::jsonrpc::McpToolDefinition;

#[derive(Deserialize, Debug, Clone)]
pub struct TargetDbConfig {
    pub host: String,
    pub db_name: String,
    #[serde(default)]
    pub port: Option<u16>,
    #[serde(default)]
    pub user: Option<String>,
    #[serde(default)]
    pub pass: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct EstadoCuentaArgs {
    pub nombre_cliente: String,
}

#[derive(Deserialize, Debug)]
pub struct EjecutarSqlArgs {
    pub query: String,
    pub razonamiento: String,
    #[serde(default)]
    pub target_config: Option<TargetDbConfig>,
}

/// Retorna la lista de definiciones de herramientas compatibles con MCP
pub fn get_mcp_tools_definition() -> Vec<McpToolDefinition> {
    vec![
        McpToolDefinition {
            name: "EstadoCuentaCliente".to_string(),
            description: "Obtiene el estado de cuenta simple de un cliente buscando su identificador único por su nombre.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "nombre_cliente": {
                        "type": "string",
                        "description": "Nombre o razón social parcial del cliente."
                    }
                },
                "required": ["nombre_cliente"]
            }),
        },
        McpToolDefinition {
            name: "ObtenerEsquemaBaseDatos".to_string(),
            description: "Obtiene el esquema completo de la base de datos (tablas, vistas, procedimientos almacenados y sus columnas/tipos). Es crucial llamarla al inicio para auto-descubrir el mapa de la base de datos.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        McpToolDefinition {
            name: "EjecutarSQLDinamico".to_string(),
            description: "Ejecuta consultas SELECT de SQL Server de manera segura. Permite conectarse a bases de datos remotas dinámicamente.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "La consulta SQL SELECT a ejecutar en SQL Server."
                    },
                    "razonamiento": {
                        "type": "string",
                        "description": "El razonamiento del agente para ejecutar esta consulta específica."
                    },
                    "target_config": {
                        "type": "object",
                        "description": "Configuración dinámica del servidor SQL destino. Si no se provee, se usará la base de datos principal.",
                        "properties": {
                            "host": { "type": "string", "description": "Host o dirección IP del servidor." },
                            "db_name": { "type": "string", "description": "Nombre de la base de datos." },
                            "port": { "type": "integer", "description": "Puerto TCP (por defecto 1433)." },
                            "user": { "type": "string", "description": "Usuario de autenticación SQL Server." },
                            "pass": { "type": "string", "description": "Contraseña de autenticación SQL Server." }
                        },
                        "required": ["host", "db_name"]
                    }
                },
                "required": ["query", "razonamiento"]
            }),
        },
    ]
}

/// Conexión dinámica a bases de datos externas para Tiberius.
async fn get_dynamic_connection(config: &TargetDbConfig) -> Result<Client<tokio_util::compat::Compat<TcpStream>>, String> {
    let mut tiberius_config = Config::new();
    
    // Fallback de Credenciales desde DATABASE_URL si no se proveen
    let default_url = std::env::var("DATABASE_URL").unwrap_or_default();
    let url = url::Url::parse(&default_url).map_err(|_| "DATABASE_URL inválida para fallback")?;
    
    let user = config.user.as_deref().unwrap_or(url.username());
    let pass = config.pass.as_deref().unwrap_or(url.password().unwrap_or(""));
    let port = config.port.unwrap_or(1433);

    tiberius_config.host(&config.host);
    tiberius_config.port(port);
    tiberius_config.database(&config.db_name);
    tiberius_config.authentication(AuthMethod::sql_server(user, pass));
    tiberius_config.trust_cert();

    let tcp = TcpStream::connect(format!("{}:{}", config.host, port))
        .await
        .map_err(|e| format!("Error conectando TCP a {}:{}: {}", config.host, port, e))?;
    
    tcp.set_nodelay(true).ok();

    let client = Client::connect(tiberius_config, tcp.compat_write())
        .await
        .map_err(|e| format!("Error conectando Tiberius: {}", e))?;

    Ok(client)
}

/// Enrutador principal de herramientas MCP
pub async fn dispatch_mcp_tool(pool: &DbPool, name: &str, arguments: Value) -> Result<Value, String> {
    match name {
        "EstadoCuentaCliente" => {
            let args: EstadoCuentaArgs = serde_json::from_value(arguments)
                .map_err(|e| format!("Argumentos inválidos para EstadoCuentaCliente: {}", e))?;
            
            let mut client = match pool.get().await {
                Ok(c) => c,
                Err(e) => return Err(e.to_string())
            };

            let search_sql = format!("SELECT TOP 1 co_cli FROM CLIENTES WHERE cli_des LIKE '%{}%'", args.nombre_cliente);
            let rows = run_read_query_with_client(&mut *client, &search_sql).await?;
            let rows_array = rows.as_array().ok_or("Error procesando filas de clientes")?;
            
            if let Some(row) = rows_array.first() {
                let client_id = row.get("co_cli")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                
                let sp = format!("EXEC sp_edo_cuenta_simple @clientes = '{}'", client_id);
                run_read_query_with_client(&mut *client, &sp).await
            } else {
                Ok(json!({ "message": "Cliente no encontrado" }))
            }
        },

        "ObtenerEsquemaBaseDatos" => {
            match crate::db::introspection::get_database_schema(pool).await {
                Ok(objects) => Ok(json!(objects)),
                Err(e) => Err(format!("Error obteniendo esquema de base de datos: {}", e)),
            }
        },

        "EjecutarSQLDinamico" => {
            let args: EjecutarSqlArgs = serde_json::from_value(arguments)
                .map_err(|e| format!("Argumentos inválidos para EjecutarSQLDinamico: {}", e))?;
            
            eprintln!("🧠 IA Razonó: {}", args.razonamiento);
            
            if let Some(config) = args.target_config {
                eprintln!("🌐 Conectando a Target Dinámico: {} -> {}", config.host, config.db_name);
                eprintln!("🚀 Ejecutando SQL (Dynamic): {}", args.query);
                
                let mut client = get_dynamic_connection(&config).await?;
                run_read_query_with_client(&mut client, &args.query).await
            } else {
                eprintln!("🏠 Usando Pool de Base de Datos Principal");
                eprintln!("🚀 Ejecutando SQL (Pool): {}", args.query);
                run_read_query(pool, &args.query).await
            }
        },

        _ => Err(format!("Herramienta no implementada: {}", name)),
    }
}

/// Función genérica que acepta un cliente ya conectado
async fn run_read_query_with_client<S>(client: &mut Client<S>, query: &str) -> Result<Value, String> 
where S: AsyncRead + AsyncWrite + Unpin + Send 
{
    let rows = client.query(query, &[])
        .await
        .map_err(|e| format!("Error ejecutando SQL: {}", e))?
        .into_first_result()
        .await
        .map_err(|e| format!("Error obteniendo resultados: {}", e))?;

    let mut results = Vec::new();

    for row in rows {
        let mut map = serde_json::Map::new();

        for col in row.columns() {
            let name = col.name();
            
            // LECTURA EN CASCADA SEGURA (Evita Panics)
            let val = if let Ok(Some(s)) = row.try_get::<&str, _>(name) {
                json!(s)
            } 
            else if let Ok(Some(i)) = row.try_get::<i32, _>(name) {
                json!(i)
            }
            else if let Ok(Some(i)) = row.try_get::<i64, _>(name) {
                json!(i)
            }
            else if let Ok(Some(d)) = row.try_get::<rust_decimal::Decimal, _>(name) {
                json!(d.to_string())
            }
            else if let Ok(Some(f)) = row.try_get::<f64, _>(name) {
                json!(f)
            }
            else if let Ok(Some(b)) = row.try_get::<bool, _>(name) {
                json!(b)
            }
            else if let Ok(Some(dt)) = row.try_get::<chrono::NaiveDateTime, _>(name) {
                json!(dt.to_string())
            }
            else if let Ok(Some(d)) = row.try_get::<chrono::NaiveDate, _>(name) {
                json!(d.to_string())
            }
            else {
                json!(null)
            };
            
            map.insert(name.to_string(), val);
        }
        results.push(Value::Object(map));
    }

    Ok(json!(results))
}

/// Wrapper para usar el pool por defecto
async fn run_read_query(pool: &DbPool, query: &str) -> Result<Value, String> {
    let mut client = match pool.get().await {
        Ok(c) => c,
        Err(e) => return Err(format!("Error pool: {}", e))
    };
    run_read_query_with_client(&mut *client, query).await
}