use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::common::{MessageId, Surface};
use crate::component::Component;
use crate::data::{PatchOperation, ValidationError};

/// Tagged union of all protocol message types.
///
/// Serializes with a `"type"` discriminator tag matching the OpenAPI spec.
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

/// Incremental data update via patch operations.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PatchMessage {
    /// Correlation ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<MessageId>,

    /// Array of patch operations to apply.
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
