use super::connection::DbPool;
use super::models::{DbObject, DbParam};
use std::collections::{HashMap, HashSet};
use std::fs;
use serde::Deserialize;

#[derive(Deserialize)]
struct AppConfig {
    allowed_objects: Option<Vec<String>>,
}

pub async fn get_database_schema(pool: &DbPool) -> Result<Vec<DbObject>, Box<dyn std::error::Error>> {
    let mut client = pool.get().await?;

    // 1. CARGAR CONFIGURACIÓN
    let allowed_list: Option<HashSet<String>> = match fs::read_to_string("config.json") {
        Ok(content) => {
            match serde_json::from_str::<AppConfig>(&content) {
                Ok(cfg) => cfg.allowed_objects.map(|list| list.into_iter().collect()),
                Err(_) => None,
            }
        },
        Err(_) => None,
    };
    
    // 2. QUERY DE OBJETOS (Tiberius: Directo y limpio, sin hacks)
    // Tiberius maneja UTF-8/Locales nativamente.
    let query_str = "
        SELECT 
            name,
            CAST(CASE type 
                WHEN 'U' THEN 'TABLE'
                WHEN 'V' THEN 'VIEW'
                WHEN 'P' THEN 'PROCEDURE'
            END AS NVARCHAR(50)) as kind,
            SCHEMA_NAME(schema_id) as [schema]
        FROM sys.objects
        WHERE type IN ('U', 'V', 'P')
            AND is_ms_shipped = 0
            AND name NOT LIKE 'spt_%'
            AND SCHEMA_NAME(schema_id) NOT IN ('ssma_oracle', 'sys', 'guest', 'INFORMATION_SCHEMA')
    ";

    let rows = client.query(query_str, &[]).await?.into_first_result().await?;
    
    let mut objects_map: HashMap<String, DbObject> = HashMap::new();
    let mut ordered_names: Vec<String> = Vec::new();

    for row in rows {
        // En Tiberius obtenemos por nombre de columna o índice.
        // name (0), kind (1), schema (2)
        let name: &str = row.get("name").unwrap_or_default();
        let name_string = name.to_string();

        if let Some(allow) = &allowed_list {
            if !allow.contains(&name_string) {
                continue;
            }
        }

        let kind: &str = row.get("kind").unwrap_or("UNKNOWN");
        let schema: &str = row.get("schema").unwrap_or("dbo");

        let obj = DbObject {
            name: name_string.clone(),
            kind: kind.to_string(),
            schema: schema.to_string(),
            params: Some(Vec::new()),
        };

        objects_map.insert(name_string.clone(), obj);
        ordered_names.push(name_string);
    }

    // 3. ENRIQUECIMIENTO OPTIMIZADO (Filtro a nivel de SQL Server para bases de datos ERP grandes)
    let query_struct = if let Some(ref list) = allowed_list {
        if list.is_empty() {
            "SELECT CAST('' AS NVARCHAR(128)) as object_name, CAST('' AS NVARCHAR(128)) as field_name, CAST('' AS NVARCHAR(128)) as data_type, 0 as position WHERE 1=0".to_string()
        } else {
            let escaped_names: Vec<String> = list.iter()
                .map(|name| format!("'{}'", name.replace("'", "''")))
                .collect();
            let names_in_clause = escaped_names.join(",");

            format!("
                SELECT 
                    o.name AS object_name,
                    c.name AS field_name,
                    t.name AS data_type,
                    c.column_id AS position
                FROM sys.objects o
                JOIN sys.columns c ON o.object_id = c.object_id
                JOIN sys.types t ON c.user_type_id = t.user_type_id
                WHERE o.type IN ('U', 'V')
                  AND o.name IN ({names})
                
                UNION ALL

                SELECT 
                    p.name AS object_name,
                    par.name AS field_name,
                    t.name AS data_type,
                    par.parameter_id AS position
                FROM sys.procedures p
                JOIN sys.parameters par ON p.object_id = par.object_id
                JOIN sys.types t ON par.user_type_id = t.user_type_id
                WHERE p.name IN ({names})
            ", names = names_in_clause)
        }
    } else {
        "
            SELECT 
                o.name AS object_name,
                c.name AS field_name,
                t.name AS data_type,
                c.column_id AS position
            FROM sys.objects o
            JOIN sys.columns c ON o.object_id = c.object_id
            JOIN sys.types t ON c.user_type_id = t.user_type_id
            WHERE o.type IN ('U', 'V')
            
            UNION ALL

            SELECT 
                p.name AS object_name,
                par.name AS field_name,
                t.name AS data_type,
                par.parameter_id AS position
            FROM sys.procedures p
            JOIN sys.parameters par ON p.object_id = par.object_id
            JOIN sys.types t ON par.user_type_id = t.user_type_id
        ".to_string()
    };

    let struct_rows = client.query(query_struct.as_str(), &[]).await?.into_first_result().await?;

    for row in struct_rows {
        let obj_name: &str = row.get("object_name").unwrap_or_default();
        
        if let Some(obj) = objects_map.get_mut(obj_name) {
            let field_name: &str = row.get("field_name").unwrap_or_default();
            let data_type: &str = row.get("data_type").unwrap_or_default();

            if let Some(params) = &mut obj.params {
                params.push(DbParam {
                    name: field_name.to_string(),
                    data_type: data_type.to_string(),
                });
            }
        }
    }

    let result = ordered_names.into_iter()
        .filter_map(|name| objects_map.remove(&name))
        .collect();

    Ok(result)
}