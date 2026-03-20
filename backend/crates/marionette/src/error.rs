#![allow(clippy::enum_variant_names)]

use marionette_protocol::{ErrorMessage, ProtocolMessage, ValidationError};

/// Errors that can occur during action handling.
#[derive(Debug)]
pub enum ActionError {
    /// Action name not registered in the router.
    NotFound(String),
    /// Authorization check failed.
    Unauthorized(String),
    /// Payload deserialization failed.
    BadPayload(String),
    /// Handler returned an internal error.
    Internal(String),
}

impl std::fmt::Display for ActionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(name) => write!(f, "Action not found: {name}"),
            Self::Unauthorized(msg) => write!(f, "Unauthorized: {msg}"),
            Self::BadPayload(msg) => write!(f, "Bad payload: {msg}"),
            Self::Internal(msg) => write!(f, "Internal error: {msg}"),
        }
    }
}

impl std::error::Error for ActionError {}

impl From<ActionError> for Vec<ProtocolMessage> {
    fn from(err: ActionError) -> Self {
        let message = err.to_string();
        vec![ProtocolMessage::Error(ErrorMessage {
            id: None,
            errors: vec![ValidationError {
                path: None,
                message,
            }],
        })]
    }
}

/// Result type for action handlers.
pub type ActionResult = Result<Vec<ProtocolMessage>, ActionError>;
