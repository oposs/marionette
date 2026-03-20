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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn component_round_trip() {
        let component = Component {
            r#type: "text-input".into(),
            props: Some(json!({"label": "Name", "placeholder": "Enter name"})),
            children: Some(vec!["child-1".into(), "child-2".into()]),
            bind: Some("/user/name".into()),
            action: Some(ComponentAction {
                r#type: "submit".into(),
                name: Some("save".into()),
                target: Some("form-1".into()),
                id_path: Some("/user/id".into()),
                extra: serde_json::Map::new(),
            }),
            visible: Some("/user/isEditing".into()),
        };

        let json = serde_json::to_value(&component).unwrap();
        // Verify "type" field is serialized as "type", not "r#type"
        assert_eq!(json["type"], "text-input");
        assert_eq!(json["bind"], "/user/name");
        assert_eq!(json["children"][0], "child-1");
        assert_eq!(json["action"]["type"], "submit");
        assert_eq!(json["action"]["idPath"], "/user/id");
        assert_eq!(json["visible"], "/user/isEditing");

        let deserialized: Component = serde_json::from_value(json).unwrap();
        assert_eq!(deserialized, component);
    }

    #[test]
    fn component_minimal() {
        let component = Component {
            r#type: "container".into(),
            props: None,
            children: None,
            bind: None,
            action: None,
            visible: None,
        };

        let json = serde_json::to_value(&component).unwrap();
        assert_eq!(json, json!({"type": "container"}));

        let deserialized: Component = serde_json::from_value(json).unwrap();
        assert_eq!(deserialized, component);
    }

    #[test]
    fn component_action_extra_fields() {
        let mut extra = serde_json::Map::new();
        extra.insert("confirmText".into(), json!("Are you sure?"));
        extra.insert("style".into(), json!("danger"));

        let action = ComponentAction {
            r#type: "delete".into(),
            name: Some("remove".into()),
            target: None,
            id_path: None,
            extra,
        };

        let json = serde_json::to_value(&action).unwrap();
        // Extra fields should be at the top level, not nested
        assert_eq!(json["type"], "delete");
        assert_eq!(json["name"], "remove");
        assert_eq!(json["confirmText"], "Are you sure?");
        assert_eq!(json["style"], "danger");
        // target and idPath should be omitted
        assert!(json.get("target").is_none());
        assert!(json.get("idPath").is_none());

        let deserialized: ComponentAction = serde_json::from_value(json).unwrap();
        assert_eq!(deserialized, action);
    }
}
