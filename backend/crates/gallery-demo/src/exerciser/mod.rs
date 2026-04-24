//! Exerciser screens — robustness stress-tests composed with the same
//! builders as catalog screens (see 19-PATTERNS.md §`src/exerciser/mod.rs`).
//!
//! Per 19-CONTEXT.md §D-1..D-4, each file inside `exerciser/` hosts its own
//! `#[gallery_demo]` fn; auto-discovery happens via the linkme `DEMOS`
//! distributed slice populated at link time (Phase 16 contract).
//!
//! See 19-REQUIREMENTS.md §EXER-01..03 for scope. Sibling catalog modules
//! (buttons, forms, data_table, feedback, typography) predate this.
//!
//! Plan 19-01 ships stub fns so the module compiles and the linkme DEMOS
//! slice carries the three exer-0N keys. Wave 2 plans (19-02 / 19-03 / 19-04)
//! REPLACE each stub body with the real composition — the file layout and
//! `key = "..."` registrations are locked here.

pub mod nested_appshell;
pub mod pathological_scale;
pub mod rapid_patching;
