# Phase 19: Exerciser Screens - Pattern Map

**Mapped:** 2026-04-24
**Files analyzed:** 16 new + 5 modified = 21 total
**Analogs found:** 18 / 21 (3 have partial / no analog — flagged in §No Analog Found)

> **Consumption note for `gsd-planner`:** This phase has NEARLY ZERO new primitives (per RESEARCH.md §Don't Hand-Roll). Phase 18 catalog screens give you bit-for-bit analogs for the 3 exerciser `#[gallery_demo]` fns. Handler patterns inherit from Phase 17's toast/modal/confirm handlers. The 3 frontend instrumentation files (`exer01/observe.svelte.ts`, `exer02/invariants.svelte.ts`, `exer03/perf.svelte.ts`) have NO perfect analog — they are browser-DOM probes, not SDUI components. Their nearest relatives are `surfaces.focus-preservation.browser-test.ts` (observation idiom) and `sendAction` from dispatcher (transport).

---

## File Classification

### Backend (Rust) — `backend/crates/gallery-demo/`

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `src/exerciser/mod.rs` | module root | n/a | `src/catalog/mod.rs` | exact (trivial pub mod declarations) |
| `src/exerciser/nested_appshell.rs` | model (gallery_demo builder) | transform (Vec\<Node\>) | `src/catalog/feedback.rs` | exact (same Card-based composition shape) |
| `src/exerciser/rapid_patching.rs` | model (gallery_demo builder) | transform (Vec\<Node\>) | `src/catalog/forms.rs` + `src/catalog/data_table.rs` | role-match (multi-card + bound input + RadioGroup) |
| `src/exerciser/pathological_scale.rs` | model (gallery_demo builder) | transform (Vec\<Node\>) | `src/catalog/data_table.rs` + `src/catalog/forms.rs` | exact (reuses CAT-03 DataTable + CAT-02 FieldSet patterns) |
| `src/handlers/exer01.rs` (new) | controller | request-response | `src/handlers/catalog_forms.rs` (validate_text_input) | role-match (payload-in → PatchMessage-out) |
| `src/handlers/exer02.rs` (new) | controller | **streaming / pub-sub** (server-push) | NONE EXACT — closest = `src/handlers/toast.rs` for PatchMessage shape only | **partial** (research gap — see §No Analog Found) |
| `src/handlers/exer03.rs` (new) | controller | request-response | `src/handlers/catalog_forms.rs` | role-match (payload-in → PatchMessage-out writing to /demo/* paths) |
| `src/handlers/fetch_rows.rs` (MODIFY) | controller | request-response | self (existing) | exact — add one match arm |
| `src/handlers/show.rs` (MODIFY) | controller | request-response | self (existing) | exact — add 3 `seed_for_key` arms |
| `src/handlers/mod.rs` (MODIFY) | config (router registration) | n/a | self (existing) | exact — add `.action(...)` calls |
| `src/state.rs` (MODIFY) | model (process state) | n/a | self (existing) | exact — add `exer02_loop`, `exer02_cadence_ms`, `exer02_tick` fields |
| `src/fixtures.rs` | model (generator) | transform | self (existing) | **unchanged** — generator is already generic over `n` (research §Don't Hand-Roll verified) |
| `src/lib.rs` (MODIFY) | module root | n/a | self | exact — add `pub mod exerciser;` |
| `src/main.rs` (MODIFY) | config (boot) | n/a | self | trivial — may need `pub mod exerciser;` if binary's tree mirrors lib |

### Frontend (TypeScript / Svelte) — `frontend/src/lib/`

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `lib/registry/icons.ts` (MODIFY) | config (icon registry) | n/a | self (existing) | exact — append 16 entries |
| `lib/init.ts` (MODIFY) | config (app init) | n/a | self (existing) | exact — add `installPatchProbe(fn)` export + wrap `applyPatch` call on line 45-47 |
| `lib/exer01/observe.svelte.ts` (new) | utility (browser probe) | event-driven | `lib/store/surfaces.focus-preservation.browser-test.ts` | partial (observation idiom — not a mountable helper; see §No Analog Found) |
| `lib/exer02/invariants.svelte.ts` (new) | utility (DOM event watchers) | event-driven | `lib/store/surfaces.focus-preservation.browser-test.ts` (lines 31-66 focus+cursor observation) | partial (pattern exists inside a test; production helper is new) |
| `lib/exer03/perf.svelte.ts` (new) | utility (browser perf probe) | event-driven | NONE — `performance.*` + `PerformanceObserver` are Web APIs; no existing probe module | **no analog** (see §No Analog Found) |

### Planning artifact

| New File | Role | Data Flow | Closest Analog | Match Quality |
|----------|------|-----------|----------------|---------------|
| `.planning/seeds/v1.3-appshell-nestability.md` | doc (seed proposal) | n/a | `.planning/seeds/gallery-live-token-editor.md` | exact (same frontmatter + section shape) |

---

## Pattern Assignments

### `src/exerciser/mod.rs` (module root)

**Analog:** `backend/crates/gallery-demo/src/catalog/mod.rs` (lines 1-20)

**Copy verbatim (pattern):**
```rust
//! Exerciser screens — robustness stress-tests composed with the same builders
//! as catalog screens.
//!
//! Per CONTEXT.md §D-1..D-4, each file inside `exerciser/` hosts its own
//! `#[gallery_demo]` fn; auto-discovery happens via the linkme DEMOS
//! distributed slice populated at link time.
//!
//! See Phase 19 REQUIREMENTS.md §EXER-01..03 for scope. Sibling catalog
//! modules (buttons, forms, data_table, feedback, typography) predate this.

pub mod nested_appshell;
pub mod pathological_scale;
pub mod rapid_patching;
```

---

### `src/exerciser/nested_appshell.rs` (gallery_demo, EXER-01)

**Primary analog:** `backend/crates/gallery-demo/src/catalog/feedback.rs` (lines 29-81)
**Secondary analog:** `backend/crates/gallery-demo/src/catalog/buttons.rs` (lines 27-74) — card flattening pattern

**Imports pattern** (from `feedback.rs:29-31`, plus AppShell):
```rust
use marionette::builders::{AppShell, Badge, Button, Container, Heading, NavItem, Text};
use marionette::gallery::Node;
use marionette_protocol::ComponentAction;
```

**Class constants pattern** (from `feedback.rs:35-42`):
```rust
const OUTER_CLASS: &str = "flex flex-col gap-6 p-6";
const CARD_CLASS: &str =
    "rounded-lg border bg-card text-card-foreground shadow-sm p-6 flex flex-col gap-4";
const MATRIX_GRID_CLASS: &str =
    "grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-3";
```

**`#[gallery_demo]` attribute** (from `buttons.rs:27-30`):
```rust
#[cfg(feature = "gallery")]
#[marionette_macros::gallery_demo(key = "exer-01", name = "Exerciser: Nested AppShell")]
#[must_use]
pub fn gallery_demo() -> Vec<Node> { ... }
```

**Top-level card assembly pattern** (from `feedback.rs:49-81`):
```rust
// Title + intro (locked copy from UI-SPEC §Copywriting)
let title = Heading::new("Nested AppShell").id("exer-01-title").level(1).build();
let intro = Text::new("Outer AppShell hosts an inner AppShell in its content slot…")
    .id("exer-01-intro").build();

// Build Cards via helper fns. Each helper returns (card_root_tuple, descendants).
let (card1_root, card1_desc) = build_structural_preview_card();
let (card2_root, card2_desc) = build_observation_matrix_card();
let (card3_root, card3_desc) = build_v13_proposal_card();

// Outer root: title + intro + 3 card roots.
let (outer_root, outer_direct) = Container::new()
    .id("exer-01-root")
    .class(OUTER_CLASS)
    .children(vec![title, intro, card1_root, card2_root, card3_root])
    .build_tree();

// Flatten: outer root + outer direct children + all card descendants.
let mut result: Vec<Node> = Vec::with_capacity(
    1 + outer_direct.len() + card1_desc.len() + card2_desc.len() + card3_desc.len()
);
result.push(outer_root);
result.extend(outer_direct);
result.extend(card1_desc);
result.extend(card2_desc);
result.extend(card3_desc);
result
```

**Test pattern** (from `buttons.rs:157-234`, `data_table.rs:110-249`):
```rust
#[cfg(all(test, feature = "gallery"))]
mod tests {
    use super::*;
    use marionette::gallery::registered_demos;

    #[test]
    fn root_id_is_exer_01_root() {
        let v = gallery_demo();
        assert_eq!(v[0].0, "exer-01-root");
    }

    #[test]
    fn outer_class_is_locked_string() {
        let v = gallery_demo();
        let root_comp = &v[0].1;
        let val = serde_json::to_value(root_comp).expect("serialize");
        assert_eq!(val["props"]["class"], OUTER_CLASS);
    }

    #[test]
    fn tree_contains_exactly_one_app_shell() {
        // Pitfall 1 regression guard: if someone "fixes" the nesting by
        // falling back to Phase 17's static-preview workaround (Container +
        // Heading + Text), the app-shell count drops to zero and this test
        // fails. The whole point of EXER-01 is the real nested AppShell.
        let v = gallery_demo();
        let count = v.iter().filter(|(_, c)| {
            let s = serde_json::to_value(c).expect("serialize");
            s["type"] == "app-shell"
        }).count();
        // The outer AppShell is mounted by the gallery binary (not by this
        // demo fn), so we expect exactly one app-shell in this demo's returned
        // Vec<Node> — the inner shell.
        assert_eq!(count, 1, "exactly one nested app-shell expected in exer-01");
    }

    #[test]
    fn registered_demos_includes_exer_01() {
        let e = registered_demos().find(|e| e.key == "exer-01")
            .expect("exer-01 must be registered via linkme");
        assert_eq!(e.display_name, "Exerciser: Nested AppShell");
    }
}
```

**Critical anti-pattern to AVOID (from `marionette/src/builders/app_shell.rs:246-320`):**
The existing `AppShell::gallery_demo()` at that path emits a static-preview using `Container + Heading + Text` — NOT a real AppShell. EXER-01 must do the OPPOSITE: it must actually invoke `AppShell::new()` to reproduce the collision. See RESEARCH.md §Anti-Patterns line 569-572 ("Do NOT copy `builders/app_shell.rs::gallery_demo()`'s static-preview pattern").

---

### `src/exerciser/rapid_patching.rs` (gallery_demo, EXER-02)

**Primary analog:** `backend/crates/gallery-demo/src/catalog/forms.rs` (lines 54-122 top-level + 130-200 assembly helper)
**Secondary analog:** `backend/crates/gallery-demo/src/catalog/buttons.rs` (lines 80-155) — per-card descendants flattening

**Imports pattern** (adapted from `forms.rs:21-28`):
```rust
use marionette::builders::radio_group::RadioOption;
use marionette::builders::{
    Badge, Button, Container, Heading, RadioGroup, Text, TextInput,
};
use marionette::gallery::Node;
use marionette_protocol::ComponentAction;
```

**Core card assembly** (pattern from `forms.rs:57-122`) — 4 Cards: Focused input, Cadence control, Invariant dashboard, Patch log.

**Bind + action pattern** (from `forms.rs:200-270` — a representative TextInput with blur-action):
```rust
let focused = TextInput::new()
    .id("exer-02-focused-input")
    .label("Type here — focus must not leak")
    .placeholder("Start typing… paste fast… compose CJK via IME…")
    .description("This input is the sole target of the focus-retention test…")
    .bind("/demo/exer-02/focused-value")
    .build();
```

**RadioGroup pattern** (from `forms.rs` radio card — build helper):
```rust
use marionette::builders::radio_group::RadioOption;

let cadence = RadioGroup::new(vec![
    RadioOption { value: "250".into(), label: "Aggressive (250 ms)".into() },
    RadioOption { value: "500".into(), label: "Default (500 ms)".into() },
    RadioOption { value: "1000".into(), label: "Relaxed (1000 ms)".into() },
    RadioOption { value: "2000".into(), label: "Slow (2000 ms)".into() },
])
.id("exer-02-cadence")
.bind("/demo/exer-02/cadence-ms")
.build();
```

**Button row with action** (from `feedback.rs:93-104`):
```rust
let start_btn = Button::new("Start patching")
    .id("exer-02-start")
    .action(ComponentAction::click("gallery-demo/exer-02/start"))
    .icon("play")
    .build();
let pause_btn = Button::new("Pause patching")
    .id("exer-02-pause")
    .action(ComponentAction::click("gallery-demo/exer-02/pause"))
    .icon("pause")
    .variant("outline")
    .build();
let reset_btn = Button::new("Reset counters")
    .id("exer-02-reset")
    .action(ComponentAction::click("gallery-demo/exer-02/reset"))
    .icon("rotate-ccw")
    .variant("ghost")
    .build();
```

**Patch log container (empty — populated at runtime via SetChildren)** (pattern from `forms.rs` error-slot idiom):
```rust
let log_container = Container::new()
    .id("exer-02-log-container")
    .class("max-h-64 overflow-y-auto font-mono text-xs bg-muted/50 rounded border p-3 flex flex-col gap-1")
    .build();  // empty — backend patches children via SetChildren + SetNode
```

---

### `src/exerciser/pathological_scale.rs` (gallery_demo, EXER-03)

**Primary analog:** `backend/crates/gallery-demo/src/catalog/data_table.rs` (lines 21-108)
**Secondary analog:** `backend/crates/gallery-demo/src/catalog/forms.rs` — FieldSet composition (though pathological_scale needs FieldSet not demonstrated there; see PATTERN NOTE)

**Imports pattern** (merging `data_table.rs:13-16` + FieldSet):
```rust
use marionette::builders::data_table::{ColumnKind, DataTable, Filter, TableColumn};
use marionette::builders::select::SelectOption;
use marionette::builders::{
    Checkbox, Container, FieldSeparator, FieldSet, Heading, RadioGroup, Select,
    Switch, Text, TextInput, Textarea,
};
use marionette::gallery::Node;
```

**DataTable with 10k rows** (adapted from `data_table.rs:69-83` — change `.total_rows(500u64)` to `10_000u64` and `.source("catalog-synthetic-rows")` to `"exer-03-synthetic"`):
```rust
let table = DataTable::new(columns)
    .id("exer-03-data-table")
    .source("exer-03-synthetic")                    // NEW source arm in fetch_rows.rs
    .bind("/demo/exer-03/rows")
    .row_id_key("id")
    .page_size(50u32)                                // Same 50-row page size
    .total_rows(10_000u64)                           // CHANGED: 500 → 10 000
    .filter(Filter::text("name-search").label("Name").placeholder("Filter by name…"))
    .filter(Filter::select("status-filter", status_options).label("Status"))
    .filter(Filter::date_range("joined-range").label("Joined"))
    .build();
```

**80-field FormScreen codegen pattern** — no direct analog; closest is forms.rs per-field construction. Generator strategy:
```rust
fn build_personal_info_group() -> (Node, Vec<Node>) {
    // 20 fields: 15 TextInput, 2 Select, 2 RadioGroup, 1 Textarea
    let fields = vec![
        TextInput::new().id("exer-03-personal-first-name")
            .label("First name").placeholder("Placeholder for First name")
            .bind("/demo/exer-03/personal-info/first-name").build(),
        // ... 14 more TextInput
        // ... 2 Select, 2 RadioGroup, 1 Textarea
    ];
    let (fieldset_root, fieldset_desc) = FieldSet::new("Personal info")
        .id("exer-03-fieldset-personal-info")
        .cols(2)
        .children(fields)
        .build_tree();
    (fieldset_root, fieldset_desc)
}
```
**Analog check needed:** Verify `FieldSet` builder API (`.cols(n)`, `.children(...)`) matches actual builder in `backend/crates/marionette/src/builders/field_set.rs`. Planner should grep for `FieldSet::new` in existing code before generating 80 field calls.

**Seed strategy** (per Pitfall 7 — avoid 10k Set ops at mount time):
In `show.rs`, add `"exer-03"` arm seeding ONLY the 80 field defaults + empty rows (NOT the 10k row slice). `fetch-rows` handles pagination.

---

### `src/handlers/exer01.rs` (new)

**Analog:** `backend/crates/gallery-demo/src/handlers/catalog_forms.rs` (lines 75-135 `validate_text_input`)

**Imports pattern**:
```rust
use marionette::error::ActionResult;
use marionette::extractors::HandlerContext;
use marionette_protocol::data::PatchOperation;
use marionette_protocol::messages::PatchMessage;
use marionette_protocol::ProtocolMessage;
use serde::Deserialize;
```

**Payload deserialization** (from `fetch_rows.rs:14-21, 29-32`):
```rust
#[derive(Debug, Deserialize)]
struct ObservationReport {
    #[serde(rename = "provider-context")]
    provider_context: MatrixEntry,
    #[serde(rename = "mobile-sheet")]
    mobile_sheet: MatrixEntry,
    #[serde(rename = "keyboard-shortcuts")]
    keyboard_shortcuts: MatrixEntry,
    #[serde(rename = "sidebar-tokens")]
    sidebar_tokens: MatrixEntry,
}

#[derive(Debug, Deserialize)]
struct MatrixEntry {
    state: String,   // "PASS" / "FAIL" / "WARN"
    details: String,
}
```

**Handler body pattern** (from `catalog_forms.rs:75-135`):
```rust
#[allow(clippy::unused_async)]
pub async fn handle_exer01_report(ctx: HandlerContext) -> ActionResult {
    let payload: ObservationReport = serde_json::from_value(
        ctx.action.payload.clone().unwrap_or_default(),
    )
    .map_err(|e| marionette::error::ActionError::BadPayload(
        format!("exer-01 report invalid: {e}")
    ))?;

    let patch = vec![
        PatchOperation::Set {
            path: "/demo/exer-01/matrix/provider-context".into(),
            value: serde_json::to_value(&payload.provider_context).unwrap(),
        },
        // ... 3 more Set ops
    ];

    Ok(vec![ProtocolMessage::Patch(PatchMessage {
        id: ctx.action.id.clone(),
        surface: "content".into(),
        patch,
    })])
}
```

---

### `src/handlers/exer02.rs` (new — **ARCHITECTURAL GAP**)

**Primary shape analog:** `backend/crates/gallery-demo/src/handlers/toast.rs` (lines 12-36 for PatchMessage construction)
**Server-push gap:** NO analog — see §No Analog Found below.

**What IS analogous (PatchMessage construction, state handle pattern)** — from `toast.rs:12-36`:
```rust
use marionette::error::ActionResult;
use marionette::extractors::HandlerContext;
use marionette_protocol::data::PatchOperation;
use marionette_protocol::messages::PatchMessage;
use marionette_protocol::ProtocolMessage;

pub async fn handle_exer02_start(ctx: HandlerContext) -> ActionResult {
    // ... cancellation + spawn loop (from RESEARCH.md §Example 2, lines 812-868)
    // GAP: the spawned loop CANNOT send PatchMessages out-of-band because
    // HandlerContext exposes no broadcaster. Plan 19-02 MUST resolve this.
    Ok(vec![])
}
```

**Reset handler pattern** (from RESEARCH.md §Example 2 lines 877-895 — deterministic one-shot patch-back):
This is a regular request-response handler — it returns `Ok(vec![ProtocolMessage::Patch(...)])` directly, so no gap.

**State extension** (add to `state.rs:19-28`):
```rust
#[derive(Clone, Default)]
pub struct GalleryState {
    pub demo_values: Arc<RwLock<HashMap<String, serde_json::Value>>>,
    pub modal_open: Arc<RwLock<bool>>,
    pub confirm_open: Arc<RwLock<bool>>,
    // NEW for EXER-02:
    pub exer02_loop: Arc<Mutex<Option<JoinHandle<()>>>>,
    pub exer02_cadence_ms: Arc<Mutex<u64>>,
    pub exer02_tick: Arc<Mutex<u64>>,
}
```

---

### `src/handlers/exer03.rs` (new)

**Analog:** `backend/crates/gallery-demo/src/handlers/catalog_forms.rs` (lines 84-133 `validate_text_input`) — payload → PatchOperation::Set × N
**Round-trip pattern** — frontend measures, calls `sendAction`, backend writes `/demo/exer-03/perf/*` paths.

**Imports + payload** (from `catalog_forms.rs:11-18, 43-60`):
```rust
use marionette::error::ActionResult;
use marionette::extractors::HandlerContext;
use marionette_protocol::data::PatchOperation;
use marionette_protocol::messages::PatchMessage;
use marionette_protocol::ProtocolMessage;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct PerfSnapshot {
    ttfp_ms: Option<f64>,
    fps: Option<f64>,
    memory_mb: Option<f64>,
    latency_p95_ms: Option<f64>,
}

#[allow(clippy::unused_async)]
pub async fn handle_exer03_report_perf(ctx: HandlerContext) -> ActionResult {
    let payload: PerfSnapshot = serde_json::from_value(
        ctx.action.payload.clone().unwrap_or_default(),
    )
    .map_err(|e| marionette::error::ActionError::BadPayload(
        format!("exer-03 perf payload invalid: {e}")
    ))?;

    let mut patch: Vec<PatchOperation> = Vec::new();
    if let Some(v) = payload.ttfp_ms {
        patch.push(PatchOperation::Set {
            path: "/demo/exer-03/perf/ttfp_ms".into(),
            value: serde_json::json!(v),
        });
    }
    // ... same for fps, memory_mb, latency_p95_ms
    Ok(vec![ProtocolMessage::Patch(PatchMessage {
        id: ctx.action.id.clone(),
        surface: "content".into(),
        patch,
    })])
}

#[allow(clippy::unused_async)]
pub async fn handle_exer03_remeasure(ctx: HandlerContext) -> ActionResult {
    // Emit a single marker Set so the frontend's reactive instrumentation
    // knows to re-capture; may also emit a toast via the toast.rs pattern.
    Ok(vec![ProtocolMessage::Patch(PatchMessage {
        id: ctx.action.id.clone(),
        surface: "content".into(),
        patch: vec![PatchOperation::Set {
            path: "/demo/exer-03/perf/remeasure-tick".into(),
            value: serde_json::json!(chrono::Utc::now().timestamp_millis()),
        }],
    })])
}
```

---

### `src/handlers/fetch_rows.rs` (MODIFY — add `exer-03-synthetic` source arm)

**Analog:** self, `fetch_rows.rs:34-62` — existing `"catalog-synthetic-rows"` match arm

**Pattern to copy** (from `fetch_rows.rs:36-55`):
```rust
let (path_prefix, rows): (&str, Vec<serde_json::Value>) = match payload.source.as_str() {
    "demo-rows" => ("/demo/data-table/rows", demo_rows_legacy()),
    "catalog-synthetic-rows" => {
        let all = crate::fixtures::synthetic_rows(500);
        // ... existing slice + action injection ...
        ("/demo/catalog-data-table/rows", json_rows)
    }
    // NEW ARM:
    "exer-03-synthetic" => {
        let all = crate::fixtures::synthetic_rows(10_000);        // CHANGED: 10k
        let start = payload.offset as usize;
        let end = start.saturating_add(payload.limit as usize).min(all.len());
        let slice = all.get(start..end).unwrap_or(&[]);
        let json_rows: Vec<serde_json::Value> = slice
            .iter()
            .map(|r| {
                let mut v = serde_json::to_value(r).expect("Row serializes");
                v["actions"] = serde_json::json!([
                    { "label": "Edit",      "action": { "type": "click", "name": "gallery-demo/noop" } },
                    { "label": "Delete",    "action": { "type": "click", "name": "gallery-demo/noop" } },
                    { "label": "Duplicate", "action": { "type": "click", "name": "gallery-demo/noop" } },
                ]);
                v
            })
            .collect();
        ("/demo/exer-03/rows", json_rows)                          // NEW path
    }
    other => return Err(ActionError::BadPayload(format!("unknown fetch-rows source: {other}"))),
};
```

**Test pattern to copy** (from `fetch_rows.rs:130-170`):
```rust
#[tokio::test]
async fn exer03_rows_first_page_50_ids_1_through_50() {
    let ctx = make_ctx(serde_json::json!({
        "source": "exer-03-synthetic", "offset": 0, "limit": 50
    }));
    let result = handle_demo_fetch_rows(ctx).await.expect("ok");
    let ProtocolMessage::Patch(msg) = &result[0] else { panic!("expected Patch") };
    assert_eq!(msg.patch.len(), 50);
    let PatchOperation::Set { path, .. } = &msg.patch[0] else { panic!() };
    assert_eq!(path, "/demo/exer-03/rows/1");
}

#[tokio::test]
async fn exer03_rows_last_page_offset_9950() {
    // Proves the 10 000 cap is honoured.
    let ctx = make_ctx(serde_json::json!({
        "source": "exer-03-synthetic", "offset": 9950, "limit": 50
    }));
    let result = handle_demo_fetch_rows(ctx).await.expect("ok");
    let ProtocolMessage::Patch(msg) = &result[0] else { panic!() };
    assert_eq!(msg.patch.len(), 50);
    let PatchOperation::Set { path, .. } = &msg.patch[49] else { panic!() };
    assert_eq!(path, "/demo/exer-03/rows/10000");
}
```

---

### `src/handlers/show.rs` (MODIFY — add 3 `seed_for_key` arms)

**Analog:** self, `show.rs:62-210` — existing `seed_for_key` arms

**Pattern** (from `show.rs:106` pattern — `catalog-buttons` empty-seed arm):
```rust
// in seed_for_key's match:
"exer-01" => serde_json::json!({
    "demo": { "exer-01": {
        "matrix": {
            // Initial seed per UI-SPEC §EXER-01 Observation matrix copy lines 180-184.
            "provider-context": {
                "state": "FAIL",
                "details": "shadcn <Sidebar.Provider> is not scoped: the inner provider re-mounts with the same viewport anchors as the outer, visually replacing the outer 20-entry nav. Observed 2026-04-22 in Phase 17 G-02.",
            },
            "mobile-sheet": {
                "state": "FAIL",
                "details": "Inner AppShell on narrow viewport opens a Sheet that covers the outer Sheet; dismissing either closes both. Expected scoping by surface name is absent.",
            },
            "keyboard-shortcuts": {
                "state": "FAIL",
                "details": "Sidebar toggle shortcut (Ctrl+B by default) triggers both providers. Last-registered wins; which shell responds is implementation-detail, not contract.",
            },
            "sidebar-tokens": {
                "state": "WARN",
                "details": "CSS custom-property inheritance cascades naturally: inner shell inherits whatever the outer sets. Not a bug per se — scoped tokens would need :where(.surface-name) or a style-isolation mechanism.",
            },
        },
    }},
}),

"exer-02" => serde_json::json!({
    "demo": { "exer-02": {
        "focused-value": "",
        "cadence-ms": 500,
        "invariants": {
            "focus":  { "state": "PENDING" },
            "cursor": { "state": "PENDING" },
            "typed":  { "state": "PENDING" },
            "ime":    { "state": "PENDING" },
        },
        "elapsed-s": 0,
    }},
}),

// Pattern from the larger catalog-forms seed (show.rs:128-181).
// Per Pitfall 7: EXER-03 seeds empty rows (NOT 10k at mount time — let fetch-rows paginate).
"exer-03" => serde_json::json!({
    "demo": { "exer-03": {
        "rows": {},    // empty — fetch-rows pages lazily
        "perf": {
            "ttfp_ms": null,
            "fps": null,
            "memory_mb": null,
            "latency_p95_ms": null,
            "remeasure-tick": 0,
        },
        // 80 field defaults — planner generates these from UI-SPEC §EXER-03 table
        "personal-info": {
            "first-name": "",
            "last-name": "",
            // ... 18 more Personal info fields
        },
        "contact": { /* 20 fields */ },
        "preferences": { /* 20 fields */ },
        "advanced": { /* 20 fields */ },
    }},
}),
```

**Test pattern to copy** (from `show.rs:290-317` `catalog_data_table_seed_matches_row_shape_and_action_injection`): assert the seed shape stays stable.

---

### `src/handlers/mod.rs` (MODIFY — register actions)

**Analog:** self, `mod.rs:24-75` — existing `register_gallery_actions`

**Pattern** (add after the `catalog-forms/validate-*` block, line 74):
```rust
// --- EXER-01/02/03 handlers (Phase 19) ---
.action(
    "gallery-demo/exer-01/report",
    box_handler(exer01::handle_exer01_report),
    AuthRequirement::None,
)
.action(
    "gallery-demo/exer-02/start",
    box_handler(exer02::handle_exer02_start),
    AuthRequirement::None,
)
.action(
    "gallery-demo/exer-02/pause",
    box_handler(exer02::handle_exer02_pause),
    AuthRequirement::None,
)
.action(
    "gallery-demo/exer-02/reset",
    box_handler(exer02::handle_exer02_reset),
    AuthRequirement::None,
)
.action(
    "gallery-demo/exer-03/report-perf",
    box_handler(exer03::handle_exer03_report_perf),
    AuthRequirement::None,
)
.action(
    "gallery-demo/exer-03/remeasure",
    box_handler(exer03::handle_exer03_remeasure),
    AuthRequirement::None,
)
```

And add the module declarations after line 17:
```rust
pub mod exer01;
pub mod exer02;
pub mod exer03;
```

---

### `src/lib.rs` (MODIFY — add exerciser module)

**Analog:** self, `lib.rs:25-29` — existing `pub mod catalog;`

**Pattern** (insert at line 27):
```rust
pub mod catalog;
pub mod exerciser;     // NEW
pub mod fixtures;
pub mod handlers;
```

If the `main.rs` binary mirrors with `mod ...;` declarations, add there too. (Inspect `main.rs:39-40` — the binary calls `gallery_demo::handlers::register_gallery_actions`; modules traverse via lib, so main.rs likely needs no change. Verify during planning.)

---

### `src/state.rs` (MODIFY — add EXER-02 task handle)

**Analog:** self, `state.rs:19-28` — existing GalleryState struct

**Pattern** (add fields — note the `Mutex` import at top):
```rust
use tokio::sync::{Mutex, RwLock};
use tokio::task::JoinHandle;

