//! EXER-01 handlers (Plan 19-02):
//! - `handle_exer01_report` — receives the observation payload from the
//!   frontend probe and writes Set ops into /demo/exer-01/matrix/{dim} + the
//!   per-cell /details subpath (the findings-Text bind target).
//! - `handle_exer01_open_seed` — dispatches a `toast` event carrying the
//!   seed file path. The client renders it via svelte-sonner chrome
//!   (stacking / fade / countdown), not as a persistent SDUI node.
//!   See docs/OpenSDUI-CONCEPT.md §"Where the Client Is Smart" for the protocol-vs-client
//!   boundary this leans on.
//!
//! Threat model (from 19-02-PLAN.md):
//! - T-19-02-01 (Tampering on ObservationReport): strict serde Deserialize;
//!   missing any of the 4 dimensions returns `ActionError::BadPayload`.
//!   `state` and `details` are plain `String`s echoed into SDUI Text nodes
//!   (no HTML interpolation) — no XSS vector.
//! - T-19-02-02 (Tampering on open-seed path): `path` is plain String echoed
//!   into a toast event hint (client renders as text verbatim) — no XSS.

use marionette::error::{ActionError, ActionResult};
use marionette::extractors::HandlerContext;
use marionette_protocol::data::PatchOperation;
use marionette_protocol::messages::{EventMessage, PatchMessage};
use marionette_protocol::ProtocolMessage;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct ObservationReport {
    #[serde(rename = "provider-context")]
    provider_context: MatrixEntry,
    #[serde(rename = "mobile-sheet")]
    mobile_sheet: MatrixEntry,
    #[serde(rename = "keyboard-shortcuts")]
    keyboard_shortcuts: MatrixEntry,
    #[serde(rename = "sidebar-tokens")]
    sidebar_tokens: MatrixEntry,
}

#[derive(Debug, Deserialize, Serialize)]
struct MatrixEntry {
    /// One of "PASS" / "FAIL" / "WARN".
    state: String,
    details: String,
}

/// Handle an EXER-01 observation report from the frontend probe.
///
/// # Panics
/// Panics only if `serde_json::to_value` fails on a `MatrixEntry` — which
/// cannot happen because `MatrixEntry` is a trivial struct of two `String`
/// fields, both of which always serialize successfully. The `.expect()` is
/// therefore an assertion of an invariant, not a fallible operation.
pub async fn handle_exer01_report(ctx: HandlerContext) -> ActionResult {
    let payload: ObservationReport = serde_json::from_value(
        ctx.action.payload.clone().unwrap_or_default(),
    )
    .map_err(|e| ActionError::BadPayload(format!("exer-01 report invalid: {e}")))?;

    // 4 dimensions × 2 ops (cell root + findings text) = 8 Set ops total.
    // The /{dim} Set refreshes the structured state object; the /{dim}/details
    // Set drives the findings Text whose `bind` points at the same path.
    let mut patch = Vec::with_capacity(8);
    for (key, entry) in [
        ("provider-context", &payload.provider_context),
        ("mobile-sheet", &payload.mobile_sheet),
        ("keyboard-shortcuts", &payload.keyboard_shortcuts),
        ("sidebar-tokens", &payload.sidebar_tokens),
    ] {
        patch.push(PatchOperation::Set {
            path: format!("/demo/exer-01/matrix/{key}"),
            value: serde_json::to_value(entry).expect("MatrixEntry serializes"),
        });
        patch.push(PatchOperation::Set {
            path: format!("/demo/exer-01/matrix/{key}/details"),
            value: serde_json::Value::String(entry.details.clone()),
        });
    }

    Ok(vec![ProtocolMessage::Patch(PatchMessage {
        id: ctx.action.id.clone(),
        surface: "content".into(),
        patch,
    })])
}

#[derive(Debug, Deserialize)]
struct OpenSeedPayload {
    path: String,
}

#[allow(clippy::unused_async)]
pub async fn handle_exer01_open_seed(ctx: HandlerContext) -> ActionResult {
    let payload: OpenSeedPayload = serde_json::from_value(
        ctx.action.payload.clone().unwrap_or_default(),
    )
    .map_err(|e| ActionError::BadPayload(format!("open-seed payload invalid: {e}")))?;

    // Dispatch a `toast` event — the client renders via svelte-sonner
    // (stacking / fade / countdown). Protocol owns the message content and
    // severity; client owns the overlay mechanics. See docs/OpenSDUI-CONCEPT.md
    // §"Where the Client Is Smart".
    Ok(vec![ProtocolMessage::Event(EventMessage {
        id: ctx.action.id.clone(),
        name: "toast".into(),
        surface: None,
        hint: Some(serde_json::json!({
            "message": format!("Open seed draft: {}", payload.path),
            "severity": "info",
            "duration": 4000,
        })),
    })])
}

