//! EXER-02 rapid-patching handlers.
//!
//! Plan 19-01 ships stubs returning Ok(vec![]). Plan 19-03 fills in the
//! real handlers: handle_exer02_start spawns a tokio task that every
//! cadence_ms emits one PatchMessage via handle_exer02_tick (client-initiated
//! tick resolution per 19-RESEARCH.md §Assumption A1 — the frontend
//! calls tick every cadence_ms; backend responds with the per-tick patch).
//! handle_exer02_pause/reset cancel the task + emit reset patches.
//!
//! The stubs are kept in a single file (not split per handler) so Plan 19-03
//! can grow the module naturally — they all share state() singleton access.

use marionette::error::ActionResult;
use marionette::extractors::HandlerContext;

#[allow(clippy::unused_async)]
pub async fn handle_exer02_start(_ctx: HandlerContext) -> ActionResult {
    Ok(vec![])
}

#[allow(clippy::unused_async)]
pub async fn handle_exer02_pause(_ctx: HandlerContext) -> ActionResult {
    Ok(vec![])
}

#[allow(clippy::unused_async)]
pub async fn handle_exer02_reset(_ctx: HandlerContext) -> ActionResult {
    Ok(vec![])
}

#[allow(clippy::unused_async)]
pub async fn handle_exer02_tick(_ctx: HandlerContext) -> ActionResult {
    // Plan 19-01 -> 19-03 handoff: Plan 19-01 router_tests verify this stub
    // is reachable via the registered router and returns Ok(vec![]).
    Ok(vec![])
}
