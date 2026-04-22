#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

// Allow `::marionette::…` absolute paths emitted by the `#[gallery_demo]`
// proc-macro to resolve inside this crate. The macro emits
// `::marionette::gallery::__linkme::…` which is correct for external consumers
// (gallery-smoke, future downstream) but requires a self-alias to resolve
// from within the marionette crate itself (Phase 17 Plan 04 DEMO-01).
extern crate self as marionette;

pub mod auth;
pub mod builders;
pub mod db;
pub mod error;
pub mod extractors;
pub mod gallery;
pub mod migration;
pub mod router;
pub mod session;
pub mod validation;
pub mod ws;

pub use db::{init_db, session as db_session, test_db};
pub use error::*;
pub use extractors::*;
pub use marionette_macros::*;
pub use marionette_protocol as protocol;
pub use router::ActionRouter;
pub use session::WsSession;
pub use ws::{ws_handler, AppState};
