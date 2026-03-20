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
