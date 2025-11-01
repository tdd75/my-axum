use crate::core::dto::error_dto::ErrorDTO;
use axum::http::StatusCode;
use rust_i18n::t;
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
pub fn deserialize_with_fields<T>(body: Value, locale: &str) -> Result<(T, Vec<String>), ErrorDTO>
where
    T: DeserializeOwned,
{
    // Extract field names from the JSON object
    let fields = match &body {
        Value::Object(obj) => obj.keys().cloned().collect::<Vec<String>>(),
        _ => {
            return Err(ErrorDTO::new(
                StatusCode::BAD_REQUEST,
                t!("common.request_body_must_be_json", locale = locale).to_string(),
            ));
        }
    };

    // Deserialize the JSON to the target DTO type
    let dto = serde_json::from_value(body).map_err(|e| {
        ErrorDTO::new(
            StatusCode::BAD_REQUEST,
            t!("common.invalid_request_body", error = e, locale = locale).to_string(),
        )
    })?;

    Ok((dto, fields))
}

pub trait ToJson: serde::Serialize {
    fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or_else(|_| {
            json!({
                "error": t!("common.serialize_error_failed", locale = "en")
            })
        })
    }

    fn to_json_string(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| {
            json!({
                "error": t!("common.serialize_error_failed", locale = "en")
            })
            .to_string()
        })
    }
}

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    use serde::{Deserialize, Serialize};
    use serde_json::json;

    use super::{ToJson, deserialize_with_fields, serialize_status_code};
    use crate::core::dto::error_dto::ErrorDTO;

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    struct TestDto {
        name: String,
        age: i32,
        email: Option<String>,
    }

    #[derive(Serialize)]
    struct StatusHolder {
        #[serde(serialize_with = "serialize_status_code")]
        status: StatusCode,
    }

    #[derive(Serialize)]
    struct JsonHolder {
        value: &'static str,
    }

    impl ToJson for JsonHolder {}

    #[test]
    fn serializes_status_code_as_number() {
        let serialized = serde_json::to_string(&StatusHolder {
            status: StatusCode::NOT_FOUND,
        })
        .unwrap();

        assert!(serialized.contains("\"status\":404"));
    }

    #[test]
    fn deserializes_body_and_collects_fields() {
        let result: Result<(TestDto, Vec<String>), ErrorDTO> = deserialize_with_fields(
            json!({
                "name": "John",
                "age": 30,
                "email": "john@example.com"
            }),
            "en",
        );

        let (dto, fields) = result.unwrap();
        assert_eq!(dto.name, "John");
        assert_eq!(dto.age, 30);
        assert_eq!(dto.email.as_deref(), Some("john@example.com"));
        assert!(fields.contains(&"name".to_string()));
        assert!(fields.contains(&"email".to_string()));
    }

    #[test]
    fn rejects_non_object_body() {
        let result: Result<(TestDto, Vec<String>), ErrorDTO> =
            deserialize_with_fields(json!("not-an-object"), "en");

        let error = result.unwrap_err();
        assert_eq!(error.status, StatusCode::BAD_REQUEST);
        assert_eq!(error.message, "Request body must be a JSON object");
    }

    #[test]
    fn serializes_to_json_helpers() {
        let json = JsonHolder { value: "ok" }.to_json();
        assert_eq!(json["value"], "ok");
        assert!(
            JsonHolder { value: "ok" }
                .to_json_string()
                .contains("\"ok\"")
        );
    }
}
