use std::sync::Arc;

use serde::de::DeserializeOwned;

use marionette_protocol::ActionMessage;

use crate::error::ActionError;

/// Typed extractor for deserializing action payload.
#[derive(Debug)]
pub struct Payload<T>(pub T);

/// Typed extractor providing database access.
///
/// Wraps the connection in `Arc` for cheap cloning across extractors and handlers.
#[derive(Debug, Clone)]
pub struct Db(pub Arc<sea_orm::DatabaseConnection>);

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
    /// Database connection (shared via Arc).
    pub db: Arc<sea_orm::DatabaseConnection>,
    /// Current session.
    pub session: Session,
    /// Type-keyed registry of app-defined services, cloned from
    /// [`AppState`]. Handlers reach app state through this rather than
    /// crate-local globals. See [`crate::extensions`].
    ///
    /// [`AppState`]: crate::ws::AppState
    pub extensions: crate::extensions::Extensions,
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
            .clone()
            .unwrap_or(serde_json::Value::Null);
        serde_json::from_value(value)
            .map(Payload)
            .map_err(|e| ActionError::BadPayload(e.to_string()))
    }
}

impl FromHandlerContext for Db {
    fn from_context(ctx: &HandlerContext) -> Result<Self, ActionError> {
        Ok(Db(Arc::clone(&ctx.db)))
    }
}

impl FromHandlerContext for Session {
    fn from_context(ctx: &HandlerContext) -> Result<Self, ActionError> {
        Ok(ctx.session.clone())
    }
}
