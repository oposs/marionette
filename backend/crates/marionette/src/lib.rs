#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod auth;
pub mod builders;
pub mod error;
pub mod extractors;
pub mod router;
pub mod session;
pub mod ws;

pub use error::*;
pub use extractors::*;
pub use marionette_macros::*;
pub use marionette_protocol as protocol;
pub use router::ActionRouter;
pub use session::WsSession;
pub use ws::{ws_handler, AppState};
