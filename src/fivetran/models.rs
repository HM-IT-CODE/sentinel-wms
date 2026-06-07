use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConnectorStatus {
    pub code: String,
    pub message: Option<String>,
    pub data: Option<ConnectorData>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConnectorData {
    pub id: String,
    #[serde(default)]
    pub service: Option<String>,
    #[serde(default)]
    pub schema: Option<String>,
    #[serde(default)]
    pub paused: Option<bool>,
    pub status: ConnectorState,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConnectorState {
    pub setup_state: String,
    pub sync_state: String,
    #[serde(default)]
    pub update_state: Option<String>,
    #[serde(default)]
    pub is_historical_sync: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ForceSyncResponse {
    pub code: String,
    pub message: String,
    pub data: Option<Value>,
}
