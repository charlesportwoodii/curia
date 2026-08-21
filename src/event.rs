use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{Fields, Level};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEvent {
    pub level: Level,
    pub target: String,
    pub message: String,
    pub fields: Fields,
    pub timestamp: DateTime<Utc>,
    pub file: Option<String>,
    pub line: Option<u32>,
}
