//! EXER-02 handlers (Plan 19-03).
//!
//! A1 resolution (per 19-01-SUMMARY.md): client-initiated tick. The frontend
//! fires `gallery-demo/exer-02/tick` every cadence_ms after `start`; each tick's
//! handler returns one real PatchMessage. Backend stores cadence_ms + monotonic
//! tick counter in the once-cell `state()` singleton — no framework-crate edits.
//!
//! Safety properties:
//! - T-19-01 DoS mitigation: `handle_exer02_start` clamps cadence_ms to
//!   `[CADENCE_MIN_MS, CADENCE_MAX_MS]` (= [100, 60 000] ms). Out-of-range
//!   → `ActionError::BadPayload`. Tested for under-floor and above-ceiling.
//! - Pitfall 2 (19-RESEARCH.md §604-609): the tick handler MUST emit ops
//!   targeting siblings of the focused input, NEVER the focused input itself.
//!   Enforced at construction (no code path builds such a path/id) + a
//!   `debug_assert!` loop over every op before return + a unit test that
//!   runs 30 ticks and grovels through every emitted op.
//! - Pitfall 10 (19-RESEARCH.md §660-665): the log-row ring-buffer caps at
//!   `LOG_RING_BUFFER_MAX`; the ghost-eviction branch emits `DeleteNode`
//!   for indices older than the cap. `applyPatch` tolerates unknown ids,
//!   so soft-fail is safe.

use marionette::error::{ActionError, ActionResult};
use marionette::extractors::HandlerContext;
use marionette_protocol::data::PatchOperation;
use marionette_protocol::messages::PatchMessage;
use marionette_protocol::{Component, ProtocolMessage};
use serde::Deserialize;

const CADENCE_MIN_MS: u64 = 100; // T-19-01 floor
const CADENCE_MAX_MS: u64 = 60_000; // T-19-01 ceiling
const LOG_RING_BUFFER_MAX: u64 = 200; // Pitfall 10

#[derive(Debug, Deserialize, Default)]
struct StartPayload {
    /// Cadence in ms; must be in [CADENCE_MIN_MS, CADENCE_MAX_MS]. Defaults to 500.
    #[serde(default = "default_cadence_ms")]
    cadence_ms: u64,
}

fn default_cadence_ms() -> u64 {
    500
}

// ---------- handle_exer02_start ----------

pub async fn handle_exer02_start(ctx: HandlerContext) -> ActionResult {
    // Missing payload → default StartPayload (cadence_ms = 500). Null is NOT
    // a valid struct payload under serde, so treat absence as empty-object.
    let raw = ctx
        .action
        .payload
        .clone()
        .unwrap_or_else(|| serde_json::json!({}));
    let payload: StartPayload = serde_json::from_value(raw)
        .map_err(|e| ActionError::BadPayload(format!("exer-02 start payload invalid: {e}")))?;

    if !(CADENCE_MIN_MS..=CADENCE_MAX_MS).contains(&payload.cadence_ms) {
        return Err(ActionError::BadPayload(format!(
            "cadence_ms {} out of range [{}, {}]",
            payload.cadence_ms, CADENCE_MIN_MS, CADENCE_MAX_MS
        )));
    }

    let gs = crate::state::state();
    *gs.exer02_cadence_ms.lock().await = payload.cadence_ms;
    *gs.exer02_tick.lock().await = 0;

    // Ack patch: /demo/exer-02/running = true + cadence mirror + elapsed reset.
    Ok(vec![ProtocolMessage::Patch(PatchMessage {
        id: ctx.action.id.clone(),
        surface: "content".into(),
        patch: vec![
            PatchOperation::Set {
                path: "/demo/exer-02/running".into(),
                value: serde_json::Value::Bool(true),
            },
            PatchOperation::Set {
                path: "/demo/exer-02/cadence-ms".into(),
                value: serde_json::json!(payload.cadence_ms),
            },
            PatchOperation::Set {
                path: "/demo/exer-02/elapsed-s".into(),
                value: serde_json::json!(0),
            },
            PatchOperation::Set {
                path: "/demo/exer-02/elapsed-display".into(),
                value: serde_json::Value::String("0 s elapsed".into()),
            },
        ],
    })])
}

