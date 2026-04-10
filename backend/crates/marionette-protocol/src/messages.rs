use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::common::{MessageId, Surface};
use crate::component::Component;
use crate::data::{PatchOperation, ValidationError};

/// Tagged union of all protocol message types.
///
/// Serializes with a `"type"` discriminator tag matching the `OpenAPI` spec.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ProtocolMessage {
    /// Server greeting with protocol version.
    Hello(HelloMessage),
    /// Full surface render with component tree and data.
    Render(RenderMessage),
    /// Incremental data update.
    Patch(PatchMessage),
    /// Client-initiated action.
    Action(ActionMessage),
    /// Lifecycle event.
    Event(EventMessage),
    /// Validation errors.
    Error(ErrorMessage),
}

/// Server greeting message.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HelloMessage {
    /// Protocol version (semver).
    pub version: String,
}

/// Full surface render with component tree and data.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RenderMessage {
    /// Correlation ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<MessageId>,

    /// Target surface name.
    pub surface: Surface,

    /// ID of the root node in the adjacency list.
    pub root: String,

    /// Flat map of node ID to component definition.
    pub nodes: HashMap<String, Component>,

    /// Application state that components bind to.
    pub data: serde_json::Value,
}

/// Incremental update via patch operations applied to a single surface.
///
/// A `PatchMessage` targets exactly one surface and carries a batch of
/// `PatchOperation` entries that are applied in declared order, all-or-nothing.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PatchMessage {
    /// Correlation ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<MessageId>,

    /// Target surface name. Required — one message targets exactly one surface.
    pub surface: Surface,

    /// Array of patch operations to apply, in declared order.
    pub patch: Vec<PatchOperation>,
}

/// Client-initiated action.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ActionMessage {
    /// Correlation ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<MessageId>,

    /// Action identifier.
    pub name: String,

    /// Component ID that triggered the action.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,

    /// Action-specific data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<serde_json::Value>,

    /// Patches to apply immediately (rolled back on error).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optimistic: Option<OptimisticUpdate>,
}

/// Optimistic update containing patches to apply immediately.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OptimisticUpdate {
    /// Patches to apply optimistically.
    pub patch: Vec<PatchOperation>,
}

/// Lifecycle event message.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventMessage {
    /// Correlation ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<MessageId>,

    /// Event identifier.
    pub name: String,

    /// Target surface.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub surface: Option<Surface>,

    /// Event-specific metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<serde_json::Value>,
}