#[derive(Clone, Default)]
pub struct GalleryState {
    pub demo_values: Arc<RwLock<HashMap<String, serde_json::Value>>>,
    pub modal_open: Arc<RwLock<bool>>,
    pub confirm_open: Arc<RwLock<bool>>,
    // --- NEW for EXER-02 (Phase 19) ---
    /// Active patch-loop task handle. `Some` while loop is running; `None`
    /// when paused/stopped. Pause/Reset MUST abort before storing None
    /// (Pitfall 9) — use `std::mem::take(&mut *guard)` then `.abort()`.
    pub exer02_loop: Arc<Mutex<Option<JoinHandle<()>>>>,
    /// Current cadence in ms — mirrors /demo/exer-02/cadence-ms (frontend writes, backend reads).
    pub exer02_cadence_ms: Arc<Mutex<u64>>,
    /// Monotonic tick counter for patch iteration ids.
    pub exer02_tick: Arc<Mutex<u64>>,
}
```

**Integration gap flag:** `GalleryState` is defined but NOT currently wired into `AppState` (which is the struct the WebSocket handler uses). See `main.rs:42-47` — the current `AppState` has `router`, `db`, `login_form`, `listmonk` but no `GalleryState`. Plan 19-02 MUST resolve how EXER-02 handlers access `exer02_loop` — either extend `AppState` or use a separate `OnceLock<GalleryState>`. See §No Analog Found.

---

### `lib/registry/icons.ts` (MODIFY — append 16 icons)

**Analog:** self, `icons.ts:1-48` — existing registry

**Pattern** (add imports at top + entries in `defaults` array — from UI-SPEC §Design System "Icon additions required" lines 35-56):
```typescript
// Add after the existing 14 imports (line 15):
import Activity from '@lucide/svelte/icons/activity';
import Focus from '@lucide/svelte/icons/focus';
import Type from '@lucide/svelte/icons/type';
import Languages from '@lucide/svelte/icons/languages';
import MoveHorizontal from '@lucide/svelte/icons/move-horizontal';
import Gauge from '@lucide/svelte/icons/gauge';
import Timer from '@lucide/svelte/icons/timer';
import Cpu from '@lucide/svelte/icons/cpu';
import Zap from '@lucide/svelte/icons/zap';
import LayoutDashboard from '@lucide/svelte/icons/layout-dashboard';
import Layers from '@lucide/svelte/icons/layers';
import TriangleAlert from '@lucide/svelte/icons/triangle-alert';
import CircleCheck from '@lucide/svelte/icons/circle-check';
import CircleX from '@lucide/svelte/icons/circle-x';
import Play from '@lucide/svelte/icons/play';
import Pause from '@lucide/svelte/icons/pause';
import RotateCcw from '@lucide/svelte/icons/rotate-ccw';