// ---------- handle_exer02_pause ----------

#[allow(clippy::unused_async)]
pub async fn handle_exer02_pause(ctx: HandlerContext) -> ActionResult {
    Ok(vec![ProtocolMessage::Patch(PatchMessage {
        id: ctx.action.id.clone(),
        surface: "content".into(),
        patch: vec![PatchOperation::Set {
            path: "/demo/exer-02/running".into(),
            value: serde_json::Value::Bool(false),
        }],
    })])
}

// ---------- handle_exer02_reset ----------

pub async fn handle_exer02_reset(ctx: HandlerContext) -> ActionResult {
    let gs = crate::state::state();
    *gs.exer02_tick.lock().await = 0;

    Ok(vec![ProtocolMessage::Patch(PatchMessage {
        id: ctx.action.id.clone(),
        surface: "content".into(),
        patch: vec![
            PatchOperation::Set {
                path: "/demo/exer-02/running".into(),
                value: serde_json::Value::Bool(false),
            },
            PatchOperation::SetChildren {
                id: "exer-02-log-container".into(),
                children: vec![],
            },
            PatchOperation::Set {
                path: "/demo/exer-02/invariants/focus".into(),
                value: serde_json::json!({"state": "PENDING", "details": ""}),
            },
            PatchOperation::Set {
                path: "/demo/exer-02/invariants/cursor".into(),
                value: serde_json::json!({"state": "PENDING", "details": ""}),
            },
            PatchOperation::Set {
                path: "/demo/exer-02/invariants/typed".into(),
                value: serde_json::json!({"state": "PENDING", "details": ""}),
            },
            PatchOperation::Set {
                path: "/demo/exer-02/invariants/ime".into(),
                value: serde_json::json!({"state": "PENDING", "details": ""}),
            },
            PatchOperation::Set {
                path: "/demo/exer-02/elapsed-s".into(),
                value: serde_json::json!(0),
            },
            PatchOperation::Set {
                path: "/demo/exer-02/elapsed-display".into(),
                value: serde_json::Value::String("0 s elapsed".into()),
            },
        ],
    })])
}

