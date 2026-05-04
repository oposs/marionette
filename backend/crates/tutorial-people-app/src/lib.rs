//! Tutorial: People App — a minimal Marionette application.
//!
//! Exposes [`handlers`], [`state`], and [`ui`] as a library so integration
//! tests under `tests/` can exercise the same registration code the binary
//! uses, without bringing up an HTTP server.
//!
//! Walkthrough lives in `docs/AUTHORING.md`.

pub mod handlers;
pub mod state;
pub mod ui;
