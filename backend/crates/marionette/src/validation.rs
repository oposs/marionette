//! Phase 15 D-D3 — per-field validation patch builder.
//!
//! Save handlers that detect per-field validation failures return
//! `Ok(vec![validation_error_patch("content", errors)])` instead of
//! `Err(ActionError::BadPayload(...))`. The resulting `PatchMessage`
//! carries one `SetData` op per invalid field, targeted at
//! `/_errors{bind}` — the path shape the frontend `Field.Error`
//! primitive reads (see `frontend/src/lib/components/form/TextInput.svelte:30`).
//!
//! `ActionError::BadPayload` stays reserved for protocol-layer failures
//! (JSON parse, missing `form_bind`, auth / DB errors).
//!
//! # Security invariant
//!
//! Bind paths passed to this helper MUST be server-derived string
//! literals (e.g., `"/contactForm/name"`). NEVER interpolate
//! user-supplied input into a bind path — doing so would allow
//! clients to overwrite arbitrary `/_errors/*` entries via crafted
//! action payloads.

#[cfg(test)]
mod tests {
    use super::*;
    use marionette_protocol::data::PatchOperation;
    use marionette_protocol::ProtocolMessage;

    #[test]
    fn validation_error_patch_shapes_single_error() {
        let msg = validation_error_patch(
            "content",
            vec![("/contactForm/name", "Name is required.")],
        );
        let ProtocolMessage::Patch(pm) = msg else {
            panic!("expected Patch variant");
        };
        assert_eq!(pm.surface, "content");
        assert_eq!(pm.id, None);
        assert_eq!(pm.patch.len(), 1);
        let PatchOperation::Set { path, value } = &pm.patch[0] else {
            panic!("expected Set op");
        };
        assert_eq!(path, "/_errors/contactForm/name");
        assert_eq!(
            value,
            &serde_json::Value::String("Name is required.".into())
        );
    }

    #[test]
    fn validation_error_patch_shapes_multi_field() {
        let msg = validation_error_patch(
            "content",
            vec![
                ("/userForm/email", "Email is required."),
                (
                    "/userForm/password",
                    "Password must be at least 8 characters.",
                ),
            ],
        );
        let ProtocolMessage::Patch(pm) = msg else {
            panic!("expected Patch");
        };
        assert_eq!(pm.patch.len(), 2);
        // Order preserved.
        let PatchOperation::Set { path: p0, .. } = &pm.patch[0] else {
            panic!("expected Set op at index 0");
        };
        assert_eq!(p0, "/_errors/userForm/email");
        let PatchOperation::Set { path: p1, .. } = &pm.patch[1] else {
            panic!("expected Set op at index 1");
        };
        assert_eq!(p1, "/_errors/userForm/password");
    }

    #[test]
    fn validation_error_patch_empty_iter_returns_empty_patch() {
        let errors: Vec<(&str, &str)> = vec![];
        let msg = validation_error_patch("content", errors);
        let ProtocolMessage::Patch(pm) = msg else {
            panic!("expected Patch");
        };
        assert!(pm.patch.is_empty());
        assert_eq!(pm.surface, "content");
        assert_eq!(pm.id, None);
    }
}