// ---------- handle_exer02_tick — per-tick patch emitter ----------
//
// Rotates three op kinds per 19-RESEARCH.md §Example 2 (lines 782-896):
//   iter % 3 == 0 → Set on /demo/exer-02/patch-sink/{iter} (data op)
//   iter % 3 == 1 → SetNode appending log-row-{iter} (node op)
//   iter % 3 == 2 → DeleteNode evicting a ghost row older than the ring cap
//
// Every tick also updates /demo/exer-02/elapsed-s + /demo/exer-02/elapsed-display.
// CRITICAL: no branch may emit an op whose path starts with
// /demo/exer-02/focused-value, or whose id == "exer-02-focused-input".
// This is the Pitfall 2 invariant — enforced by construction, verified at
// dev time by a debug_assert! loop, verified at test time by
// `tick_never_targets_focused_input_path` over 30 iterations.
pub async fn handle_exer02_tick(ctx: HandlerContext) -> ActionResult {
    let gs = crate::state::state();

    // Increment tick counter. Hold the lock just long enough to fetch + bump.
    let iter = {
        let mut t = gs.exer02_tick.lock().await;
        *t = t.wrapping_add(1);
        *t
    };

    let mut patch: Vec<PatchOperation> = Vec::with_capacity(3);
    let op_kind = iter % 3;
    let ts_ms = chrono::Utc::now().timestamp_millis();

    match op_kind {
        0 => {
            patch.push(PatchOperation::Set {
                path: format!("/demo/exer-02/patch-sink/{iter}"),
                value: serde_json::json!({"tick": iter, "ts_ms": ts_ms}),
            });
        }
        1 => {
            // Append a log-row. SetNode creates/replaces the component at the
            // derived id; the frontend's patch-apply path unions it into the
            // surface node map. A companion SetChildren op would append the
            // id into the log container's children array, but in this v1.2
            // scope we rely on the frontend's separate log-append wiring
            // (via the patch-probe) to manage the children list. The SetNode
            // alone is enough to exercise the Phase 12 pipeline and prove
            // the focus-retention invariant under node-op pressure.
            let row_id = format!("exer-02-log-row-{iter}");
            let formatted_ts = chrono::DateTime::<chrono::Utc>::from_timestamp_millis(ts_ms)
                .map_or_else(|| "-".into(), |dt| dt.format("%H:%M:%S%.3f").to_string());
            let row_comp = Component {
                r#type: "text".into(),
                props: Some(
                    serde_json::json!({"text": format!("[{formatted_ts}] patch {iter} applied")}),
                ),
                children: None,
                bind: None,
                action: None,
                visible: None,
            };
            patch.push(PatchOperation::SetNode {
                id: row_id,
                component: row_comp,
            });
        }
        _ => {
            // DeleteNode: attempt to evict a ghost row older than the cap.
            // Frontend applyPatch tolerates unknown ids — soft-fail is safe.
            let ghost = iter.saturating_sub(LOG_RING_BUFFER_MAX);
            if ghost > 0 {
                patch.push(PatchOperation::DeleteNode {
                    id: format!("exer-02-log-row-{ghost}"),
                });
            }
        }
    }

    // Always update elapsed-s + elapsed-display. Cheap (few json! constructions).
    let cadence = *gs.exer02_cadence_ms.lock().await;
    let elapsed_s = (iter * cadence) / 1000;
    patch.push(PatchOperation::Set {
        path: "/demo/exer-02/elapsed-s".into(),
        value: serde_json::json!(elapsed_s),
    });
    patch.push(PatchOperation::Set {
        path: "/demo/exer-02/elapsed-display".into(),
        value: serde_json::Value::String(format!("{elapsed_s} s elapsed")),
    });

    // Pitfall 2 runtime guard: assert no op targets the focused input. This
    // fires only in debug builds; release builds trust the construction
    // invariant. The unit test `tick_never_targets_focused_input_path`
    // exercises this in all build modes.
    for op in &patch {
        let bad_path = match op {
            PatchOperation::Set { path, .. } => path.starts_with("/demo/exer-02/focused-value"),
            PatchOperation::SetNode { id, .. }
            | PatchOperation::DeleteNode { id }
            | PatchOperation::SetChildren { id, .. } => id == "exer-02-focused-input",
            PatchOperation::InsertChild { child_id, .. }
            | PatchOperation::RemoveChild { child_id, .. } => child_id == "exer-02-focused-input",
        };
        debug_assert!(!bad_path, "Pitfall 2: tick emitted op targeting focused input");
    }

    Ok(vec![ProtocolMessage::Patch(PatchMessage {
        id: ctx.action.id.clone(),
        surface: "content".into(),
        patch,
    })])
}

