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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_converts_to_protocol_unauthorized() {
        let err = ActionError::Unauthorized("no access".into());
        let msgs: Vec<ProtocolMessage> = err.into();
        assert_eq!(msgs.len(), 1);
        match &msgs[0] {
            ProtocolMessage::Error(ErrorMessage { errors, .. }) => {
                assert!(errors[0].message.contains("Unauthorized"));
                assert!(errors[0].message.contains("no access"));
            }
            other => panic!("Expected Error, got {other:?}"),
        }
    }

    #[test]
    fn error_converts_to_protocol_not_found() {
        let err = ActionError::NotFound("delete-all".into());
        let msgs: Vec<ProtocolMessage> = err.into();
        match &msgs[0] {
            ProtocolMessage::Error(ErrorMessage { errors, .. }) => {
                assert!(errors[0].message.contains("Action not found"));
                assert!(errors[0].message.contains("delete-all"));
            }
            other => panic!("Expected Error, got {other:?}"),
        }
    }

    #[test]
    fn error_converts_to_protocol_bad_payload() {
        let err = ActionError::BadPayload("missing field".into());
        let msgs: Vec<ProtocolMessage> = err.into();
        match &msgs[0] {
            ProtocolMessage::Error(ErrorMessage { errors, .. }) => {
                assert!(errors[0].message.contains("Bad payload"));
            }
            other => panic!("Expected Error, got {other:?}"),
        }
    }

    #[test]
    fn error_converts_to_protocol_internal() {
        let err = ActionError::Internal("db timeout".into());
        let msgs: Vec<ProtocolMessage> = err.into();
        match &msgs[0] {
            ProtocolMessage::Error(ErrorMessage { errors, .. }) => {
                assert!(errors[0].message.contains("Internal error"));
            }
            other => panic!("Expected Error, got {other:?}"),
        }
    }
}
