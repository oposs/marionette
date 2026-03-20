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