#[cfg(test)]
mod tests {
    use super::*;
    use marionette::extractors::Session;
    use marionette_protocol::ActionMessage;
    use sea_orm::{DatabaseBackend, MockDatabase};
    use std::sync::Arc;

    fn make_ctx(action_name: &str, payload: serde_json::Value) -> HandlerContext {
        HandlerContext {
            action: ActionMessage {
                id: Some("t1".into()),
                name: action_name.into(),
                source: None,
                payload: Some(payload),
                optimistic: None,
            },
            db: Arc::new(MockDatabase::new(DatabaseBackend::Sqlite).into_connection()),
            session: Session {
                user_id: None,
                roles: vec![],
            },
            extensions: marionette::Extensions::new(),
        }
    }

    fn unwrap_patch(msgs: &[ProtocolMessage]) -> &PatchMessage {
        match &msgs[0] {
            ProtocolMessage::Patch(p) => p,
            other => panic!("expected Patch, got {other:?}"),
        }
    }

    // ---------- handle_exer01_report ----------

    #[tokio::test]
    async fn report_writes_four_set_ops() {
        let ctx = make_ctx(
            "gallery-demo/exer-01/report",
            serde_json::json!({
                "provider-context": { "state": "FAIL", "details": "d1" },
                "mobile-sheet": { "state": "FAIL", "details": "d2" },
                "keyboard-shortcuts": { "state": "FAIL", "details": "d3" },
                "sidebar-tokens": { "state": "WARN", "details": "d4" },
            }),
        );
        let out = handle_exer01_report(ctx).await.expect("ok");
        let msg = unwrap_patch(&out);
        let paths: Vec<String> = msg
            .patch
            .iter()
            .filter_map(|op| match op {
                PatchOperation::Set { path, .. } => Some(path.clone()),
                _ => None,
            })
            .collect();
        // Root-per-dimension Set ops must ALL be present.
        for key in [
            "provider-context",
            "mobile-sheet",
            "keyboard-shortcuts",
            "sidebar-tokens",
        ] {
            let root = format!("/demo/exer-01/matrix/{key}");
            let details = format!("/demo/exer-01/matrix/{key}/details");
            assert!(
                paths.iter().any(|p| p == &root),
                "missing matrix root Set for {key}: {paths:?}"
            );
            assert!(
                paths.iter().any(|p| p == &details),
                "missing matrix details Set for {key}: {paths:?}"
            );
        }
        assert_eq!(msg.surface, "content");
    }

    #[tokio::test]
    async fn bad_payload_rejected() {
        // Missing 3 of 4 dimensions — serde deserialisation fails; handler
        // returns ActionError::BadPayload (T-19-02-01 mitigation).
        let ctx = make_ctx(
            "gallery-demo/exer-01/report",
            serde_json::json!({ "provider-context": { "state": "FAIL" } }),
        );
        let result = handle_exer01_report(ctx).await;
        assert!(
            matches!(result, Err(ActionError::BadPayload(_))),
            "expected BadPayload, got {result:?}"
        );
    }

    // ---------- handle_exer01_open_seed ----------

    #[tokio::test]
    async fn open_seed_emits_toast_event_with_seed_path() {
        let ctx = make_ctx(
            "gallery-demo/exer-01/open-seed",
            serde_json::json!({ "path": ".planning/seeds/v1.3-appshell-nestability.md" }),
        );
        let out = handle_exer01_open_seed(ctx).await.expect("ok");
        assert_eq!(out.len(), 1, "expected exactly one ProtocolMessage");

        // Expect a single Event with name="toast" and a hint carrying the
        // seed path as its message (svelte-sonner renders the chrome).
        let event = match &out[0] {
            ProtocolMessage::Event(e) => e,
            other => panic!("expected Event, got {other:?}"),
        };
        assert_eq!(event.name, "toast");
        assert!(event.surface.is_none());

        let hint = event.hint.as_ref().expect("hint present");
        let message = hint
            .get("message")
            .and_then(|v| v.as_str())
            .expect("hint.message is string");
        assert!(
            message.contains(".planning/seeds/v1.3-appshell-nestability.md"),
            "toast hint.message should carry seed path, got: {message}"
        );
        assert_eq!(
            hint.get("severity").and_then(|v| v.as_str()),
            Some("info"),
            "hint.severity should be info"
        );
        assert!(
            hint.get("duration").and_then(serde_json::Value::as_u64).is_some(),
            "hint.duration should be a number"
        );
    }

    #[tokio::test]
    async fn open_seed_rejects_missing_path() {
        let ctx = make_ctx(
            "gallery-demo/exer-01/open-seed",
            serde_json::json!({}),
        );
        let result = handle_exer01_open_seed(ctx).await;
        assert!(
            matches!(result, Err(ActionError::BadPayload(_))),
            "expected BadPayload, got {result:?}"
        );
    }
}
