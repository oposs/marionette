# Phase 15 Deferred Items

## From Plan 15-01 execution

### Pre-existing clippy doc_markdown warnings in marionette crate

**Discovered during:** Plan 15-01 Task 2 verification (running `cargo clippy -p marionette-protocol -p marionette -- -D warnings`).

**Scope:** `backend/crates/marionette/src/builders/standard.rs` — 6 `doc_markdown` errors (missing backticks around terms like `FieldSet`, etc.).

**Origin:** Pre-existing; last touched by Phase 14 commits (`a599c84`, `5d58921`, `2c5856a`). Not introduced by Plan 15-01.

**Example:**
```
error: item in documentation is missing backticks
   --> crates/marionette/src/builders/standard.rs:223:42
    |
223 |     /// take the full row inside a 2-col FieldSet.
```

**Recommendation:** Address in a dedicated doc-sweep plan or alongside Phase 15 Plan 06 (Flowbite residue / doc cleanup). Out of scope for 15-01 per Rule 4 scope boundary (pre-existing, not caused by this plan's changes).