// Then append to `defaults` array (after line 42 `['circle-help', CircleHelp],`):
['activity', Activity],
['focus', Focus],
['type', Type],
['languages', Languages],
['move-horizontal', MoveHorizontal],
['gauge', Gauge],
['timer', Timer],
['cpu', Cpu],
['zap', Zap],
['layout-dashboard', LayoutDashboard],
['layers', Layers],
['triangle-alert', TriangleAlert],
['circle-check', CircleCheck],
['circle-x', CircleX],
['play', Play],
['pause', Pause],
['rotate-ccw', RotateCcw],
```

Note: the UI-SPEC lists 16 icon *additions* — the list above has 17 entries (added `rotate-ccw`). Verify the UI-SPEC count; if 16, drop one and document in plan.

---

### `lib/init.ts` (MODIFY — add `installPatchProbe`)

**Analog:** self, `init.ts:44-52` — existing `patch` handler registration

**Pattern** (wrap the existing `applyPatch` call, add exported hook at module scope):
```typescript
// At module top-level (near imports or after helpers):
let patchProbe: ((latencyMs: number) => void) | null = null;

/**
 * Install a callback invoked after every patch application. Used by
 * frontend/src/lib/exer02/invariants.svelte.ts (patch-tick coordination
 * for cursor + IME watchers — see Pitfall 5) and
 * frontend/src/lib/exer03/perf.svelte.ts (patch latency p95 — see
 * Pattern 4 in 19-RESEARCH.md).
 */
