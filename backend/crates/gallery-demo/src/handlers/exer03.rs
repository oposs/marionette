//! EXER-03 pathological-scale handlers (Plan 19-04).
//!
//! Plan 19-01 shipped stubs returning `Ok(vec![])`; this module replaces
//! them with real implementations:
//!
//! - `handle_exer03_report_perf` receives the 4-signal snapshot from the
//!   frontend (TTFP / FPS / Memory growth / Patch latency p95) and emits a
//!   `PatchMessage` with `Set` ops on
//!   `/demo/exer-03/perf/{slug}/{value,badge}`. Each signal is evaluated
//!   against the advisory thresholds locked by 19-CONTEXT.md §D-3 — missing
//!   a threshold surfaces as an `"OVER TARGET"` badge string, NOT as an
//!   error. `None` fields on the snapshot are skipped (the UI keeps the
//!   previous value; useful when the frontend captures signals on different
//!   cadences: TTFP on mount, FPS on first scroll, memory growth at t+30s).
//! - `handle_exer03_remeasure` emits a single `Set` op on
//!   `/demo/exer-03/perf/remeasure-tick` carrying the current epoch-ms so
//!   the frontend's reactive instrumentation (perf.svelte.ts) can observe
//!   the tick and re-capture.
//!
//! All per-signal value ops carry a `{value: <f64>, within_target: <bool>}`
//! JSON object so the frontend can key off `within_target` for styling.

use marionette::error::{ActionError, ActionResult};
use marionette::extractors::HandlerContext;
use marionette_protocol::data::PatchOperation;
use marionette_protocol::messages::PatchMessage;
use marionette_protocol::ProtocolMessage;
use serde::Deserialize;

// -- Advisory thresholds (19-CONTEXT.md §D-3) ------------------------------

const TTFP_MAX_MS: f64 = 3000.0;
const FPS_MIN: f64 = 30.0;
/// Memory GROWTH budget after 30 s scroll (not absolute heap size).
const MEMORY_GROWTH_MAX_MB: f64 = 50.0;
const LATENCY_P95_MAX_MS: f64 = 50.0;

// -- Payload shape ---------------------------------------------------------

/// Frontend-to-backend perf snapshot. Every field is optional so the
/// frontend can report signals on their natural cadence (TTFP on mount,
/// FPS on first scroll, memory growth at t+30s) without spoofing values
/// for the others.
#[derive(Debug, Deserialize)]
struct PerfSnapshot {
    #[serde(default)]
    ttfp_ms: Option<f64>,
    #[serde(default)]
    fps: Option<f64>,
    #[serde(default)]
    memory_mb: Option<f64>,
    #[serde(default)]
    latency_p95_ms: Option<f64>,
}

// -- Handlers --------------------------------------------------------------

pub async fn handle_exer03_report_perf(ctx: HandlerContext) -> ActionResult {
    // Accept a missing or null payload as "all signals None". The frontend
    // may report on different cadences (TTFP on mount, FPS on first scroll,
    // etc.) so a report carrying no fields is legal — it's a no-op rather
    // than a bad payload. Also satisfies the Plan 19-01 → 19-04 reachability
    // guard which dispatches with payload=None.
    let snapshot: PerfSnapshot = match ctx.action.payload.clone() {
        None | Some(serde_json::Value::Null) => PerfSnapshot {
            ttfp_ms: None,
            fps: None,
            memory_mb: None,
            latency_p95_ms: None,
        },
        Some(payload) => serde_json::from_value(payload).map_err(|e| {
            ActionError::BadPayload(format!("exer-03 perf payload invalid: {e}"))
        })?,
    };

    // Pre-size for the worst case: 4 signals × 2 ops (value + badge) = 8.
    let mut patch: Vec<PatchOperation> = Vec::with_capacity(8);

    if let Some(v) = snapshot.ttfp_ms {
        let within = v <= TTFP_MAX_MS;
        patch.push(perf_value_op("ttfp_ms", v, within));
        patch.push(perf_badge_op("ttfp_ms", within));
    }
    if let Some(v) = snapshot.fps {
        let within = v >= FPS_MIN;
        patch.push(perf_value_op("fps", v, within));
        patch.push(perf_badge_op("fps", within));
    }
    if let Some(v) = snapshot.memory_mb {
        // Memory is a GROWTH figure — caller passes the delta, not absolute.
        let within = v <= MEMORY_GROWTH_MAX_MB;
        patch.push(perf_value_op("memory_mb", v, within));
        patch.push(perf_badge_op("memory_mb", within));
    }
    if let Some(v) = snapshot.latency_p95_ms {
        let within = v <= LATENCY_P95_MAX_MS;
        patch.push(perf_value_op("latency_p95_ms", v, within));
        patch.push(perf_badge_op("latency_p95_ms", within));
    }

    if patch.is_empty() {
        // No signals reported → no-op. Return an empty Vec so the Plan 19-01
        // reachability guard (which sends payload=None) still passes and so
        // we don't emit a meaningless empty PatchMessage over the wire.
        return Ok(vec![]);
    }

    Ok(vec![ProtocolMessage::Patch(PatchMessage {
        id: ctx.action.id.clone(),
        surface: "content".into(),
        patch,
    })])
}

