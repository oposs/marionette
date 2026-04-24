//! EXER-03 pathological-scale handlers.
//!
//! Plan 19-01 ships stubs returning Ok(vec![]). Plan 19-04 fills in the
//! real handlers: handle_exer03_report_perf writes 4 Set ops on
//! /demo/exer-03/perf/{signal} with within_target flags; handle_exer03_remeasure
//! emits a marker Set op so the frontend's instrumentation knows to re-capture.

use marionette::error::ActionResult;
use marionette::extractors::HandlerContext;

#[allow(clippy::unused_async)]
pub async fn handle_exer03_report_perf(_ctx: HandlerContext) -> ActionResult {
    Ok(vec![])
}

#[allow(clippy::unused_async)]
pub async fn handle_exer03_remeasure(_ctx: HandlerContext) -> ActionResult {
    Ok(vec![])
}