export function installPatchProbe(fn: ((latencyMs: number) => void) | null): void {
    patchProbe = fn;
}

// Inside initMarionette, replace the existing patch handler (lines 44-52):
registerHandler('patch', (raw: unknown) => {
    const msg = raw as PatchMessage;
    const t0 = performance.now();
    applyPatch(msg.surface, msg.patch);
    const dt = performance.now() - t0;
    if (patchProbe) patchProbe(dt);
    if (msg.id) {
        confirmOptimistic(msg.id);
    }
});
```

---

### `lib/exer01/observe.svelte.ts` (new)

**Analog (partial):** `lib/store/surfaces.focus-preservation.browser-test.ts` (observation idiom — extract focus/cursor/value at a moment in time)
**Analog (also):** `lib/components/ui/sidebar/context.svelte.ts` (lines 62-81 — the `Symbol.for("scn-sidebar")` key and `getContext` pattern)

**Imports pattern** (from RESEARCH.md §Code Examples Example 1 line 678-679):
```typescript
import { getContext } from 'svelte';
import { sendAction } from '$lib/transport/dispatcher';
```

**Key to probe** (verbatim from `sidebar/context.svelte.ts:62`):
```typescript
const SIDEBAR_KEY = Symbol.for('scn-sidebar');
const SIDEBAR_KEYBOARD_SHORTCUT = 'b';  // from sidebar/constants.ts
```

**Probe function structure** — See RESEARCH.md §Code Examples Example 1 lines 696-779. Ready-to-copy.

**Frontend mount strategy:** Plan 19-01 must decide how/where `probeNestability()` is called — options (planner picks):
1. `onMount` inside a new Svelte component wrapping the inner AppShell.
2. Global hook on `window.__mrnExer01OuterSidebar` set at outer mount + `onMount` inside the inner mount.

---

### `lib/exer02/invariants.svelte.ts` (new)

**Analog:** `lib/store/surfaces.focus-preservation.browser-test.ts` (lines 31-66) — focus + cursor + value observation
**Secondary (event wire):** MDN `compositionstart`/`compositionend` (no codebase analog for IME).

**Observation idiom** (from `surfaces.focus-preservation.browser-test.ts:39-44`):
```typescript
const inputA = inputs[0] as HTMLInputElement;
inputA.focus();
inputA.value = 'hello';
inputA.dispatchEvent(new Event('input', { bubbles: true }));
inputA.setSelectionRange(3, 3);
expect(document.activeElement).toBe(inputA);
expect(inputA.selectionStart).toBe(3);
```

**Watcher module structure** — See RESEARCH.md §Code Examples §Pattern 3 lines 398-476 and surrounding Pitfall 5 (lines 622-630 re cursor-tick coordination). Key imports:
```typescript
import { installPatchProbe } from '$lib/init';      // NEW export
import { sendAction } from '$lib/transport/dispatcher';
```

**Invariant update pattern** — follow RESEARCH.md lines 402-476 which is the full implementation sketch.

---

### `lib/exer03/perf.svelte.ts` (new)

**Analog:** NONE in codebase — `PerformanceObserver`, `performance.memory`, `requestAnimationFrame` are Web APIs.
**Ready-to-copy implementation:** RESEARCH.md §Pattern 4 lines 486-564 — use this as the direct source (it is the research artifact; planner may paste it into the plan with minimal adaptation).

**Imports pattern:**
```typescript
import { sendAction } from '$lib/transport/dispatcher';
// Register recordPatchLatency as the patch probe at module load:
import { installPatchProbe } from '$lib/init';
```

**Module-level probe registration:**
```typescript
// At module bottom — install the latency recorder when this file is imported.
installPatchProbe((ms) => recordPatchLatency(ms));
```

**Chrome-only guard** (CRITICAL — Pitfall 4):
```typescript
export function captureMemoryMb(): number | null {
    const p = performance as Performance & { memory?: { usedJSHeapSize: number } };
    if (!p.memory) return null;  // Firefox / Safari → UI-SPEC "Perf measurement API unavailable" copy fires
    return p.memory.usedJSHeapSize / (1024 * 1024);
}
```

---

### `.planning/seeds/v1.3-appshell-nestability.md` (new seed)

**Analog:** `.planning/seeds/gallery-live-token-editor.md` (entire file — 51 lines)

**Frontmatter pattern** (from `gallery-live-token-editor.md:1-6`):
```yaml
---
title: Scoped surface-name framework extension for nested AppShell
planted_date: "2026-04-24"
trigger_condition: "v1.3 milestone kickoff, OR earlier if a real app needs multi-shell embedding (e.g., settings sub-page hosting a mini-app)"
status: planted
---
```

**Section shape to copy** (from `gallery-live-token-editor.md:8-50`):
- `# <Title>`
- `## Idea` (2-3 paragraphs)
- `## Why planted (not built immediately)` (bullet list)
- `## Trigger` (1 paragraph)
- `## Design sketch (rough, not locked)` (bullet list)
- `## Related` (bullet list)