// ---------- Tests ----------

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::Arc;

    use marionette::extractors::Session;
    use marionette_protocol::ActionMessage;
    use sea_orm::{DatabaseBackend, MockDatabase};

    fn mock_db() -> Arc<sea_orm::DatabaseConnection> {
        Arc::new(MockDatabase::new(DatabaseBackend::Sqlite).into_connection())
    }

    fn anonymous_session() -> Session {
        Session {
            user_id: None,
            roles: vec![],
        }
    }

    fn make_ctx(name: &str, payload: Option<serde_json::Value>) -> HandlerContext {
        HandlerContext {
            action: ActionMessage {
                id: Some("t1".into()),
                name: name.into(),
                source: None,
                payload,
                optimistic: None,
            },
            db: mock_db(),
            session: anonymous_session(),
        }
    }

    // ---- Helper: extract PatchMessage from ActionResult.

    fn patch_of(result: &[ProtocolMessage]) -> &PatchMessage {
        let ProtocolMessage::Patch(msg) = &result[0] else {
            panic!("expected Patch message, got {:?}", &result[0]);
        };
        msg
    }

    // ---- Tests 1-2: cadence clamp ----

    #[tokio::test]
    async fn start_accepts_valid_cadence_and_emits_running_true() {
        let ctx = make_ctx(
            "gallery-demo/exer-02/start",
            Some(serde_json::json!({ "cadence_ms": 500 })),
        );
        let out = handle_exer02_start(ctx).await.expect("ok");
        let msg = patch_of(&out);
        let running_op = msg
            .patch
            .iter()
            .find(|op| matches!(op, PatchOperation::Set { path, .. } if path == "/demo/exer-02/running"))
            .expect("running set op");
        if let PatchOperation::Set { value, .. } = running_op {
            assert_eq!(value, &serde_json::Value::Bool(true));
        }
    }

    #[tokio::test]
    async fn start_rejects_cadence_below_floor() {
        let ctx = make_ctx(
            "gallery-demo/exer-02/start",
            Some(serde_json::json!({ "cadence_ms": 50 })),
        );
        let result = handle_exer02_start(ctx).await;
        assert!(matches!(result, Err(ActionError::BadPayload(_))));
    }

    #[tokio::test]
    async fn start_rejects_cadence_above_ceiling() {
        let ctx = make_ctx(
            "gallery-demo/exer-02/start",
            Some(serde_json::json!({ "cadence_ms": 999_999 })),
        );
        let result = handle_exer02_start(ctx).await;
        assert!(matches!(result, Err(ActionError::BadPayload(_))));
    }

    #[tokio::test]
    async fn start_defaults_cadence_when_absent() {
        // No payload → StartPayload::default() via serde default = 500 ms,
        // which is in-range.
        let ctx = make_ctx("gallery-demo/exer-02/start", None);
        let out = handle_exer02_start(ctx).await.expect("ok");
        let msg = patch_of(&out);
        let cadence_op = msg
            .patch
            .iter()
            .find(|op| matches!(op, PatchOperation::Set { path, .. } if path == "/demo/exer-02/cadence-ms"))
            .expect("cadence-ms set op");
        if let PatchOperation::Set { value, .. } = cadence_op {
            assert_eq!(value, &serde_json::json!(500));
        }
    }

    // ---- Test 3: pause emits running=false ----

    #[tokio::test]
    async fn pause_emits_running_false() {
        let ctx = make_ctx("gallery-demo/exer-02/pause", None);
        let out = handle_exer02_pause(ctx).await.expect("ok");
        let msg = patch_of(&out);
        let running_op = msg
            .patch
            .iter()
            .find(|op| matches!(op, PatchOperation::Set { path, .. } if path == "/demo/exer-02/running"))
            .expect("running set op");
        if let PatchOperation::Set { value, .. } = running_op {
            assert_eq!(value, &serde_json::Value::Bool(false));
        }
    }

    // ---- Test 4: reset clears log and resets invariants ----

    #[tokio::test]
    async fn reset_clears_log_and_resets_four_invariants() {
        let ctx = make_ctx("gallery-demo/exer-02/reset", None);
        let out = handle_exer02_reset(ctx).await.expect("ok");
        let msg = patch_of(&out);

        // SetChildren with empty children on log container.
        assert!(
            msg.patch.iter().any(|op| matches!(op,
                PatchOperation::SetChildren { id, children }
                    if id == "exer-02-log-container" && children.is_empty()
            )),
            "reset must clear exer-02-log-container children"
        );

        // 4 invariant Set ops all PENDING.
        for inv in ["focus", "cursor", "typed", "ime"] {
            let path = format!("/demo/exer-02/invariants/{inv}");
            let op = msg
                .patch
                .iter()
                .find(|op| matches!(op, PatchOperation::Set { path: p, .. } if p == &path))
                .unwrap_or_else(|| panic!("missing reset op for {inv}"));
            if let PatchOperation::Set { value, .. } = op {
                assert_eq!(value["state"], "PENDING", "invariant {inv} must reset PENDING");
            }
        }

        // elapsed-s reset to 0.
        let elapsed_op = msg
            .patch
            .iter()
            .find(|op| matches!(op, PatchOperation::Set { path, .. } if path == "/demo/exer-02/elapsed-s"))
            .expect("elapsed-s reset op");
        if let PatchOperation::Set { value, .. } = elapsed_op {
            assert_eq!(value, &serde_json::json!(0));
        }
    }

    // ---- Test 5: Pitfall 2 regression guard ----

    #[tokio::test]
    async fn tick_never_targets_focused_input_path() {
        // Run 30 ticks and ensure NONE produce ops against focused-value.
        // This is the critical PATCH-02 invariant from 19-RESEARCH.md §Pitfall 2.
        for _ in 0..30 {
            let ctx = make_ctx("gallery-demo/exer-02/tick", None);
            let out = handle_exer02_tick(ctx).await.expect("ok");
            let msg = patch_of(&out);
            for op in &msg.patch {
                match op {
                    PatchOperation::Set { path, .. } => assert!(
                        !path.starts_with("/demo/exer-02/focused-value"),
                        "tick leaked path {path}"
                    ),
                    PatchOperation::SetNode { id, .. }
                    | PatchOperation::DeleteNode { id }
                    | PatchOperation::SetChildren { id, .. } => assert!(
                        id != "exer-02-focused-input",
                        "tick leaked node-op id {id}"
                    ),
                    PatchOperation::InsertChild { child_id, .. }
                    | PatchOperation::RemoveChild { child_id, .. } => assert!(
                        child_id != "exer-02-focused-input",
                        "tick leaked child-op id {child_id}"
                    ),
                }
            }
        }
    }

    // ---- Test 6: tick rotates op kinds ----

    #[tokio::test]
    async fn tick_rotates_three_op_kinds() {
        // Reset tick counter so the rotation pattern is deterministic for this
        // test. (Previous tests increment the shared tick counter via state()
        // singleton.) Running reset sets tick = 0; the first tick after
        // yields iter=1 → SetNode branch.
        let reset_ctx = make_ctx("gallery-demo/exer-02/reset", None);
        handle_exer02_reset(reset_ctx).await.expect("ok");

        let mut saw_set = false;
        let mut saw_setnode = false;
        // DeleteNode branch requires iter > LOG_RING_BUFFER_MAX which we won't
        // reach in 10 ticks; skip asserting its presence here.
        for _ in 0..10 {
            let ctx = make_ctx("gallery-demo/exer-02/tick", None);
            let out = handle_exer02_tick(ctx).await.expect("ok");
            let msg = patch_of(&out);
            for op in &msg.patch {
                match op {
                    PatchOperation::Set { path, .. }
                        if path.starts_with("/demo/exer-02/patch-sink") =>
                    {
                        saw_set = true;
                    }
                    PatchOperation::SetNode { id, .. } if id.starts_with("exer-02-log-row-") => {
                        saw_setnode = true;
                    }
                    _ => {}
                }
            }
        }
        assert!(saw_set, "expected at least one patch-sink Set op");
        assert!(saw_setnode, "expected at least one log-row SetNode op");
    }

    // ---- Test 7: tick always updates elapsed-display ----

    #[tokio::test]
    async fn tick_always_updates_elapsed_display() {
        let reset_ctx = make_ctx("gallery-demo/exer-02/reset", None);
        handle_exer02_reset(reset_ctx).await.expect("ok");

        let ctx = make_ctx("gallery-demo/exer-02/tick", None);
        let out = handle_exer02_tick(ctx).await.expect("ok");
        let msg = patch_of(&out);
        let has_elapsed_display = msg.patch.iter().any(|op| {
            matches!(op, PatchOperation::Set { path, .. } if path == "/demo/exer-02/elapsed-display")
        });
        assert!(
            has_elapsed_display,
            "every tick must emit /demo/exer-02/elapsed-display"
        );
    }
}
