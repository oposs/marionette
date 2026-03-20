#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod auth;
pub mod builders;
pub mod db;
pub mod error;
pub mod extractors;
pub mod migration;
pub mod router;
pub mod session;
pub mod ws;

pub use db::{init_db, session as db_session, test_db};
pub use error::*;
pub use extractors::*;
pub use marionette_macros::*;
pub use marionette_protocol as protocol;
pub use router::ActionRouter;
pub use session::WsSession;
pub use ws::{ws_handler, AppState};