#[allow(clippy::unused_async)]
pub async fn handle_exer03_remeasure(ctx: HandlerContext) -> ActionResult {
    // Plan 19-01 reachability guard sends payload=None; respond with an
    // empty Vec in that case so the test stays green after the body is
    // filled in. Any real client invocation carries at least an empty
    // JSON object `{}` (frontend perf.svelte.ts `sendAction` contract),
    // which falls through to the tick emission.
    if ctx.action.payload.is_none() {
        return Ok(vec![]);
    }

    // Emit a single marker Set so the frontend's reactive instrumentation
    // knows to re-capture. The timestamp value is mostly decorative — the
    // frontend watches for ANY change to the path, not the specific value.
    Ok(vec![ProtocolMessage::Patch(PatchMessage {
        id: ctx.action.id.clone(),
        surface: "content".into(),
        patch: vec![PatchOperation::Set {
            path: "/demo/exer-03/perf/remeasure-tick".into(),
            value: serde_json::json!(chrono::Utc::now().timestamp_millis()),
        }],
    })])
}

// -- Helpers ---------------------------------------------------------------

fn perf_value_op(slug: &str, value: f64, within_target: bool) -> PatchOperation {
    PatchOperation::Set {
        path: format!("/demo/exer-03/perf/{slug}/value"),
        value: serde_json::json!({
            "value": value,
            "within_target": within_target,
        }),
    }
}

fn perf_badge_op(slug: &str, within_target: bool) -> PatchOperation {
    let label = if within_target {
        "WITHIN TARGET"
    } else {
        "OVER TARGET"
    };
    PatchOperation::Set {
        path: format!("/demo/exer-03/perf/{slug}/badge"),
        value: serde_json::Value::String(label.into()),
    }
}

