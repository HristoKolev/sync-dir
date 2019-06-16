use serde::{Serialize, Deserialize};

use super::prelude::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppConfig {
    pub sentry_dsn: String,
}

pub fn read_config(file_path: &str) -> Result<AppConfig> {
    let json_content = ::std::fs::read_to_string(file_path)?;
    let materialized = serde_json::from_str(&json_content)?;
    Ok(materialized)
}