**Content sources:**
- CONTEXT.md §D-1 frames the proposal (scoped-surface-name + scoped Sidebar.Provider).
- RESEARCH.md §Pitfall 1 lines 598-602 and §Code Examples Example 1 provide the technical shape.
- UI-SPEC §EXER-01 v1.3 proposal Card body line 192 provides customer-facing framing.

---

## Shared Patterns

### `#[gallery_demo]` auto-discovery

**Source:** `backend/crates/gallery-demo/src/catalog/buttons.rs:27-30`
**Apply to:** All 3 exerciser files (`nested_appshell.rs`, `rapid_patching.rs`, `pathological_scale.rs`)

```rust
#[cfg(feature = "gallery")]
#[marionette_macros::gallery_demo(key = "<kebab-case-key>", name = "<Display Name>")]
#[must_use]
pub fn gallery_demo() -> Vec<Node> { /* ... */ }
```

Keys for Phase 19: `"exer-01"`, `"exer-02"`, `"exer-03"`.
Display names: `"Exerciser: Nested AppShell"`, `"Exerciser: Rapid Patching"`, `"Exerciser: Pathological Scale"`.

### Card + build_tree flattening

**Source:** `backend/crates/gallery-demo/src/catalog/feedback.rs:49-81` (top-level) + `100-165` (helpers)
**Apply to:** All 3 exerciser files

