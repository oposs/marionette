use serde::{Deserialize, Serialize};

/// A single patch operation targeting a JSON Pointer path.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PatchOperation {
    /// JSON Pointer to the data location to update.
    pub path: String,

    /// New value to set at the path.
    pub value: serde_json::Value,
}

/// A validation error returned by the server.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ValidationError {
    /// Data path the error relates to (optional for global errors).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,

    /// Human-readable error message.
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn patch_operation_round_trip() {
        let op = PatchOperation {
            path: "/users/u-123/name".into(),
            value: json!("Alice Smith"),
        };

        let json = serde_json::to_value(&op).unwrap();
        assert_eq!(
            json,
            json!({"path": "/users/u-123/name", "value": "Alice Smith"})
        );

        let deserialized: PatchOperation = serde_json::from_value(json).unwrap();
        assert_eq!(deserialized, op);
    }

    #[test]
    fn patch_operation_complex_value() {
        let op = PatchOperation {
            path: "/settings".into(),
            value: json!({"theme": "dark", "lang": "en"}),
        };

        let json = serde_json::to_value(&op).unwrap();
        let deserialized: PatchOperation = serde_json::from_value(json).unwrap();
        assert_eq!(deserialized, op);
    }

    #[test]
    fn validation_error_round_trip() {
        let err = ValidationError {
            path: Some("/email".into()),
            message: "Invalid email format".into(),
        };

        let json = serde_json::to_value(&err).unwrap();
        assert_eq!(
            json,
            json!({"path": "/email", "message": "Invalid email format"})
        );

        let deserialized: ValidationError = serde_json::from_value(json).unwrap();
        assert_eq!(deserialized, err);
    }

    #[test]
    fn validation_error_without_path() {
        let err = ValidationError {
            path: None,
            message: "Server error".into(),
        };

        let json = serde_json::to_value(&err).unwrap();
        assert_eq!(json, json!({"message": "Server error"}));
        assert!(json.get("path").is_none());

        let deserialized: ValidationError = serde_json::from_value(json).unwrap();
        assert_eq!(deserialized, err);
    }
}
