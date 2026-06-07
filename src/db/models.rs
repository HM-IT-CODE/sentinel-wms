use serde::{Deserialize, Serialize};

// Asegúrate de que TODOS tengan 'pub' al inicio

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DbParam {
    pub name: String,       // <--- Tienen que decir PUB
    pub data_type: String,  // <--- Tienen que decir PUB
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DbObject {
    pub name: String,       // <--- IMPORTANTE: pub
    pub kind: String,       // <--- IMPORTANTE: pub
    pub schema: String,     // <--- IMPORTANTE: pub
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Vec<DbParam>>, // <--- IMPORTANTE: pub
}