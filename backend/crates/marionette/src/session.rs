use std::time::Instant;

use uuid::Uuid;

use crate::extractors::Session;

/// WebSocket session state tracking.
///
/// Created when a client connects via WebSocket. Tracks session identity,
/// authentication state, and connection timing.
pub struct WsSession {
    /// Unique session identifier.
    pub id: String,
    /// Authenticated user ID (set after authentication).
    pub user_id: Option<String>,
    /// Roles assigned to the session user.
    pub roles: Vec<String>,
    /// When the WebSocket connection was established.
    pub connected_at: Instant,
}

impl WsSession {
    /// Create a new anonymous session with a generated UUID.
    #[must_use]
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            user_id: None,
            roles: vec![],
            connected_at: Instant::now(),
        }
    }

    /// Convert this session into the [`Session`] extractor type used by handlers.
    #[must_use]
    pub fn to_session(&self) -> Session {
        Session {
            user_id: self.user_id.clone(),
            roles: self.roles.clone(),
        }
    }
}

impl Default for WsSession {
    fn default() -> Self {
        Self::new()
    }
}
