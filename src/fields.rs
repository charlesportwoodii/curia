use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Fields(Map<String, Value>);

impl Fields {
    pub fn new() -> Self {
        Self(Map::new())
    }

    pub fn from_map(map: Map<String, Value>) -> Self {
        Self(map)
    }

    // A value that is not a JSON object still has to survive, so it is nested
    // under `value` rather than discarded.
    pub fn from_serializable(value: impl Serialize) -> Self {
        match serde_json::to_value(value) {
            Ok(Value::Object(map)) => Self(map),
            Ok(other) => {
                let mut map = Map::new();
                map.insert("value".to_string(), other);
                Self(map)
            }
            Err(e) => {
                let mut map = Map::new();
                map.insert(
                    "value".to_string(),
                    Value::String(format!("<unserializable: {e}>")),
                );
                Self(map)
            }
        }
    }

    // A logging call must never fail, so a value that will not serialize becomes
    // a visible marker rather than a dropped key or a propagated error.
    pub fn insert(&mut self, key: impl Into<String>, value: impl Serialize) -> &mut Self {
        let value = serde_json::to_value(value)
            .unwrap_or_else(|e| Value::String(format!("<unserializable: {e}>")));
        self.0.insert(key.into(), value);
        self
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.0.get(key)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn as_map(&self) -> &Map<String, Value> {
        &self.0
    }

    // Used to strip a bridge's own bookkeeping keys before an event is handed on.
    pub fn remove_prefixed(&mut self, prefix: &str) {
        self.0.retain(|key, _| !key.starts_with(prefix));
    }
}
