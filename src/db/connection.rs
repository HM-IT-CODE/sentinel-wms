use bb8::Pool;
use bb8_tiberius::ConnectionManager;
use tiberius::{Config, AuthMethod};
use std::env;

// Alias para el Pool de conexiones
pub type DbPool = Pool<ConnectionManager>;

pub async fn init_pool() -> DbPool {
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL no configurada");
    
    // Parseo manual de la cadena de conexión
    // Formato esperado: sqlserver://user:password@host:port/database?trust_server_certificate=true
    let config = parse_connection_string(&db_url);
    
    let mgr = ConnectionManager::new(config);
    
    Pool::builder()
        .max_size(10) // Mismo default que sqlx
        .build(mgr)
        .await
        .expect("Error creando el pool de conexiones a SQL Server")
}

fn parse_connection_string(url_str: &str) -> Config {
    let url = url::Url::parse(url_str).expect("URL de base de datos inválida");
    
    let mut config = Config::new();
    
    // Host y Puerto
    if let Some(host) = url.host_str() {
        config.host(host);
    }
    if let Some(port) = url.port() {
        config.port(port);
    } else {
        config.port(1433); // Default SQL Server port
    }
    
    // Usuario y Password
    let user = url.username();
    let pass = url.password().unwrap_or("");
    
    if !user.is_empty() {
        config.authentication(AuthMethod::sql_server(user, pass));
    }
    
    // Base de datos (path viene como "/DBName")
    let db_name = url.path().trim_start_matches('/');
    if !db_name.is_empty() {
        config.database(db_name);
    }

    // Trust Server Certificate (Crítico para certificados auto-firmados)
    // Buscamos el parámetro en query string
    for (k, v) in url.query_pairs() {
        if k == "trust_server_certificate" && v == "true" {
            config.trust_cert();
        }
    }
    
    config
}
