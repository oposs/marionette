//! Catalog screens — curated showcases that compose fresh from Marionette builders.
//!
//! Per CONTEXT.md §D-2-B, each file inside `catalog/` hosts its own
//! `#[gallery_demo]` fn; auto-discovery happens via the linkme
//! `DEMOS` distributed slice populated at link time.
//!
//! Glob re-exports are deliberately omitted because every file has a
//! `gallery_demo` fn — re-exporting via `pub use` would cause ambiguity.
//! Callers access these fns via the registry, not by path.
//!
//! See GALLERY-DEMOS.md §Catalog-Screens and CONTEXT.md §D-2-D for the
//! file layout contract. Sibling catalog modules (forms, data_table,
//! feedback, typography) are added in plans 18-05..18-08.

pub mod buttons;
pub mod forms;
