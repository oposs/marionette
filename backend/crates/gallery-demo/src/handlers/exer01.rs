//! EXER-01 handlers (Plan 19-02):
//! - `handle_exer01_report` — receives the observation payload from the
//!   frontend probe and writes Set ops into /demo/exer-01/matrix/{dim} + the
//!   per-cell /details subpath (the findings-Text bind target).
//! - `handle_exer01_open_seed` — emits a toast whose text carries the seed
//!   file path so the user can open `.planning/seeds/v1.3-appshell-nestability.md`
//!   in their editor of choice. Inline toast-emission (copied verbatim from
//!   handlers/toast.rs::handle_toast_fire) — Plan 19-02 does not add a
//!   toast-helper extraction.
//!
//! Threat model (from 19-02-PLAN.md):
//! - T-19-02-01 (Tampering on ObservationReport): strict serde Deserialize;
//!   missing any of the 4 dimensions returns `ActionError::BadPayload`.
//!   `state` and `details` are plain `String`s echoed into SDUI Text nodes
//!   (no HTML interpolation) — no XSS vector.
//! - T-19-02-02 (Tampering on open-seed path): `path` is plain String echoed
//!   into a Toast Button label (SDUI renders text verbatim) — no XSS.

use marionette::builders::Button;
use marionette::error::{ActionError, ActionResult};
use marionette::extractors::HandlerContext;
use marionette_protocol::data::PatchOperation;
use marionette_protocol::messages::PatchMessage;
use marionette_protocol::{ComponentAction, ProtocolMessage};
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

    // Two-op sequence copied verbatim from handlers/toast.rs::handle_toast_fire
    // (lines 11-36) — SetNode for the toast Button, InsertChild into the
    // "toasts-root" container. Surface is "toasts". Clicking the toast fires
    // the shared `dismiss-toast` handler (toast.rs:38-60).
    let toast_id = format!("toast-exer01-open-seed-{}", uuid::Uuid::new_v4());
    let toast_label = format!("Open seed draft: {}", payload.path);

    let (_, toast_node) = Button::new(&toast_label)
        .id(&toast_id)
        .action(ComponentAction::click("dismiss-toast"))
        .build();

    let ops = vec![
        PatchOperation::SetNode {
            id: toast_id.clone(),
            component: toast_node,
        },
        PatchOperation::InsertChild {
            parent: "toasts-root".into(),
            index: 0,
            child_id: toast_id,
        },
    ];

    Ok(vec![ProtocolMessage::Patch(PatchMessage {
        id: ctx.action.id.clone(),
        surface: "toasts".into(),
        patch: ops,
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
    async fn open_seed_emits_toast_with_seed_path() {
        let ctx = make_ctx(
            "gallery-demo/exer-01/open-seed",
            serde_json::json!({ "path": ".planning/seeds/v1.3-appshell-nestability.md" }),
        );
        let out = handle_exer01_open_seed(ctx).await.expect("ok");
        let msg = unwrap_patch(&out);
        assert_eq!(msg.surface, "toasts");

        // Expect exactly one SetNode + one InsertChild into "toasts-root".
        let mut saw_set_node_with_path = false;
        let mut saw_insert_child_into_toasts_root = false;
        for op in &msg.patch {
            match op {
                PatchOperation::SetNode { component, .. } => {
                    // Button label holds the seed path (see handler body).
                    let j = serde_json::to_value(component).expect("serialize");
                    let label = j["props"]["label"].as_str().unwrap_or_default();
                    assert!(
                        label.contains(".planning/seeds/v1.3-appshell-nestability.md"),
                        "toast label should carry seed path, got: {label}"
                    );
                    saw_set_node_with_path = true;
                }
                PatchOperation::InsertChild { parent, .. } => {
                    assert_eq!(parent, "toasts-root");
                    saw_insert_child_into_toasts_root = true;
                }
                _ => {}
            }
        }
        assert!(saw_set_node_with_path, "missing SetNode carrying seed path");
        assert!(
            saw_insert_child_into_toasts_root,
            "missing InsertChild into toasts-root"
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
