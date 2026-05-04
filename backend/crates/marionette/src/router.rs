use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use marionette_protocol::common::AuthRequirement;
use marionette_protocol::ProtocolMessage;

use crate::error::{ActionError, ActionResult};
use crate::extractors::HandlerContext;

/// A boxed async handler function.
pub type BoxedHandler = Box<
    dyn Fn(HandlerContext) -> Pin<Box<dyn Future<Output = ActionResult> + Send>> + Send + Sync,
>;

/// Entry in the action router mapping.
struct HandlerEntry {
    handler: BoxedHandler,
    auth: AuthRequirement,
}

/// Routes incoming action messages to registered handler functions by name.
pub struct ActionRouter {
    handlers: HashMap<String, HandlerEntry>,
}

impl ActionRouter {
    /// Create a new empty router.
    #[must_use]
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    /// Register an action handler with its name and authorization requirement.
    #[must_use]
    pub fn action(mut self, name: &str, handler: BoxedHandler, auth: AuthRequirement) -> Self {
        self.handlers.insert(
            name.to_string(),
            HandlerEntry { handler, auth },
        );
        self
    }

    /// Dispatch an action message to the appropriate handler.
    ///
    /// Looks up the handler by action name, checks authorization, then calls the handler.
    /// Returns protocol messages on success or error messages on failure.
    pub async fn dispatch(&self, ctx: HandlerContext) -> Vec<ProtocolMessage> {
        let name = &ctx.action.name;
        let Some(entry) = self.handlers.get(name) else {
            return Vec::from(ActionError::NotFound(name.clone()));
        };

        if let Err(e) = crate::auth::check_auth(&entry.auth, &ctx.session) {
            return Vec::from(e);
        }

        match (entry.handler)(ctx).await {
            Ok(messages) => messages,
            Err(e) => Vec::from(e),
        }
    }
}

impl Default for ActionRouter {
    fn default() -> Self {
        Self::new()
    }
}

/// Wrap an async handler function into a [`BoxedHandler`].
///
/// # Examples
///
/// ```ignore
/// let handler = box_handler(|ctx| async move { Ok(vec![]) });
/// ```
pub fn box_handler<F, Fut>(f: F) -> BoxedHandler
where
    F: Fn(HandlerContext) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ActionResult> + Send + 'static,
{
    Box::new(move |ctx| Box::pin(f(ctx)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use marionette_protocol::{ActionMessage, ErrorMessage, PatchMessage, PatchOperation};
    use sea_orm::{DatabaseBackend, MockDatabase};

    use crate::extractors::Session;

    fn mock_db() -> std::sync::Arc<sea_orm::DatabaseConnection> {
        std::sync::Arc::new(MockDatabase::new(DatabaseBackend::Sqlite).into_connection())
    }

    fn make_ctx(name: &str, session: Session) -> HandlerContext {
        HandlerContext {
            action: ActionMessage {
                id: None,
                name: name.into(),
                source: None,
                payload: None,
                optimistic: None,
            },
            db: mock_db(),
            session,
            extensions: crate::extensions::Extensions::new(),
        }
    }

    fn anonymous() -> Session {
        Session {
            user_id: None,
            roles: vec![],
        }
    }

    fn authed(user_id: &str) -> Session {
        Session {
            user_id: Some(user_id.into()),
            roles: vec![],
        }
    }

    fn echo_handler() -> BoxedHandler {
        box_handler(|_ctx| async move {
            Ok(vec![ProtocolMessage::Patch(PatchMessage {
                id: None,
                surface: "main".into(),
                patch: vec![PatchOperation::Set {
                    path: "/test".into(),
                    value: serde_json::json!("ok"),
                }],
            })])
        })
    }

    fn error_handler() -> BoxedHandler {
        box_handler(|_ctx| async move {
            Err(ActionError::Internal("boom".into()))
        })
    }

    #[tokio::test]
    async fn router_dispatches_to_correct_handler() {
        let router = ActionRouter::new()
            .action("echo", echo_handler(), AuthRequirement::None)
            .action("fail", error_handler(), AuthRequirement::None);

        let result = router.dispatch(make_ctx("echo", anonymous())).await;
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0], ProtocolMessage::Patch(_)));
    }

    #[tokio::test]
    async fn router_returns_not_found_for_unknown() {
        let router = ActionRouter::new()
            .action("echo", echo_handler(), AuthRequirement::None);

        let result = router.dispatch(make_ctx("unknown", anonymous())).await;
        assert_eq!(result.len(), 1);
        match &result[0] {
            ProtocolMessage::Error(ErrorMessage { errors, .. }) => {
                assert!(errors[0].message.contains("Action not found"));
                assert!(errors[0].message.contains("unknown"));
            }
            other => panic!("Expected Error, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn router_checks_auth_before_handler() {
        let router = ActionRouter::new()
            .action("protected", echo_handler(), AuthRequirement::Authenticated);

        let result = router.dispatch(make_ctx("protected", anonymous())).await;
        assert_eq!(result.len(), 1);
        match &result[0] {
            ProtocolMessage::Error(ErrorMessage { errors, .. }) => {
                assert!(errors[0].message.contains("Unauthorized"));
            }
            other => panic!("Expected Error, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn router_allows_public_handler() {
        let router = ActionRouter::new()
            .action("public", echo_handler(), AuthRequirement::None);

        let result = router.dispatch(make_ctx("public", anonymous())).await;
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0], ProtocolMessage::Patch(_)));
    }

    #[tokio::test]
    async fn router_allows_authenticated_handler_with_user() {
        let router = ActionRouter::new()
            .action("protected", echo_handler(), AuthRequirement::Authenticated);

        let result = router.dispatch(make_ctx("protected", authed("u1"))).await;
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0], ProtocolMessage::Patch(_)));
    }

    #[tokio::test]
    async fn router_handler_error_converts_to_error_message() {
        let router = ActionRouter::new()
            .action("fail", error_handler(), AuthRequirement::None);

        let result = router.dispatch(make_ctx("fail", anonymous())).await;
        assert_eq!(result.len(), 1);
        match &result[0] {
            ProtocolMessage::Error(ErrorMessage { errors, .. }) => {
                assert!(errors[0].message.contains("Internal error"));
                assert!(errors[0].message.contains("boom"));
            }
            other => panic!("Expected Error, got {other:?}"),
        }
    }
}
