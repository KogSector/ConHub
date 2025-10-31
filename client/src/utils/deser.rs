use serde::de::DeserializeOwned;
use serde_json::Value;

pub fn from_json_value<T: DeserializeOwned>(value: Value) -> Result<T, serde_json::Error> {
    serde_json::from_value(value)
}