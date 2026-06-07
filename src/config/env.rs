use std::env;
use dotenvy::dotenv;

/// Carga las variables de entorno, preferiblemente desde el archivo .env local
pub fn load_env() {
    if let Ok(mut exe_path) = env::current_exe() {
        // De target/debug/mcp-sql-sentinel.exe subimos 3 niveles a la raiz del proyecto
        exe_path.pop(); // Remueve el nombre del ejecutable
        exe_path.pop(); // Remueve 'debug'
        exe_path.pop(); // Remueve 'target'
        let env_path = exe_path.join(".env");
        if env_path.exists() {
            dotenvy::from_path(&env_path).ok();
        } else {
            dotenv().ok();
        }
    } else {
        dotenv().ok();
    }
}

/// Obtiene una variable de entorno como String, con un valor por defecto si no existe
pub fn get_var_or(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}
