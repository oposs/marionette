/// Named render target in the frontend layout (e.g., "main", "sidebar", "modal", "toast").
pub type Surface = String;

/// RFC 6901 JSON Pointer path. Used for data binding and patch targeting.
pub type JsonPointer = String;

/// Optional correlation ID set by client, echoed by server.
pub type MessageId = String;
