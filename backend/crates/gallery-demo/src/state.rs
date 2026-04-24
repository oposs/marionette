//! In-memory gallery state — `Arc<RwLock<_>>` per CONTEXT.md §CRATE-01.
//!
//! State survives for the lifetime of the process and resets on restart —
//! "restart is reset" is a product feature of the gallery (no persistence
//! per §Out of Scope). Handlers clone `Arc<RwLock<_>>` into closures at
//! registration time.
//!
//! Phase 19 extension: EXER-02 rapid-patching needs a tokio JoinHandle
//! slot plus cadence + tick counters. These are exposed via a once-cell
//! singleton (`state()`) so exer02 handlers can read/write without
//! extending `marionette::ws::AppState` (Option B in 19-PATTERNS.md
//! §AppState integration gap — dev-local harness, no framework crate change).

#![allow(dead_code)] // fields are read by handlers registered in main.rs

use std::collections::HashMap;
use std::sync::{Arc, OnceLock};
use tokio::sync::{Mutex, RwLock};
use tokio::task::JoinHandle;

/// Gallery-demo process-wide mutable state.
///
/// All fields are `Arc<RwLock<_>>` or `Arc<Mutex<_>>` so handler closures can
/// clone once at ActionRouter registration time and pass ownership into
/// `box_handler`'s `Future`.
#[derive(Clone)]
pub struct GalleryState {
    /// Per-demo bind-path state (`/demo/{key}/...`). Keyed by the full
    /// bind path; values are arbitrary JSON. Seeded lazily by `handle_gallery_show`.
    pub demo_values: Arc<RwLock<HashMap<String, serde_json::Value>>>,
    /// Modal-demo open flag. Flipped by handle_modal_open / handle_modal_close.
    pub modal_open: Arc<RwLock<bool>>,
    /// ConfirmDialog-demo open flag + last decision. Flipped by the three confirm handlers.
    pub confirm_open: Arc<RwLock<bool>>,

    // --- Phase 19 EXER-02 (Plan 19-03) ---
    /// Active patch-loop task handle. `Some` while loop is running; `None`
    /// when paused/stopped. Pause/Reset MUST abort before storing None
    /// (19-RESEARCH.md §Pitfall 9) — use `Option::take()` then `.abort()`.
    pub exer02_loop: Arc<Mutex<Option<JoinHandle<()>>>>,
    /// Current cadence in ms — mirrors /demo/exer-02/cadence-ms
    /// (frontend writes, backend reads on each start).
    pub exer02_cadence_ms: Arc<Mutex<u64>>,
    /// Monotonic tick counter for patch iteration ids.
    pub exer02_tick: Arc<Mutex<u64>>,
}

impl Default for GalleryState {
    fn default() -> Self {
        Self {
            demo_values: Arc::new(RwLock::new(HashMap::new())),
            modal_open: Arc::new(RwLock::new(false)),
            confirm_open: Arc::new(RwLock::new(false)),
            exer02_loop: Arc::new(Mutex::new(None)),
            exer02_cadence_ms: Arc::new(Mutex::new(500)),
            exer02_tick: Arc::new(Mutex::new(0)),
        }
    }
}

/// Crate-local singleton. Handlers that need process-wide gallery state
/// (currently: EXER-02 start/pause/reset/tick) read from this instead of
/// receiving state through `HandlerContext` — because `marionette::ws::AppState`
/// does not carry a gallery slot (see 19-PATTERNS.md §AppState integration gap).
///
/// Returned reference is 'static — safe to clone the inner `Arc<_>` fields.
#[must_use]
pub fn state() -> &'static GalleryState {
    static INSTANCE: OnceLock<GalleryState> = OnceLock::new();
    INSTANCE.get_or_init(GalleryState::default)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_state_has_phase_19_defaults() {
        let s = GalleryState::default();
        // Tokio Mutex can only be awaited in async contexts, so construct a
        // blocking runtime for the test.
        let rt = tokio::runtime::Runtime::new().expect("rt");
        rt.block_on(async {
            assert!(
                s.exer02_loop.lock().await.is_none(),
                "loop handle starts None"
            );
            assert_eq!(
                *s.exer02_cadence_ms.lock().await,
                500,
                "default cadence 500ms"
            );
            assert_eq!(*s.exer02_tick.lock().await, 0, "tick starts at 0");
        });
    }

    #[test]
    fn state_singleton_returns_identical_reference() {
        let a = state();
        let b = state();
        assert!(
            std::ptr::eq(a, b),
            "state() must return the same &'static reference"
        );
    }
}