Each Card is a helper fn `fn build_<name>_card() -> (Node, Vec<Node>)` that:
1. Builds heading + children (e.g., grid, field, buttons)
2. Wraps inner content in a grid `Container` via `.build_tree()` (returns root tuple + descendants)
3. Wraps the grid in a Card `Container` with `CARD_CLASS`
4. Returns (card_root, flattened_descendants)

Top-level `gallery_demo()` collects all card roots into outer `.children(...)` and all descendants into a flat `Vec<Node>`.

### Locked class constants

**Source:** `backend/crates/gallery-demo/src/catalog/feedback.rs:35-42`
**Apply to:** All 3 exerciser files

```rust
const OUTER_CLASS: &str = "flex flex-col gap-6 p-6";
const CARD_CLASS: &str =
    "rounded-lg border bg-card text-card-foreground shadow-sm p-6 flex flex-col gap-4";
// Screen-specific grid class per UI-SPEC §Spacing Scale (EXER-01/02/03 all use 4-col at lg)
const DASHBOARD_GRID_CLASS: &str = "grid grid-cols-2 sm:grid-cols-2 lg:grid-cols-4 gap-3";
```

### Handler shape: PatchMessage response

**Source:** `backend/crates/gallery-demo/src/handlers/catalog_forms.rs:67-73` (`patch_response` helper)
**Apply to:** `exer01.rs`, `exer03.rs` (NOT `exer02.rs` start/pause — those need server-push; see §No Analog Found)

