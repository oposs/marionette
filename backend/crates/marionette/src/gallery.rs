//! Gallery demo registry — `DemoEntry`, the `linkme`-backed `DEMOS` distributed slice,
//! and the `registered_demos()` iteration API.
//!
//! Registration is performed by the `#[gallery_demo]` attribute macro in
//! `marionette-macros`, which emits a `static` of type [`DemoEntry`] into the
//! [`DEMOS`] distributed slice. The `gallery` cargo feature gates the entire
//! registry: under default build no demo symbols exist (FRAME-03); under
//! `--features gallery` the slice is populated and iterable.
//!
//! Stable iteration order is owned by this module — `linkme`'s native link-time
//! order is not portable across platforms, so [`registered_demos`] sorts the
//! slice alphabetically by [`DemoEntry::key`] on first call and memoizes the
//! result.

use std::sync::OnceLock;

pub use crate::builders::Node;

/// Registry entry for a gallery demo. The `#[gallery_demo]` attribute macro
/// emits a `static` of this type that `linkme` collects into [`DEMOS`].
///
/// `registered_demos()` yields `&'static DemoEntry` references in alphabetical
/// order by [`DemoEntry::key`].
#[derive(Debug)]
pub struct DemoEntry {
    /// Stable identifier used as the sort key. Derived from the annotated fn
    /// ident by default; overridable via `#[gallery_demo(key = "...")]`.
    pub key: &'static str,
    /// Entry-point fn. Takes no arguments; returns a flat `Vec<Node>` where
    /// index 0 is the root and remaining elements are descendants. Per the
    /// pure-fn contract (Phase 17's `GALLERY-DEMOS.md`), implementations MUST
    /// NOT perform I/O or touch external state.
    pub render: fn() -> Vec<Node>,
    /// Navigation-facing label. Title-cased from `key` by default; overridable
    /// via `#[gallery_demo(name = "...")]`.
    pub display_name: &'static str,
}

/// The distributed slice. Populated at link time by `#[gallery_demo]`-emitted
/// `static`s. Only exists under `--features gallery`.
#[cfg(feature = "gallery")]
#[linkme::distributed_slice]
pub static DEMOS: [DemoEntry] = [..];

/// Re-export of the `linkme` crate for the `#[gallery_demo]` macro's emitted
/// tokens. Consumers don't need their own `linkme` dependency — the macro
/// emits `#[::marionette::gallery::__linkme::distributed_slice(...)]`.
#[cfg(feature = "gallery")]
#[doc(hidden)]
pub use ::linkme as __linkme;

static SORTED_CACHE: OnceLock<Vec<&'static DemoEntry>> = OnceLock::new();

/// Pure function extracted for unit testability. Sorts by `key` alphabetically,
/// then dedups with keep-first semantics. On duplicate, `debug_assert!`s in
/// debug builds and `tracing::warn!`s in all builds.
#[cfg_attr(not(any(feature = "gallery", test)), allow(dead_code))]
fn sort_entries(entries: &[&'static DemoEntry]) -> Vec<&'static DemoEntry> {
    let mut v: Vec<&'static DemoEntry> = entries.to_vec();
    v.sort_by_key(|e| e.key);
    for pair in v.windows(2) {
        if pair[0].key == pair[1].key {
            debug_assert!(
                pair[0].key != pair[1].key,
                "duplicate #[gallery_demo] key = {:?} (display_names: {:?}, {:?})",
                pair[0].key,
                pair[0].display_name,
                pair[1].display_name,
            );
            tracing::warn!(
                key = pair[0].key,
                first_display_name = pair[0].display_name,
                second_display_name = pair[1].display_name,
                "duplicate #[gallery_demo] key — keeping first, discarding later entries",
            );
        }
    }
    v.dedup_by(|a, b| a.key == b.key);
    v
}

#[cfg(feature = "gallery")]
fn build_sorted() -> Vec<&'static DemoEntry> {
    let refs: Vec<&'static DemoEntry> = DEMOS.iter().collect();
    sort_entries(&refs)
}

#[cfg(not(feature = "gallery"))]
fn build_sorted() -> Vec<&'static DemoEntry> {
    Vec::new()
}

/// Iterate all demos registered via `#[gallery_demo]`, in stable alphabetical
/// order by [`DemoEntry::key`]. Under default build (no `gallery` feature),
/// yields an empty iterator.
///
/// First call performs a one-time O(n log n) sort + dedup and memoizes via
/// [`std::sync::OnceLock`]. Subsequent calls iterate the cached `Vec`.
///
/// Duplicate `key` values are detected during the one-time sort pass:
/// `debug_assert!` panics in debug builds with both sites named; release
/// builds log `tracing::warn!` and keep the first-registered entry.
pub fn registered_demos() -> impl Iterator<Item = &'static DemoEntry> {
    SORTED_CACHE.get_or_init(build_sorted).iter().copied()
}

#[cfg(test)]
mod tests {
    use super::{sort_entries, DemoEntry, Node};
    use marionette_protocol::Component;

    /// Harmless Node value for test stubs. `render` is `fn() -> Vec<Node>`
    /// per D-Z1; the tests inspect only `key` / `display_name`, never the
    /// output of `render`. The literal matches the actual `Component` struct
    /// (see marionette-protocol/src/component.rs lines 1-29).
    fn minimal_nodes() -> Vec<Node> {
        vec![(
            String::new(),
            Component {
                r#type: "text".into(),
                props: None,
                children: None,
                bind: None,
                action: None,
                visible: None,
            },
        )]
    }

    fn leak_entry(key: &'static str, display_name: &'static str) -> &'static DemoEntry {
        Box::leak(Box::new(DemoEntry { key, render: minimal_nodes, display_name }))
    }

    #[test]
    fn sort_entries_yields_alphabetical_order() {
        let a = leak_entry("zebra", "Zebra");
        let b = leak_entry("apple", "Apple");
        let c = leak_entry("mango", "Mango");
        let sorted = sort_entries(&[a, b, c]);
        let keys: Vec<&str> = sorted.iter().map(|e| e.key).collect();
        assert_eq!(keys, vec!["apple", "mango", "zebra"]);
    }

    #[test]
    fn sort_entries_empty_input_yields_empty() {
        let sorted = sort_entries(&[]);
        assert!(sorted.is_empty());
    }

    #[test]
    #[should_panic(expected = "duplicate #[gallery_demo] key")]
    #[cfg(debug_assertions)]
    fn sort_entries_duplicate_panics_in_debug() {
        let a = leak_entry("dup", "First");
        let b = leak_entry("dup", "Second");
        let _ = sort_entries(&[a, b]);
    }

    #[test]
    #[cfg(not(debug_assertions))]
    fn sort_entries_dedups_keep_first_in_release() {
        let a = leak_entry("dup", "First");
        let b = leak_entry("dup", "Second");
        let sorted = sort_entries(&[a, b]);
        assert_eq!(sorted.len(), 1);
        assert_eq!(sorted[0].display_name, "First");
    }

    #[test]
    fn registered_demos_is_idempotent() {
        let first: Vec<&'static str> = super::registered_demos().map(|e| e.key).collect();
        let second: Vec<&'static str> = super::registered_demos().map(|e| e.key).collect();
        assert_eq!(first, second);
    }
}
