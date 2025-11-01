use crate::core::dto::error_dto::ErrorDTO;
use axum::http::StatusCode;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

// Serializes StatusCode as u16 in DTOs
pub fn serialize_status_code<S>(status: &StatusCode, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_u16(status.as_u16())
}

/// Deserializes a JSON Value to a DTO and extracts the field names that were provided
pub fn deserialize_with_fields<T>(body: Value) -> Result<(T, Vec<String>), ErrorDTO>
where
    T: DeserializeOwned,
{
    // Extract field names from the JSON object
    let fields = match &body {
        Value::Object(obj) => obj.keys().cloned().collect::<Vec<String>>(),
        _ => {
            return Err(ErrorDTO::new(
                StatusCode::BAD_REQUEST,
                "Request body must be a JSON object".to_string(),
            ));
        }
    };

    // Deserialize the JSON to the target DTO type
    let dto = serde_json::from_value(body).map_err(|e| {
        ErrorDTO::new(
            StatusCode::BAD_REQUEST,
            format!("Invalid request body: {}", e),
        )
    })?;

    Ok((dto, fields))
}

pub trait ToJson: serde::Serialize {
    fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or_else(|_| {
            json!({
                "error": "Failed to serialize error"
            })
        })
    }

    fn to_json_string(&self) -> String {
        serde_json::to_string(self)
            .unwrap_or_else(|_| r#"{"error":"Failed to serialize error"}"#.to_string())
    }
}