```rust
#[allow(clippy::unnecessary_wraps)]
fn patch_response(ctx: &HandlerContext, patch: Vec<PatchOperation>) -> ActionResult {
    Ok(vec![ProtocolMessage::Patch(PatchMessage {
        id: ctx.action.id.clone(),
        surface: "content".into(),
        patch,
    })])
}
```

### Payload extraction

**Source:** `backend/crates/gallery-demo/src/handlers/catalog_forms.rs:43-60` (`payload_string`, `payload_bool`)
**Apply to:** `exer01.rs`, `exer03.rs`

For typed payloads, use `serde_json::from_value(ctx.action.payload.clone().unwrap_or_default()).map_err(...)` — see `fetch_rows.rs:29-32`.

### Test harness (action handlers)

**Source:** `backend/crates/gallery-demo/src/handlers/fetch_rows.rs:95-239`
**Apply to:** `exer01.rs`, `exer02.rs`, `exer03.rs` unit tests

```rust
fn mock_db() -> Arc<sea_orm::DatabaseConnection> {
    Arc::new(MockDatabase::new(DatabaseBackend::Sqlite).into_connection())
}

fn anonymous_session() -> Session {
    Session { user_id: None, roles: vec![] }
}

fn make_ctx(payload: serde_json::Value) -> HandlerContext {
    HandlerContext {
        action: ActionMessage {
            id: Some("t1".into()),
            name: "exer-…".into(),
            source: None,
            payload: Some(payload),
            optimistic: None,
        },
        db: mock_db(),
        session: anonymous_session(),
    }
}
```

### Test harness (gallery_demo builder)

**Source:** `backend/crates/gallery-demo/src/catalog/data_table.rs:110-249` (complete suite of 7 tests)
**Apply to:** All 3 exerciser files

Required tests per exerciser:
1. `root_id_is_<key>_root` — outer Container id matches convention
2. `outer_class_is_locked_string` — `OUTER_CLASS` is applied
3. `registered_demos_includes_<key>` — linkme registration works
4. Screen-specific invariants (e.g., EXER-01: exactly one `app-shell` type in tree; EXER-03: 80 text-input binds uniquely prefixed by group)
5. `seed_for_key("<key>")` alignment test — every bind path in the tree has a matching seed entry (from `show.rs:360-409`)

### Toast emission (for `Remeasure` trigger UX)

