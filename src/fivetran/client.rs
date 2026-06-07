use reqwest::{Client, header};
use super::models::{ConnectorStatus, ForceSyncResponse};
use anyhow::{Result, Context};
use crate::config::env;
use base64::Engine;

/// Función 1: Construye el cliente HTTP configurado para Fivetran
pub fn build_client() -> Result<Client> {
    let api_key = env::get_var_or("FIVETRAN_API_KEY", "");
    let api_secret = env::get_var_or("FIVETRAN_API_SECRET", "");
    
    // Basic Auth para Fivetran
    let auth = format!("{}:{}", api_key, api_secret);
    let b64_auth = base64::engine::general_purpose::STANDARD.encode(auth);

    let mut headers = header::HeaderMap::new();
    headers.insert(
        header::AUTHORIZATION,
        header::HeaderValue::from_str(&format!("Basic {}", b64_auth))?,
    );
    headers.insert(
        header::ACCEPT,
        header::HeaderValue::from_static("application/json; version=2"),
    );

    let client = Client::builder()
        .default_headers(headers)
        .build()
        .context("No se pudo construir el cliente HTTP reqwest")?;
        
    Ok(client)
}

/// Función 2: Obtiene el estado del conector
pub async fn get_connector_status(client: &Client, connector_id: &str) -> Result<ConnectorStatus> {
    let url = format!("https://api.fivetran.com/v1/connectors/{}", connector_id);
    let res = client.get(&url).send().await?.error_for_status()?;
    let status = res.json::<ConnectorStatus>().await?;
    Ok(status)
}

/// Función 3: Fuerza la sincronización
pub async fn force_sync(client: &Client, connector_id: &str) -> Result<ForceSyncResponse> {
    let url = format!("https://api.fivetran.com/v1/connectors/{}/force", connector_id);
    let res = client.post(&url).send().await?.error_for_status()?;
    let data = res.json::<ForceSyncResponse>().await?;
    Ok(data)
}
