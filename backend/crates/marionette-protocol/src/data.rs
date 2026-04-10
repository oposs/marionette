use serde::{Deserialize, Serialize};

use crate::component::Component;

/// A single patch operation applied to a surface.
///
/// Operations inside a `PatchMessage.patch` array are applied in declared order, all-or-nothing.
/// Data operations (`Set`) and node-tree operations can be mixed freely in one batch.
/// Serialized with a `"op"` discriminator tag using kebab-case variant names.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "op", rename_all = "kebab-case")]
#[allow(clippy::large_enum_variant)]
pub enum PatchOperation {
    /// Data op — set a value at a JSON Pointer path in the surface's data store.
    Set {
        path: String,
        value: serde_json::Value,
    },
    /// Node op — replace (or create) the component at this node ID.
    SetNode {
        id: String,
        component: Component,
    },
    /// Node op — delete the node with this ID from the surface's adjacency list.
    DeleteNode { id: String },
    /// Node op — replace the children array of the given node.
    SetChildren {
        id: String,
        children: Vec<String>,
    },
    /// Node op — insert an existing child ID into a parent's children array at `index`.
    InsertChild {
        parent: String,
        index: usize,
        #[serde(rename = "childId")]
        child_id: String,
    },
    /// Node op — remove a child ID from a parent's children array.
    RemoveChild {
        parent: String,
        #[serde(rename = "childId")]
        child_id: String,
    },
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
    use crate::component::Component;
    use serde_json::json;

    fn sample_component() -> Component {
        Component {
            r#type: "text-input".into(),
            props: Some(json!({"label": "Name"})),
            children: None,
            bind: Some("/user/name".into()),
            action: None,
            visible: None,
        }
    }

    #[test]
    fn patch_op_set_round_trip() {
        let op = PatchOperation::Set {
            path: "/users/u-123/name".into(),
            value: json!("Alice"),
        };
        let v = serde_json::to_value(&op).unwrap();
        assert_eq!(v["op"], "set");
        assert_eq!(v["path"], "/users/u-123/name");
        assert_eq!(v["value"], "Alice");
        let back: PatchOperation = serde_json::from_value(v).unwrap();
        assert_eq!(back, op);
    }

    #[test]
    fn patch_op_set_node_round_trip() {
        let op = PatchOperation::SetNode {
            id: "field-a".into(),
            component: sample_component(),
        };
        let v = serde_json::to_value(&op).unwrap();
        assert_eq!(v["op"], "set-node");
        assert_eq!(v["id"], "field-a");
        assert_eq!(v["component"]["type"], "text-input");
        let back: PatchOperation = serde_json::from_value(v).unwrap();
        assert_eq!(back, op);
    }

    #[test]
    fn patch_op_delete_node_round_trip() {
        let op = PatchOperation::DeleteNode { id: "field-b".into() };
        let v = serde_json::to_value(&op).unwrap();
        assert_eq!(v, json!({"op": "delete-node", "id": "field-b"}));
        let back: PatchOperation = serde_json::from_value(v).unwrap();
        assert_eq!(back, op);
    }

    #[test]
    fn patch_op_set_children_round_trip() {
        let op = PatchOperation::SetChildren {
            id: "form-1".into(),
            children: vec!["a".into(), "b".into(), "c".into()],
        };
        let v = serde_json::to_value(&op).unwrap();
        assert_eq!(v["op"], "set-children");
        assert_eq!(v["id"], "form-1");
        assert_eq!(v["children"], json!(["a", "b", "c"]));
        let back: PatchOperation = serde_json::from_value(v).unwrap();
        assert_eq!(back, op);
    }

    #[test]
    fn patch_op_insert_child_round_trip() {
        let op = PatchOperation::InsertChild {
            parent: "form-1".into(),
            index: 2,
            child_id: "new-field".into(),
        };
        let v = serde_json::to_value(&op).unwrap();
        assert_eq!(
            v,
            json!({"op": "insert-child", "parent": "form-1", "index": 2, "childId": "new-field"})
        );
        let back: PatchOperation = serde_json::from_value(v).unwrap();
        assert_eq!(back, op);
    }

    #[test]
    fn patch_op_remove_child_round_trip() {
        let op = PatchOperation::RemoveChild {
            parent: "form-1".into(),
            child_id: "old-field".into(),
        };
        let v = serde_json::to_value(&op).unwrap();
        assert_eq!(
            v,
            json!({"op": "remove-child", "parent": "form-1", "childId": "old-field"})
        );
        let back: PatchOperation = serde_json::from_value(v).unwrap();
        assert_eq!(back, op);
    }

    #[test]
    fn patch_op_unknown_discriminator_rejected() {
        let v = json!({"op": "swap-root", "id": "x"});
        let result: Result<PatchOperation, _> = serde_json::from_value(v);
        assert!(result.is_err(), "unknown op must be rejected by tagged enum");
    }

    #[test]
    fn validation_error_round_trip() {
        let err = ValidationError {
            path: Some("/email".into()),
            message: "Invalid email format".into(),
        };
        let v = serde_json::to_value(&err).unwrap();
        assert_eq!(v, json!({"path": "/email", "message": "Invalid email format"}));
        let back: ValidationError = serde_json::from_value(v).unwrap();
        assert_eq!(back, err);
    }

    #[test]
    fn validation_error_without_path() {
        let err = ValidationError {
            path: None,
            message: "Server error".into(),
        };
        let v = serde_json::to_value(&err).unwrap();
        assert_eq!(v, json!({"message": "Server error"}));
        let back: ValidationError = serde_json::from_value(v).unwrap();
        assert_eq!(back, err);
    }
}