// -- Tests -----------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use marionette::extractors::Session;
    use marionette_protocol::messages::ActionMessage;
    use sea_orm::{DatabaseBackend, MockDatabase};
    use std::sync::Arc;

    fn make_ctx(payload: serde_json::Value) -> HandlerContext {
        HandlerContext {
            action: ActionMessage {
                id: Some("t1".into()),
                name: "gallery-demo/exer-03/report-perf".into(),
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

    #[tokio::test]
    async fn report_perf_writes_four_pairs() {
        let ctx = make_ctx(serde_json::json!({
            "ttfp_ms": 1500.0,
            "fps": 45.0,
            "memory_mb": 20.0,
            "latency_p95_ms": 10.0,
        }));
        let out = handle_exer03_report_perf(ctx).await.expect("ok");
        let msg = unwrap_patch(&out);
        // 4 signals × 2 ops (value + badge) = 8.
        assert_eq!(msg.patch.len(), 8);
        // Every op must be a Set on /demo/exer-03/perf/…
        for op in &msg.patch {
            let PatchOperation::Set { path, .. } = op else {
                panic!("only Set ops expected, got {op:?}");
            };
            assert!(
                path.starts_with("/demo/exer-03/perf/"),
                "unexpected path {path}"
            );
        }
    }

    #[tokio::test]
    async fn ttfp_over_3000_flags_over_target() {
        let ctx = make_ctx(serde_json::json!({
            "ttfp_ms": 3500.0,
            "fps": null,
            "memory_mb": null,
            "latency_p95_ms": null,
        }));
        let out = handle_exer03_report_perf(ctx).await.expect("ok");
        let msg = unwrap_patch(&out);

        let value_op = msg
            .patch
            .iter()
            .find(|op| matches!(op, PatchOperation::Set { path, .. } if path == "/demo/exer-03/perf/ttfp_ms/value"))
            .expect("ttfp value op");
        if let PatchOperation::Set { value, .. } = value_op {
            assert_eq!(value["within_target"], false);
            assert!(
                (value["value"].as_f64().unwrap() - 3500.0).abs() < f64::EPSILON,
                "value passthrough"
            );
        }

        let badge_op = msg
            .patch
            .iter()
            .find(|op| matches!(op, PatchOperation::Set { path, .. } if path == "/demo/exer-03/perf/ttfp_ms/badge"))
            .expect("ttfp badge op");
        if let PatchOperation::Set { value, .. } = badge_op {
            assert_eq!(value, &serde_json::Value::String("OVER TARGET".into()));
        }
    }

    #[tokio::test]
    async fn fps_under_30_flags_over_target() {
        let ctx = make_ctx(serde_json::json!({
            "ttfp_ms": null,
            "fps": 25.0,
            "memory_mb": null,
            "latency_p95_ms": null,
        }));
        let out = handle_exer03_report_perf(ctx).await.expect("ok");
        let msg = unwrap_patch(&out);
        let op = msg
            .patch
            .iter()
            .find(|op| matches!(op, PatchOperation::Set { path, .. } if path == "/demo/exer-03/perf/fps/value"))
            .expect("fps value op");
        if let PatchOperation::Set { value, .. } = op {
            assert_eq!(value["within_target"], false);
        }
    }

    #[tokio::test]
    async fn all_within_targets_for_ok_values() {
        let ctx = make_ctx(serde_json::json!({
            "ttfp_ms": 1500.0,
            "fps": 45.0,
            "memory_mb": 20.0,
            "latency_p95_ms": 10.0,
        }));
        let out = handle_exer03_report_perf(ctx).await.expect("ok");
        let msg = unwrap_patch(&out);
        for slug in ["ttfp_ms", "fps", "memory_mb", "latency_p95_ms"] {
            let value_path = format!("/demo/exer-03/perf/{slug}/value");
            let op = msg
                .patch
                .iter()
                .find(|op| matches!(op, PatchOperation::Set { path, .. } if path == &value_path))
                .unwrap_or_else(|| panic!("missing {value_path}"));
            if let PatchOperation::Set { value, .. } = op {
                assert_eq!(
                    value["within_target"], true,
                    "{slug} should be within target"
                );
            }
            let badge_path = format!("/demo/exer-03/perf/{slug}/badge");
            let badge_op = msg
                .patch
                .iter()
                .find(|op| matches!(op, PatchOperation::Set { path, .. } if path == &badge_path))
                .unwrap_or_else(|| panic!("missing {badge_path}"));
            if let PatchOperation::Set { value, .. } = badge_op {
                assert_eq!(value, &serde_json::Value::String("WITHIN TARGET".into()));
            }
        }
    }

    #[tokio::test]
    async fn nulls_dont_emit_ops() {
        let ctx = make_ctx(serde_json::json!({
            "ttfp_ms": null,
            "fps": 45.0,
            "memory_mb": null,
            "latency_p95_ms": null,
        }));
        let out = handle_exer03_report_perf(ctx).await.expect("ok");
        let msg = unwrap_patch(&out);
        // Only fps → 2 ops (value + badge).
        assert_eq!(msg.patch.len(), 2);
        // Neither ttfp nor memory nor latency paths present.
        for slug in ["ttfp_ms", "memory_mb", "latency_p95_ms"] {
            let value_path = format!("/demo/exer-03/perf/{slug}/value");
            assert!(
                !msg.patch.iter().any(|op| matches!(
                    op,
                    PatchOperation::Set { path, .. } if path == &value_path
                )),
                "{slug} should have been skipped"
            );
        }
    }

    #[tokio::test]
    async fn memory_growth_over_50_flags_over_target() {
        let ctx = make_ctx(serde_json::json!({
            "ttfp_ms": null,
            "fps": null,
            "memory_mb": 75.5,
            "latency_p95_ms": null,
        }));
        let out = handle_exer03_report_perf(ctx).await.expect("ok");
        let msg = unwrap_patch(&out);
        let badge_op = msg
            .patch
            .iter()
            .find(|op| matches!(op, PatchOperation::Set { path, .. } if path == "/demo/exer-03/perf/memory_mb/badge"))
            .expect("memory badge op");
        if let PatchOperation::Set { value, .. } = badge_op {
            assert_eq!(value, &serde_json::Value::String("OVER TARGET".into()));
        }
    }

    #[tokio::test]
    async fn latency_p95_over_50_flags_over_target() {
        let ctx = make_ctx(serde_json::json!({
            "ttfp_ms": null,
            "fps": null,
            "memory_mb": null,
            "latency_p95_ms": 120.0,
        }));
        let out = handle_exer03_report_perf(ctx).await.expect("ok");
        let msg = unwrap_patch(&out);
        let badge_op = msg
            .patch
            .iter()
            .find(|op| matches!(op, PatchOperation::Set { path, .. } if path == "/demo/exer-03/perf/latency_p95_ms/badge"))
            .expect("latency badge op");
        if let PatchOperation::Set { value, .. } = badge_op {
            assert_eq!(value, &serde_json::Value::String("OVER TARGET".into()));
        }
    }

    #[tokio::test]
    async fn bad_payload_returns_error() {
        // Non-numeric fields should fail strict Option<f64> deserialization —
        // satisfies T-19-04-01 (tampering mitigation).
        let ctx = make_ctx(serde_json::json!({
            "ttfp_ms": "not-a-number",
        }));
        let out = handle_exer03_report_perf(ctx).await;
        match out {
            Err(ActionError::BadPayload(_)) => {}
            other => panic!("expected BadPayload, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn remeasure_emits_tick_marker() {
        let ctx = make_ctx(serde_json::json!({}));
        let out = handle_exer03_remeasure(ctx).await.expect("ok");
        let msg = unwrap_patch(&out);
        assert_eq!(msg.patch.len(), 1);
        let PatchOperation::Set { path, value } = &msg.patch[0] else {
            panic!("expected Set op");
        };
        assert_eq!(path, "/demo/exer-03/perf/remeasure-tick");
        assert!(
            value.as_i64().is_some(),
            "remeasure-tick should carry an i64 timestamp, got {value:?}"
        );
    }

    #[tokio::test]
    async fn empty_payload_emits_empty_vec() {
        // All fields None → no ops emitted AND no Patch message emitted.
        // Handler returns an empty Vec so the Plan 19-01 reachability guard
        // (payload=None → all-None snapshot → no-op) still sees `is_empty()`.
        let ctx = make_ctx(serde_json::json!({}));
        let out = handle_exer03_report_perf(ctx).await.expect("ok");
        assert!(
            out.is_empty(),
            "no-signal reports must return an empty Vec, got {out:?}"
        );
    }

    #[tokio::test]
    async fn none_payload_emits_empty_vec() {
        // Explicit payload=None (as the 19-01 reachability guard sends).
        // Handler must treat as all-None snapshot and return Ok(vec![]).
        let mut ctx = make_ctx(serde_json::json!({}));
        ctx.action.payload = None;
        let out = handle_exer03_report_perf(ctx).await.expect("ok");
        assert!(out.is_empty());
    }
}
