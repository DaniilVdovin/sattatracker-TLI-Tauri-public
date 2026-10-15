use lazy_static::lazy_static;
use serde_json::Value;
use std::collections::HashMap;

lazy_static! {
    static ref ITEM_DB: HashMap<String, String> = {
        let json_str = include_str!("../data/en_id_table.json");
        let parsed: Value = serde_json::from_str(json_str).unwrap_or(Value::Null);

        let mut map = HashMap::new();
        if let Some(obj) = parsed.as_object() {
            for (id, data) in obj {
                if let Some(name) = data.get("name").and_then(|n| n.as_str()) {
                    map.insert(id.clone(), name.to_string());
                }
            }
        }
        map
    };
}

pub struct DbManager;

impl DbManager {
    pub fn get_item_name(id: &str) -> String {
        ITEM_DB
            .get(id)
            .cloned()
            .unwrap_or_else(|| format!("Unknown Item ({})", id))
    }
}
