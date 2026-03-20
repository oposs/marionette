use serde::{Deserialize, Serialize};

/// UI component definition matching spec/schemas/component.yaml.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Component {
    /// Component type identifier (open set).
    #[serde(rename = "type")]
    pub r#type: String,

    /// Component-specific properties.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub props: Option<serde_json::Value>,

    /// Ordered list of child node IDs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<String>>,

    /// JSON Pointer (RFC 6901) to the data this component reads/writes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bind: Option<String>,

    /// Action triggered by this component.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<ComponentAction>,

    /// JSON Pointer to a boolean controlling visibility.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visible: Option<String>,
}

/// Action definition for a component, matching spec/schemas/component.yaml#ComponentAction.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ComponentAction {
    /// Action type identifier.
    #[serde(rename = "type")]
    pub r#type: String,

    /// Action name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Target identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,

    /// JSON Pointer for ID path.
    #[serde(rename = "idPath", skip_serializing_if = "Option::is_none")]
    pub id_path: Option<String>,

    /// Additional properties (open-ended).
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}
