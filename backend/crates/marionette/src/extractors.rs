use serde::de::DeserializeOwned;

use marionette_protocol::ActionMessage;

use crate::error::ActionError;

/// Typed extractor for deserializing action payload.
#[derive(Debug)]
pub struct Payload<T>(pub T);

/// Typed extractor providing database access.
#[derive(Debug, Clone)]
pub struct Db(pub sea_orm::DatabaseConnection);

/// Current session information.
#[derive(Debug, Clone)]
pub struct Session {
    /// Authenticated user ID, or `None` for anonymous sessions.
    pub user_id: Option<String>,
    /// Roles assigned to the current user.
    pub roles: Vec<String>,
}

/// Context passed to action handlers during dispatch.
pub struct HandlerContext {
    /// The incoming action message.
    pub action: ActionMessage,
    /// Database connection.
    pub db: sea_orm::DatabaseConnection,
    /// Current session.
    pub session: Session,
}

/// Trait for extracting typed values from the handler context.
pub trait FromHandlerContext: Sized {
    /// Extract this type from the given handler context.
    ///
    /// # Errors
    ///
    /// Returns `ActionError` if extraction fails (e.g., payload deserialization error).
    fn from_context(ctx: &HandlerContext) -> Result<Self, ActionError>;
}

impl<T: DeserializeOwned> FromHandlerContext for Payload<T> {
    fn from_context(ctx: &HandlerContext) -> Result<Self, ActionError> {
        let value = ctx
            .action
            .payload
            .as_ref()
            .cloned()
            .unwrap_or(serde_json::Value::Null);
        serde_json::from_value(value)
            .map(Payload)
            .map_err(|e| ActionError::BadPayload(e.to_string()))
    }
}

impl FromHandlerContext for Db {
    fn from_context(ctx: &HandlerContext) -> Result<Self, ActionError> {
        Ok(Db(ctx.db.clone()))
    }
}

impl FromHandlerContext for Session {
    fn from_context(ctx: &HandlerContext) -> Result<Self, ActionError> {
        Ok(ctx.session.clone())
    }
}
