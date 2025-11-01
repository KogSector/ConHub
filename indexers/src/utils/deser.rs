use anyhow::Result;
use serde::de::DeserializeOwned;

/// Deserialize from JSON string
pub fn from_json_str<T: DeserializeOwned>(s: &str) -> Result<T> {
    serde_json::from_str(s).map_err(Into::into)
}

/// Deserialize from JSON value
pub fn from_json_value<T: DeserializeOwned>(value: serde_json::Value) -> Result<T> {
    serde_json::from_value(value).map_err(Into::into)
}