**Source:** `backend/crates/gallery-demo/src/handlers/toast.rs:11-36`
**Apply to:** `exer03.rs` `handle_exer03_remeasure` — planner may emit a toast acknowledging the trigger:

```rust
let toast_id = format!("toast-exer03-remeasure-{}", uuid::Uuid::new_v4());
let (_, toast_node) = Button::new("Remeasurement triggered — readings will update on next paint")
    .id(&toast_id)
    .action(ComponentAction::click("dismiss-toast"))
    .build();
// Followed by SetNode + InsertChild ops into "toasts-root"
```

### Frontend action send

**Source:** `frontend/src/lib/transport/dispatcher.ts:43-73` (`sendAction`)
**Apply to:** `exer01/observe.svelte.ts`, `exer02/invariants.svelte.ts`, `exer03/perf.svelte.ts`

```typescript
import { sendAction } from '$lib/transport/dispatcher';
sendAction('gallery-demo/exer-01/report', { /* payload */ });
```

---

## No Analog Found

Files where the codebase has no close match — planner must synthesize from RESEARCH.md directly (research ships ready-to-copy code).

| File | Role | Data Flow | Reason | Where to look in RESEARCH.md |
|------|------|-----------|--------|------------------------------|
| `src/handlers/exer02.rs` start/pause (the tokio-interval loop with server push) | controller | **streaming / pub-sub** | **No backend handler in the codebase emits PatchMessages out-of-band.** Every existing handler (toast, modal, confirm, catalog_forms, fetch_rows, show) returns `ActionResult` synchronously. The WebSocket `write_loop` (ws.rs:105) is private; `HandlerContext` exposes only `action`, `db`, `session` (extractors.rs:29-36). The tick loop pattern from RESEARCH.md §Example 2 lines 812-868 calls out this gap explicitly: `// Assumption: ctx exposes a way to BROADCAST messages to all connected clients... This detail is the single most important Plan 19-02 research-during-planning item.` | §Pattern 2 lines 341-390 (shape), §Example 2 lines 782-896 (full impl), §Pitfall 9 (task handle lifecycle) |
| `lib/exer03/perf.svelte.ts` | utility | event-driven (browser timing APIs) | No existing frontend module uses `PerformanceObserver`, `performance.memory`, or rAF-based FPS sampling. Closest adjacent concept is `virtualizer.svelte.ts` (uses rAF indirectly via tanstack-virtual) but it is not a timing probe. | §Pattern 4 lines 479-564 (full impl), §Pitfall 3 (TTFP observer timing), §Pitfall 4 (`performance.memory` Chrome-only) |
| `lib/exer01/observe.svelte.ts` (full shape) | utility | event-driven (DOM + getContext probe) | The `getContext(Symbol.for("scn-sidebar"))` read exists only inside sidebar internals (`sidebar/context.svelte.ts:80`). No existing module cross-cuts the shadcn provider ecosystem from outside. | §Code Examples Example 1 lines 672-779 (full impl) |

---

## AppState integration gap (FLAG FOR PLANNER)

`GalleryState` (state.rs:19-28) is defined but NOT currently held by `AppState` (ws.rs:24-33) nor instantiated in `main.rs:42-47`. Every existing handler gets data via `HandlerContext.action.payload` / `HandlerContext.db` — none references `GalleryState` today. Plan 19-02 MUST resolve ONE of:

1. **Option A (invasive):** Extend `AppState` (in `marionette::ws`) with `pub gallery: Option<Arc<...>>` — touches the framework crate.
2. **Option B (contained):** Store `GalleryState` in a crate-local `once_cell::sync::Lazy<GalleryState>` inside `gallery-demo/src/state.rs`; handlers access via `state()` helper.
3. **Option C (per-call):** Pass state by capturing it in closures at `register_gallery_actions` time (requires router API change).

Option B is the least-invasive choice and matches the "dev-local harness, no new framework primitives" ethos locked in CONTEXT.md §Inherited Decisions. Planner should verify `once_cell` is in the workspace dep tree (likely yes — widely used) or substitute `std::sync::OnceLock`.

---

## Metadata

**Analog search scope:**
- `backend/crates/gallery-demo/src/` (catalog, handlers, fixtures, state, lib, main)
- `backend/crates/marionette/src/` (builders, extractors, gallery, ws)
- `backend/crates/marionette-protocol/src/data.rs`
- `frontend/src/lib/` (init, registry, store, transport, utils, components/ui/sidebar)
- `.planning/seeds/`
- `.planning/phases/17-*` and `18-*` SUMMARY patterns (inherited)

**Files scanned (read):** 18
- `backend/crates/gallery-demo/src/catalog/{buttons,data_table,feedback,forms,typography}.rs` (partial for latter two)
- `backend/crates/gallery-demo/src/catalog/mod.rs`
- `backend/crates/gallery-demo/src/handlers/{mod,show,fetch_rows,toast,modal,confirm,noop,catalog_forms}.rs`
- `backend/crates/gallery-demo/src/{fixtures,state,lib,main}.rs`
- `backend/crates/marionette/src/builders/app_shell.rs` (lines 240-320 — negative analog)
- `backend/crates/marionette/src/extractors.rs` (HandlerContext definition)
- `backend/crates/marionette/src/ws.rs` (AppState definition — gap identification)
- `frontend/src/lib/init.ts`
- `frontend/src/lib/registry/icons.ts`
- `frontend/src/lib/transport/dispatcher.ts`
- `frontend/src/lib/store/surfaces.focus-preservation.browser-test.ts`
- `frontend/src/lib/components/ui/sidebar/context.svelte.ts`
- `frontend/src/lib/utils/virtualizer.svelte.ts`
- `.planning/seeds/gallery-live-token-editor.md`

**Files classified:** 21 (14 backend, 5 frontend, 1 seed, plus the lib.rs/main.rs pair where only lib.rs likely needs change)

**Analogs found matrix:**
- Exact: 11 (mod.rs, nested_appshell.rs, pathological_scale.rs, fetch_rows MODIFY, show MODIFY, handlers/mod MODIFY, state.rs MODIFY, lib.rs MODIFY, icons.ts MODIFY, init.ts MODIFY, v1.3 seed)
- Role-match: 4 (exer01.rs, exer03.rs, rapid_patching.rs, exer02 reset-only half)
- Partial: 3 (exer01 observe.svelte.ts, exer02 invariants.svelte.ts, exer02.rs start/pause half)
- No analog: 1 (exer03 perf.svelte.ts)

**Pattern extraction date:** 2026-04-24
