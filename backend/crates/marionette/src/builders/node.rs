//! Node type alias and helper functions for component builders.

use marionette_protocol::Component;

/// A node is a `(node_id, Component)` pair for insertion into a surface's nodes map.
pub type Node = (String, Component);

/// Generate a unique node ID with the given prefix.
///
/// Format: `"{prefix}-{uuid_v4}"`.
#[must_use]
pub fn node_id(prefix: &str) -> String {
    format!("{prefix}-{}", uuid::Uuid::new_v4())
}
