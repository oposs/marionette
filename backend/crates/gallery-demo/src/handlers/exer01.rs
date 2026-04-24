//! EXER-01 observation-report handler.
//!
//! Plan 19-01 ships a stub returning Ok(vec![]). Plan 19-02 fills in the
//! real handler: deserialize an ObservationReport payload and emit a
//! PatchMessage with 4 Set ops on /demo/exer-01/matrix/{dimension}.

use marionette::error::ActionResult;
use marionette::extractors::HandlerContext;

#[allow(clippy::unused_async)]
pub async fn handle_exer01_report(_ctx: HandlerContext) -> ActionResult {
    // Stub — Plan 19-02 replaces this body with the observation-matrix update.
    Ok(vec![])
}
