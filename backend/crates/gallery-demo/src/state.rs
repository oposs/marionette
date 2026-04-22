//! In-memory gallery state — `Arc<RwLock<_>>` per CONTEXT.md §CRATE-01.
//!
//! State survives for the lifetime of the process and resets on restart —
//! "restart is reset" is a product feature of the gallery (no persistence
//! per §Out of Scope). Handlers clone `Arc<RwLock<_>>` into closures at
//! registration time.

#![allow(dead_code)] // fields are read by handlers registered in main.rs

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Gallery-demo process-wide mutable state.
///
/// All fields are `Arc<RwLock<_>>` so handler closures can clone once at
/// ActionRouter registration time and pass ownership into `box_handler`'s
/// `Future`. Reads take `.read().await`; writes take `.write().await`.
#[derive(Clone, Default)]
pub struct GalleryState {
    /// Per-demo bind-path state (`/demo/{key}/...`). Keyed by the full
    /// bind path; values are arbitrary JSON. Seeded lazily by `handle_gallery_show`.
    pub demo_values: Arc<RwLock<HashMap<String, serde_json::Value>>>,
    /// Modal-demo open flag. Flipped by handle_modal_open / handle_modal_close.
    pub modal_open: Arc<RwLock<bool>>,
    /// ConfirmDialog-demo open flag + last decision. Flipped by the three confirm handlers.
    pub confirm_open: Arc<RwLock<bool>>,
}
