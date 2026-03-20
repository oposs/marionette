/// Named render target in the frontend layout (e.g., "main", "sidebar", "modal", "toast").
pub type Surface = String;

/// RFC 6901 JSON Pointer path. Used for data binding and patch targeting.
pub type JsonPointer = String;

/// Optional correlation ID set by client, echoed by server.
pub type MessageId = String;

/// Authorization requirement for action handlers.
///
/// Used by the `#[requires]` attribute macro to generate compile-time
/// authorization metadata on handler functions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthRequirement {
    /// No authentication required (public endpoint).
    None,
    /// User must be authenticated.
    Authenticated,
    /// User must have the specified role.
    Role(&'static str),
}