/// Validation error response.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ErrorMessage {
    /// Correlation ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<MessageId>,

    /// Array of validation errors.
    pub errors: Vec<ValidationError>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn hello_round_trip() {
        let msg = ProtocolMessage::Hello(HelloMessage {
            version: "1.1.0".into(),
        });
        let json = serde_json::to_value(&msg).unwrap();
        assert_eq!(json, json!({"type": "hello", "version": "1.1.0"}));

        let deserialized: ProtocolMessage = serde_json::from_value(json).unwrap();
        assert_eq!(deserialized, msg);
    }

    #[test]
    fn render_round_trip() {
        let mut nodes = HashMap::new();
        nodes.insert(
            "page-1".into(),
            Component {
                r#type: "container".into(),
                props: None,
                children: Some(vec!["input-1".into()]),
                bind: None,
                action: None,
                visible: None,
            },
        );

        let msg = ProtocolMessage::Render(RenderMessage {
            id: Some("msg-123".into()),
            surface: "main".into(),
            root: "page-1".into(),
            nodes,
            data: json!({"name": "Alice"}),
        });

        let json = serde_json::to_value(&msg).unwrap();
        assert_eq!(json["type"], "render");
        assert_eq!(json["surface"], "main");
        assert_eq!(json["root"], "page-1");
        assert_eq!(json["id"], "msg-123");
        assert_eq!(json["nodes"]["page-1"]["type"], "container");

        let deserialized: ProtocolMessage = serde_json::from_value(json).unwrap();
        assert_eq!(deserialized, msg);
    }

    #[test]
    fn patch_round_trip() {
        let msg = ProtocolMessage::Patch(PatchMessage {
            id: None,
            surface: "main".into(),
            patch: vec![PatchOperation::Set {
                path: "/user/name".into(),
                value: json!("Bob"),
            }],
        });

        let json = serde_json::to_value(&msg).unwrap();
        assert_eq!(json["type"], "patch");
        assert_eq!(json["surface"], "main");
        assert_eq!(json["patch"][0]["op"], "set");
        assert_eq!(json["patch"][0]["path"], "/user/name");
        assert_eq!(json["patch"][0]["value"], "Bob");

        let deserialized: ProtocolMessage = serde_json::from_value(json).unwrap();
        assert_eq!(deserialized, msg);
    }

    #[test]
    fn patch_message_surface_required() {
        let v = json!({"type": "patch", "patch": []});
        let result: Result<ProtocolMessage, _> = serde_json::from_value(v);
        assert!(result.is_err(), "PatchMessage without surface must be rejected");
    }

    #[test]
    fn patch_message_targets_non_main_surface() {
        let msg = ProtocolMessage::Patch(PatchMessage {
            id: Some("msg-1".into()),
            surface: "modal".into(),
            patch: vec![PatchOperation::DeleteNode { id: "old-modal".into() }],
        });
        let v = serde_json::to_value(&msg).unwrap();
        assert_eq!(v["surface"], "modal");
        assert_eq!(v["patch"][0]["op"], "delete-node");
        let back: ProtocolMessage = serde_json::from_value(v).unwrap();
        assert_eq!(back, msg);
    }

    #[test]
    fn action_round_trip() {
        let msg = ProtocolMessage::Action(ActionMessage {
            id: Some("msg-456".into()),
            name: "save".into(),
            source: Some("btn-1".into()),
            payload: Some(json!({"confirmed": true})),
            optimistic: Some(OptimisticUpdate {
                patch: vec![PatchOperation::Set {
                    path: "/saving".into(),
                    value: json!(true),
                }],
            }),
        });

        let json = serde_json::to_value(&msg).unwrap();
        assert_eq!(json["type"], "action");
        assert_eq!(json["name"], "save");
        assert_eq!(json["source"], "btn-1");
        assert_eq!(json["payload"]["confirmed"], true);
        assert_eq!(json["optimistic"]["patch"][0]["path"], "/saving");

        let deserialized: ProtocolMessage = serde_json::from_value(json).unwrap();
        assert_eq!(deserialized, msg);
    }

    #[test]
    fn event_round_trip() {
        let msg = ProtocolMessage::Event(EventMessage {
            id: None,
            name: "navigate".into(),
            surface: Some("main".into()),
            hint: Some(json!({"url": "/contacts"})),
        });

        let json = serde_json::to_value(&msg).unwrap();
        assert_eq!(json["type"], "event");
        assert_eq!(json["name"], "navigate");
        assert_eq!(json["surface"], "main");

        let deserialized: ProtocolMessage = serde_json::from_value(json).unwrap();
        assert_eq!(deserialized, msg);
    }

    #[test]
    fn error_round_trip() {
        let msg = ProtocolMessage::Error(ErrorMessage {
            id: Some("msg-789".into()),
            errors: vec![
                ValidationError {
                    path: Some("/email".into()),
                    message: "Invalid email".into(),
                },
                ValidationError {
                    path: None,
                    message: "Server error".into(),
                },
            ],
        });

        let json = serde_json::to_value(&msg).unwrap();
        assert_eq!(json["type"], "error");
        assert_eq!(json["errors"][0]["path"], "/email");
        assert_eq!(json["errors"][0]["message"], "Invalid email");
        assert_eq!(json["errors"][1].get("path"), None);
        assert_eq!(json["errors"][1]["message"], "Server error");

        let deserialized: ProtocolMessage = serde_json::from_value(json).unwrap();
        assert_eq!(deserialized, msg);
    }

    #[test]
    fn optional_fields_omitted() {
        let msg = ProtocolMessage::Patch(PatchMessage {
            id: None,
            surface: "main".into(),
            patch: vec![],
        });

        let json = serde_json::to_value(&msg).unwrap();
        assert!(json.get("id").is_none(), "None id should be omitted");

        let event = ProtocolMessage::Event(EventMessage {
            id: None,
            name: "init".into(),
            surface: None,
            hint: None,
        });

        let json = serde_json::to_value(&event).unwrap();
        assert!(json.get("id").is_none());
        assert!(json.get("surface").is_none());
        assert!(json.get("hint").is_none());
        // Required fields still present
        assert_eq!(json["type"], "event");
        assert_eq!(json["name"], "init");
    }

    #[test]
    fn deserialize_from_spec_json() {
        // Hello
        let hello: ProtocolMessage =
            serde_json::from_value(json!({"type": "hello", "version": "1.1.0"})).unwrap();
        assert!(matches!(hello, ProtocolMessage::Hello(_)));

        // Render
        let render: ProtocolMessage = serde_json::from_value(json!({
            "type": "render",
            "surface": "main",
            "root": "page-1",
            "nodes": {
                "page-1": { "type": "container", "children": ["input-1"] }
            },
            "data": { "user": { "name": "Alice" } }
        }))
        .unwrap();
        assert!(matches!(render, ProtocolMessage::Render(_)));

        // Action
        let action: ProtocolMessage = serde_json::from_value(json!({
            "type": "action",
            "name": "delete",
            "payload": { "id": "u-123" }
        }))
        .unwrap();
        assert!(matches!(action, ProtocolMessage::Action(_)));

        // Error
        let error: ProtocolMessage = serde_json::from_value(json!({
            "type": "error",
            "errors": [{ "message": "Not found" }]
        }))
        .unwrap();
        assert!(matches!(error, ProtocolMessage::Error(_)));
    }
}
