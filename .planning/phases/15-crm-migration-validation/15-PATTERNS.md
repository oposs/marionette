# Phase 15: CRM Migration & Validation — Pattern Map

**Mapped:** 2026-04-18
**Files analyzed:** ~25 (new + modified)
**Analogs found:** 24 / 25 (one helper has no direct analog — composed from two in-repo patterns)

> **How to read this doc:** Every new or modified file is listed with (a) its role, (b) its data-flow / intent, (c) the closest existing analog in the codebase, (d) a concrete excerpt from that analog (with line numbers) showing the pattern to mirror, and (e) a one-line delta describing what's different for the Phase 15 target.
>
> The planner should prefer these excerpts over RESEARCH.md speculation — they are the *actual* shape the Phase 15 code must match.

---

## File Classification

| # | Target file | Role | Data Flow | Closest Analog | Match Quality |
|---|-------------|------|-----------|----------------|---------------|
| 1 | `backend/crates/crm-demo/src/migration/m20260418_000011_extend_contact.rs` | migration | schema-change | `backend/crates/crm-demo/src/migration/m20260323_000004_create_contact.rs` | role-match (create vs alter) |
| 2 | `backend/crates/crm-demo/src/migration/mod.rs` (edit) | migration-registry | schema-chain | same file (existing registrations) | exact |
| 3 | `backend/crates/crm-demo/src/entities/contact.rs` (edit) | ORM-model | CRUD | same file (existing Model struct) | exact |
| 4 | `backend/crates/crm-demo/src/seed.rs` (edit) | fixture-data | batch-insert | same file (existing `seed_contacts`) | exact |
| 5 | `backend/crates/marionette/src/builders/standard.rs` `form_shell()` (new helper) | builder-helper | composition | Composite: `handlers/contact.rs:664-674` (handler-inline pattern) + `standard.rs:916-950` (`build_with_children` test shows shape) | synthesis (no single analog) |
| 6 | `backend/crates/marionette/src/validation.rs` new OR `error.rs` extension: `validation_error_patch()` | builder-helper | patch-shaping | `handlers/contact.rs:1025-1041` (`nav_active_patch` helper) | exact (same shape, different path template) |
| 7 | `backend/crates/crm-demo/src/handlers/company.rs:190-215, 318-330` (rewrite) | handler (form + inline) | request-response | `handlers/contact.rs:519-670` | exact (multi-field) + `contact.rs:715-731` (inline note-add) |
| 8 | `backend/crates/crm-demo/src/handlers/user.rs:217-260` (rewrite) | handler (form) | request-response | `handlers/contact.rs:519-670` | exact (2 FieldSets + separator) |
| 9 | `backend/crates/crm-demo/src/handlers/interaction.rs:63-109` (rewrite) | handler (form) | request-response | `handlers/contact.rs:519-670` + RadioGroup recipe from `15-RESEARCH.md §Example 2` | exact + primitive swap |
| 10 | `backend/crates/crm-demo/src/handlers/contact.rs:716-760` (rewrite inline forms) | handler (inline forms) | request-response | `handlers/contact.rs:716-731` (self, existing shape to mirror minimally) | exact |
| 11 | `backend/crates/crm-demo/src/handlers/contact.rs:1044-1186` (save-handler validation) | handler (save) | mutation + validation | `handlers/contact.rs:1044-1066` (current BadPayload branches) | role-match (replace `Err(BadPayload)` with `Ok(vec![validation_error_patch(...)])`) |
| 12 | `backend/crates/crm-demo/src/handlers/contact.rs:1577-1584` (Component literal → Button builder) | handler (toast) | event-driven | Any `Button::new(...)...build()` call in same file (e.g., `contact.rs:638-642`) | exact |
| 13 | `frontend/src/lib/init.ts:92-102` (edit) | frontend-setup | conditional-compile | `frontend/src/lib/components/core/FallbackComponent.svelte:12,18` | role-match (conditional gate) |
| 14 | `frontend/src/lib/components/form/Form.svelte:26-31` (edit) | component | event-handler | `frontend/src/lib/components/form/TextInput.svelte` (reads form values via `bind` → `getData`) + in-browser `FormData` collection | partial (no exact analog — see §Shared Patterns) |
| 15 | `frontend/tests/helpers/schema-validator.ts:1-6` (edit) | test-helper | file-I/O | `frontend/tests/e2e/ci-guards.spec.ts:20-26` already uses `node:*` imports under `@ts-expect-error`; target cleans those same imports | exact |
| 16 | `frontend/tests/e2e/ci-guards.spec.ts` (extend + drop `@ts-expect-error`) | CI guard | file-I/O | same file (existing TableScreen block, lines 31-47) | exact |
| 17 | `frontend/tests/e2e/company-edit.spec.ts` (new) | E2E spec | request-response | `frontend/tests/e2e/contact-edit.spec.ts` | exact |
| 18 | `frontend/tests/e2e/user-edit.spec.ts` (new) | E2E spec | request-response | `frontend/tests/e2e/contact-edit.spec.ts` | exact |
| 19 | `frontend/tests/e2e/interaction-edit.spec.ts` (new) | E2E spec | request-response | `frontend/tests/e2e/contact-edit.spec.ts` | exact |
| 20 | `frontend/tests/uat/{company,user,interaction}-edit-uat.spec.ts` (new) | UAT spec | request-response + file-I/O | `frontend/tests/uat/uat-driver.spec.ts` | exact |
| 21 | `frontend/tests/visual/form.spec.ts` (extend — 3 × 2 = 6 new snapshots) | visual spec | visual-snapshot | same file (lines 59-79) | exact |
| 22 | `spec/PROTOCOL.md` (delete ll.803-819; add worked example near l.600) | doc (protocol) | markdown | `spec/PROTOCOL.md:593-600` (canonical section to preserve) | role-match (doc surgery) |
| 23 | `CONCEPT.md` ll.260, 268, 630 (edit) | doc | markdown | n/a — prose edit | no analog (text replacement) |
| 24 | `TOOLING.md:39` (edit) | doc | markdown | n/a — single-line swap | no analog |
| 25 | `.planning/codebase/STACK.md:47` (edit) | doc (governance) | markdown | n/a — single-line swap | no analog |
| 26 | `.planning/phases/15-.../15-uat-evidence/{screen}/` (new folders + artifacts) | evidence | file-I/O | `.planning/phases/14-formscreen-enhancements/14-uat-evidence/` | exact |

(Item numbering is continuous; 25 "target" files plus the UAT evidence tree = 26 entries.)

---

## Pattern Assignments

### 1. `backend/crates/crm-demo/src/migration/m20260418_000011_extend_contact.rs` (migration, schema-change)

**Analog:** `backend/crates/crm-demo/src/migration/m20260323_000004_create_contact.rs` (full file, 40 lines)

**Imports pattern** (line 1):
```rust
use sea_orm_migration::prelude::*;
```

**Struct + MigrationName pattern** (lines 3-9):
```rust
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &'static str {
        "m20260323_000004_create_contact"
    }
}
```

**Up/Down `execute_unprepared` pattern** (lines 11-38):
```rust
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE TABLE contact ( … )",
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("DROP TABLE IF EXISTS contact")
            .await?;
        Ok(())
    }
}
```

**Delta:** Replace the single `CREATE TABLE` in `up` with three `ALTER TABLE contact ADD COLUMN …` calls (one per column: `contact_country TEXT`, `contact_notes TEXT`, `contact_opt_in INTEGER NOT NULL DEFAULT 0`). Replace the single `DROP TABLE` in `down` with three `ALTER TABLE contact DROP COLUMN …` in reverse order. Rename the migration name string to `"m20260418_000011_extend_contact"`. Issue one `execute_unprepared` per statement (see 15-RESEARCH.md Pitfall #5) — not a single semicolon-joined SQL string.

---

### 2. `backend/crates/crm-demo/src/migration/mod.rs` (edit — register migration)

**Analog:** same file, lines 3-12 + 20-29 (existing registrations).

**`mod` declaration pattern** (lines 3-12):
```rust
mod m20260323_000001_create_user;
mod m20260323_000002_create_audit_log;
mod m20260323_000003_create_company;
mod m20260323_000004_create_contact;
mod m20260323_000005_create_note;
mod m20260323_000006_create_tag;
mod m20260323_000007_create_contact_tag;
mod m20260323_000008_create_interaction;
mod m20260323_000009_create_listmonk_sync;
mod m20260323_000010_create_listmonk_cache;
```

**`migrations()` vec pattern** (lines 18-30):
```rust
fn migrations() -> Vec<Box<dyn MigrationTrait>> {
    vec![
        Box::new(m20260323_000001_create_user::Migration),
        // … 9 more …
        Box::new(m20260323_000010_create_listmonk_cache::Migration),
    ]
}
```

**Delta:** Append one line to the `mod` block (`mod m20260418_000011_extend_contact;`) and one more `Box::new(...)` to the `vec![]` body. Keep file idents alphabetical by timestamp (the new one sorts last naturally).

---

### 3. `backend/crates/crm-demo/src/entities/contact.rs` (edit — extend Model)

**Analog:** same file, lines 1-16 (existing Model struct).

**Imports + derive pattern** (lines 1-6):
```rust
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "contact")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub contact_id: i32,
    pub contact_name: String,
    pub contact_email: String,
    pub contact_phone: Option<String>,
    pub contact_title: Option<String>,
    pub contact_company: Option<i32>,
    pub contact_created_at: String,
    pub contact_updated_at: String,
}
```

**Relation pattern** (lines 18-33): keep untouched.

**Delta:** Insert three new `pub` fields between `contact_company: Option<i32>` (line 13) and `contact_created_at: String` (line 14):
```rust
pub contact_country: Option<String>,   // NEW — nullable TEXT
pub contact_notes: Option<String>,     // NEW — nullable TEXT
pub contact_opt_in: bool,              // NEW — INTEGER NOT NULL DEFAULT 0
```
Use `bool` (not `Option<bool>`) to match the `NOT NULL DEFAULT 0` column; SeaORM serialises `bool` → `INTEGER`. Per 15-RESEARCH.md Pitfall #4.

---

### 4. `backend/crates/crm-demo/src/seed.rs` (edit — populate new columns)

**Analog:** same file, lines 120-134 (existing `named_contacts` + ActiveModel loop) and lines 147-174 (generated-contact loop).

**Current named-contacts loop** (lines 120-134):
```rust
if needs_named_seed {
    for (name, email, phone, title, company_id) in named_contacts {
        let model = contact::ActiveModel {
            contact_id: NotSet,
            contact_name: Set(name.into()),
            contact_email: Set(email.into()),
            contact_phone: Set(phone.map(String::from)),
            contact_title: Set(title.map(String::from)),
            contact_company: Set(company_id),
            contact_created_at: NotSet,
            contact_updated_at: NotSet,
        };
        model.insert(db).await?;
    }
}
```

**Current generated-contact loop** (lines 148-173): same struct shape, different tuple source.

**Delta:** Extend the `named_contacts` tuple type to include `country: Option<&str>, notes: Option<&str>, opt_in: bool` (3 extra fields); add the three new `Set(...)` lines to both ActiveModel constructions (named + generated). Generated contacts can stay `None/None/false` to keep the seed small; named contacts should spread realistically (Alice: `Some("CH"), Some("Interested in Q2…"), true`; Bob: `Some("US"), None, false`; Carol: `None, Some("…contract work…"), true`) — matches 15-RESEARCH.md Example 5.

---

### 5. `backend/crates/marionette/src/builders/standard.rs` — new `form_shell()` helper (D-B1)

**Analog (no single exact match; two complementary references):**

**Reference A — Current handler-inline envelope pattern** (from `backend/crates/crm-demo/src/handlers/contact.rs:664-674`, the shape `form_shell()` will compress):
```rust
let mut all_nodes = Vec::new();
let mut extra_descendants: Vec<(String, marionette_protocol::Component)> = Vec::new();
all_nodes.push(heading);
all_nodes.push(back_button);
all_nodes.push(form_child);
extra_descendants.extend(contact_info_descendants);
extra_descendants.extend(organisation_descendants);
extra_descendants.extend(preferences_descendants);
extra_descendants.extend(action_row_descendants);
extra_descendants.extend(form_descendants);
```

**Reference B — `build_with_children` shape and test harness** (`backend/crates/marionette/src/builders/standard.rs:916-950`):
```rust
#[test]
fn container_builder_with_children() {
    let heading = Heading::new("Title").id("heading-1").build();
    let nodes = Container::new()
        .child(heading)
        .build_with_children();

    // Should contain container + heading
    assert_eq!(nodes.len(), 2);
    let (container_id, container) = &nodes[0];
    assert!(!container_id.is_empty());
    assert_eq!(container.r#type, "container");
    assert_eq!(
        container.children.as_ref().unwrap(),
        &["heading-1".to_string()]
    );

    let (heading_id, heading) = &nodes[1];
    assert_eq!(heading_id, "heading-1");
    assert_eq!(heading.r#type, "heading");
}
```

**Reference C — Module layout** (`backend/crates/marionette/src/builders/mod.rs`): `pub mod standard;` + `pub use standard::*;` — so a new `pub fn form_shell(...)` added at the bottom of `standard.rs` (below `impl TableColumn`, before `#[cfg(test)]`) is re-exported automatically from `marionette::builders`.

**Delta:** Add a free function `pub fn form_shell(root_id, heading, back_button, form_child, form_descendants) -> (String, HashMap<String, Component>)` (positional, per CONTEXT D-I preference). Internally: construct an outer `Container::new().id(&root_id).children(vec![heading.1, back_button.1, form_child.1]).build_with_children()`, then fold the returned Vec plus `form_descendants` into a `HashMap<String, Component>` and return `(root_id, nodes)`. Signature + body per 15-RESEARCH.md §Pattern 2 (lines 311-362). Add a unit test inside the existing `#[cfg(test)] mod tests` block, mirroring the style of `container_builder_with_children` (lines 916-935).

---

### 6. `backend/crates/marionette/src/validation.rs` (new file) OR `error.rs` extension — `validation_error_patch()`

**Analog:** `backend/crates/crm-demo/src/handlers/contact.rs:1025-1041` (`nav_active_patch` helper — same shape, same return type, different path template).

**Exact pattern to mirror** (lines 1025-1041):
```rust
fn nav_active_patch(active_slug: &str) -> marionette_protocol::ProtocolMessage {
    use marionette_protocol::data::PatchOperation;
    use marionette_protocol::messages::PatchMessage;
    let slugs = ["home", "contacts", "companies", "users", "audit"];
    let ops: Vec<PatchOperation> = slugs
        .iter()
        .map(|s| PatchOperation::Set {
            path: format!("/nav/active/{s}"),
            value: serde_json::json!(*s == active_slug),
        })
        .collect();
    marionette_protocol::ProtocolMessage::Patch(PatchMessage {
        id: None,
        surface: "main".into(),
        patch: ops,
    })
}
```

**Module registration analog** (`backend/crates/marionette/src/lib.rs:4-13`):
```rust
pub mod auth;
pub mod builders;
pub mod db;
pub mod error;
pub mod extractors;
pub mod migration;
pub mod router;
pub mod session;
pub mod ws;
```

**Delta:** Create `backend/crates/marionette/src/validation.rs` (new module) exposing `pub fn validation_error_patch<I, B, M>(surface, errors) -> ProtocolMessage` per 15-RESEARCH.md §Pattern 3 (lines 419-441). Key differences from `nav_active_patch`: (a) iterator input instead of single slug, (b) path format `"/_errors{bind}"` (not `/nav/active/...`), (c) `value: serde_json::Value::String(msg.into())` (not `json!(bool)`), (d) `surface` is a parameter (not hard-coded `"main"`), (e) `id: None` kept — `ws.rs::propagate_id` fills it. Register in `lib.rs` with `pub mod validation;` after line 11, matching the existing alphabetical-ish module list. Export nothing into the `pub use` block (callers import as `marionette::validation::validation_error_patch`).

**Alternative location (per CONTEXT D-D3 says "error.rs or validation.rs"):** appending to `error.rs` works too, but a dedicated module keeps `ActionError` purely about the error channel (D-D3 "no new error variant"). Prefer the new file.

---

### 7. `backend/crates/crm-demo/src/handlers/company.rs:190-215, 318-330` (rewrite — edit form + inline note-add)

**Analog A (main edit form):** `backend/crates/crm-demo/src/handlers/contact.rs:519-673` (Phase 14 canonical composition).

**Key excerpt — 3-FieldSet + separator + action-row + Form envelope** (contact.rs:519-673, condensed):
```rust
// -- FieldSet 1: Contact information --
let name_input = TextInput::new("Name")
    .id("contact-form-name")
    .bind("/contactForm/name")
    .required(true)
    .build();
let email_input = TextInput::new("Email")
    .id("contact-form-email")
    .bind("/contactForm/email")
    .input_type("email")
    .description("We will never share your email.")   // D-E3
    .build();
// … phone_input, title_input …

let (contact_info_set, contact_info_descendants) = FieldSet::new()
    .id("contact-info-set")
    .legend("Contact information")
    .children(vec![name_input, email_input, phone_input, title_input])
    .build_tree();

let separator_1 = FieldSeparator::new().id("contact-form-separator-1").build();

// … FieldSet 2 + separator_2 + FieldSet 3 …

let cancel_button = Button::new("Cancel")
    .id("contact-form-cancel")
    .variant("outline")
    .action(ComponentAction::click("contact_list"))
    .build();
let save_button = Button::new("Save contact")
    .id("contact-form-save")
    .variant("default")
    .action(ComponentAction::submit("contact_save"))
    .build();
let (action_row, action_row_descendants) = Container::new()
    .id("contact-form-actions")
    .class("flex gap-2 justify-end")
    .children(vec![cancel_button, save_button])
    .build_tree();

let (form_child, form_descendants) = Form::new()
    .id("contact-form")
    .children(vec![
        contact_info_set,
        separator_1,
        organisation_set,
        separator_2,
        preferences_set,
        action_row,
    ])
    .build_tree();
```

**Analog B (inline note-add, existing in same company.rs):** `company.rs:317-333` (current shape — stays structurally but migrates to go through the new `form_shell` / no changes to composition beyond D-E3 description if desired). The inline form already uses `Form::new().children([note_input, note_submit]).build_tree()` which is exactly the Phase 15 target shape — so company.rs:317-333 is the reference for both "what stays" and "this already matches."

**Delta for company edit form (190-215):**
- Fields: `name`, `website`, `address` → single `FieldSet("Company details")` per 15-UI-SPEC §Per-Screen (≤4 fields rule).
- `name_input` gains `.required(true).description("Will appear on invoices and contact details.")` per 15-UI-SPEC §Description Copy Contract.
- Add `cancel_button` (`variant("outline")`) and rename `save_button` label from `"Save"` to `"Save company"` per 15-UI-SPEC §Copywriting.
- Wrap in `Container class="flex gap-2 justify-end"` action row instead of the current 5-children flat `Form`.
- Replace the manual `Container::new().id("…-root").children(...).build_with_children()` pattern (see user.rs:272-286 for the current handler-inline pattern) with a call to `form_shell("company-form-root", heading, back_button, form_child, form_descendants)` from D-B1.
- Add a `back_button` (`variant("outline"); action: click("company_list")`) — today company has no back button.

**Delta for inline note-add form (318-330):** minimal — stays single-field inline `Form` but must emit per-field `/_errors/noteForm/text` patches on validation failure (D-D1) via the D-D3 helper, and wire the error path in the TextInput via the `bind` that's already there. No compositional change needed.

---

### 8. `backend/crates/crm-demo/src/handlers/user.rs:217-260` (rewrite — edit form)

**Analog:** `backend/crates/crm-demo/src/handlers/contact.rs:519-673` (same canonical composition). Plus `user.rs:272-286` (the current handler-inline `Container::build_with_children` manual wiring — to be REPLACED by `form_shell()`).

**Current handler-inline wiring to REPLACE** (user.rs:272-286):
```rust
let all_nodes = vec![heading, form_child];

let container_nodes = Container::new()
    .id("user-form-root")
    .children(all_nodes)
    .build_with_children();

let mut nodes = HashMap::new();
for (id, component) in container_nodes {
    nodes.insert(id, component);
}
for (id, component) in form_descendants {
    nodes.insert(id, component);
}
```

**Delta:**
- 5 fields → two FieldSets per 15-UI-SPEC §Per-Screen: `FieldSet("Account", [name, email, password])` + `FieldSeparator` + `FieldSet("Permissions", [role, preferred_contact_method])`.
- `email_input` gains `.description("Used for password resets and notifications.")` per 15-UI-SPEC.
- `preferred_contact_method` is a `RadioGroup` with 3 options (`email/sms/phone`), each carrying a per-option `description` per 15-UI-SPEC Description Copy Contract. Handler accepts field via `#[allow(dead_code)]` + `#[serde(default)]` in the payload struct (per CONTEXT D-E2).
- Action row same shape as company (Cancel + "Save user") — use `Container class="flex gap-2 justify-end"`.
- Replace the manual `Container::new().id("user-form-root")...build_with_children()` block (lines 272-286) with `form_shell("user-form-root", heading, back_button, form_child, form_descendants)` returning `(root, nodes)` directly.
- Add a `back_button` (click "user_list") — today user has no back button.

**RadioGroup backend builder call** (per 15-RESEARCH.md lines 747-757, adapted):
```rust
let preferred_radio = RadioGroup::new("Preferred contact method")
    .id("user-form-preferred-contact-method")
    .bind("/userForm/preferred_contact_method")
    .options(vec![
        RadioOption { value: "email".into(), label: "Email".into(),
                      description: Some("Receive updates by email.".into()) },
        RadioOption { value: "sms".into(),   label: "SMS".into(),
                      description: Some("Text messages to your phone.".into()) },
        RadioOption { value: "phone".into(), label: "Phone".into(),
                      description: Some("A human will call you.".into()) },
    ])
    .build();
```

---

### 9. `backend/crates/crm-demo/src/handlers/interaction.rs:63-109` (rewrite — edit form + RadioGroup)

**Analog A:** `backend/crates/crm-demo/src/handlers/contact.rs:519-673` (Phase 14 canonical composition).

**Analog B (current Select-to-swap):** `interaction.rs:63-82` — the current `type_select` construction:
```rust
let type_select = Select::new(
    "Type",
    vec![
        SelectOption { value: "call".into(),    label: "Call".into() },
        SelectOption { value: "email".into(),   label: "Email".into() },
        SelectOption { value: "meeting".into(), label: "Meeting".into() },
    ],
)
.id("interaction-type")
.bind("/interactionForm/interaction_type")
.build();
```

**Analog C (current handler-inline wiring to REPLACE):** `interaction.rs:121-134` — same `Container::new().id("interaction-form-root").children(all_nodes).build_with_children()` pattern as user.rs:272-286.

**Delta:**
- 4 fields → single `FieldSet("Interaction", [type, subject, date, notes])` per 15-UI-SPEC.
- **`type_select` → `RadioGroup`** per D-E1: options `[("call","Call"), ("email","Email"), ("meeting","Meeting")]`; NO per-option descriptions per 15-UI-SPEC (labels are self-explanatory).
- **`notes_input` upgrades from `TextInput` → `Textarea::new("Notes").rows(4u32).full_width(true).build()`** per 15-UI-SPEC §Textarea full_width.
- **`date_input`** gains `.input_type("datetime-local").description("Format: YYYY-MM-DD HH:MM (24-hour).")` per 15-UI-SPEC Description Copy Contract.
- Label copy: form renders "Log Interaction" heading unchanged; Save button renames to "Save interaction".
- Action row (`flex gap-2 justify-end`) with Cancel ("outline") + "Save interaction".
- Replace the manual `Container::new()...build_with_children()` (lines 121-134) with `form_shell("interaction-form-root", heading, back_button, form_child, form_descendants)`.
- Add back_button (click "contact_list", or wherever interaction cancel lands today per line 104-107).

---

### 10. `backend/crates/crm-demo/src/handlers/contact.rs:716-760` (rewrite inline tag-add + note-add)

**Analog (current shape, to preserve):** `contact.rs:715-731` + `contact.rs:747-763` — already correct structure (single-field Form). Only error-path wiring changes.

**Current tag-add form** (contact.rs:715-731):
```rust
let tag_input = TextInput::new("Add tag...")
    .id("tag-input")
    .bind("/tagForm/name")
    .build();

let tag_submit = Button::new("Add Tag")
    .id("tag-add")
    .action(ComponentAction::submit("contact_tag_save"))
    .build();

let (tag_form_child, tag_form_descendants) = Form::new()
    .id("tag-form")
    .children(vec![tag_input, tag_submit])
    .build_tree();
all_nodes.push(tag_form_child);
extra_descendants.extend(tag_form_descendants);
```

**Delta:**
- Keep the single-field `Form` shape (NO FieldSet — inline forms don't need grouping per 15-UI-SPEC §5/6).
- Rename button labels: "Add Tag" → "+ Add tag"; "Add Note" → "+ Add note" per 15-UI-SPEC §Copywriting.
- Layout wrapper: wrap each form in `Container::new().class("flex gap-2 items-end")` (tag) or `Container::new().class("flex flex-col gap-2 items-end")` (note) per 15-UI-SPEC §5/§6.
- The corresponding `handle_contact_tag_save` and `handle_note_save` handlers emit `/_errors/tagForm/name` / `/_errors/noteForm/text` patches on invalid input via `validation_error_patch("content", …)` (D-D1).

---

### 11. `backend/crates/crm-demo/src/handlers/contact.rs:1044-1186` (save handler — replace `BadPayload` with per-field patches)

**Analog (current `Err(BadPayload)` pattern to REPLACE):** `contact.rs:1052-1065`:
```rust
// Validate required fields
if data.name.trim().is_empty() {
    return Err(ActionError::BadPayload(
        "Contact name is required".into(),
    ));
}
if data.email.trim().is_empty() {
    return Err(ActionError::BadPayload("Email is required".into()));
}
if !data.email.contains('@') {
    return Err(ActionError::BadPayload(
        "Invalid email format".into(),
    ));
}
```

**Delta:** Replace the three `return Err(ActionError::BadPayload(...))` branches with a single `errors` Vec collected top-to-bottom, ending in:
```rust
if !errors.is_empty() {
    return Ok(vec![
        marionette::validation::validation_error_patch("content", errors),
    ]);
}
```
Full target shape per 15-RESEARCH.md §Example 3 (lines 790-839). Field-path convention: `/contactForm/{field}` matches the existing `.bind()` on each TextInput (the helper prefixes `/_errors`). **Per-field order must match FORM field-display order** per 15-RESEARCH.md Pitfall #1.

**Also in the same save handler:** wire `Set(data.country)`, `Set(data.notes)`, `Set(data.opt_in.unwrap_or(false))` into the ActiveModel construction (both insert and update paths) — per CONTEXT D-C3. The payload struct `ContactFormData` drops its `#[serde(default)] #[allow(dead_code)]` attributes on country/notes/opt_in.

**Apply the same pattern to:** `handle_company_save`, `handle_user_save`, `handle_interaction_save` — each gets its own error path (`/companyForm/name`, `/userForm/email`, `/interactionForm/subject`, etc.).

---

### 12. `backend/crates/crm-demo/src/handlers/contact.rs:1577-1584` (replace hand-rolled Component literal with Button builder)

**Analog (target idiom):** any `Button::new(...)...build()` call in the same file, e.g., `contact.rs:638-642`:
```rust
let save_button = Button::new("Save contact")
    .id("contact-form-save")
    .variant("default")
    .action(ComponentAction::submit("contact_save"))
    .build();
```

**Current anti-pattern to REPLACE** (contact.rs:1577-1584):
```rust
let toast_node = Component {
    r#type: "button".into(),
    props: Some(serde_json::Value::Object(toast_props)),
    children: None,
    bind: None,
    action: Some(ComponentAction::click("dismiss_toast")),
    visible: None,
};
```

**Delta:** Replace the 7-line `Component { ... }` struct literal with a 4-line builder chain:
```rust
let (_toast_id, toast_node) = Button::new(&toast_label)
    .id("toast-country-change")   // match whatever the Patch op references
    .action(ComponentAction::click("dismiss_toast"))
    .build();
```
Then drop the surrounding `toast_props` Map construction (lines 1575-1576) since `Button::new(label)` sets the label prop automatically. The patch that references this `toast_node` still expects an id — use the existing toast id from the `RemoveChild { child_id: "toast-country-change" }` above (line 1588).

---

### 13. `frontend/src/lib/init.ts:92-102` (dev-gate `__mrnSetData` + `__mrnSendAction`)

**Analog (repo's existing `import.meta.env.DEV` gate):** `frontend/src/lib/components/core/FallbackComponent.svelte:11-23`:
```svelte
$effect(() => {
    if (!import.meta.env.DEV) {
        console.warn('Unknown component type:', nodeType, 'on surface:', surface);
    }
});
</script>

{#if import.meta.env.DEV}
    <div class="border-2 border-dashed border-destructive bg-destructive/10 p-4 rounded-md">
        ...
    </div>
{/if}
```

**Current unconditional pattern to REPLACE** (init.ts:92-102):
```typescript
if (typeof window !== 'undefined') {
    (window as unknown as { __mrnSendAction: typeof sendAction }).__mrnSendAction = sendAction;
    ...
    (window as unknown as { __mrnSetData: typeof setData }).__mrnSetData = setData;
}
```

**Delta:** Change the outer guard from `if (typeof window !== 'undefined')` to `if (typeof window !== 'undefined' && import.meta.env.DEV)` — keep both hook assignments inside the same block. Exactly one gate level; do NOT gate each hook separately. Vite tree-shakes the entire `if` at build time when `DEV` is `false`. Per 15-RESEARCH.md Example 7.

---

### 14. `frontend/src/lib/components/form/Form.svelte:26-31` (fix empty-payload sendAction)

**Analog (partial — how other form components collect values):** The current `Form.svelte:22-24` reads `formErrors` via `getData(surface, '/_errors' + bind)`. Other form leaf components (e.g., `TextInput.svelte` lines 26-30, `RadioGroup.svelte`, `Switch.svelte` — per 15-UI-SPEC §Interaction Contracts) set values via `setData(surface, bind, value)` on input. The form boundary itself does NOT currently collect values — each leaf writes directly to the surface data store.

**Current pattern to FIX** (Form.svelte:26-31):
```typescript
function handleSubmit(e: SubmitEvent) {
    e.preventDefault();
    if (action) {
        sendAction(action.name ?? 'submit', {}, action.target);
    }
}
```

**Delta:** Two acceptable paths per CONTEXT D-G2:
- **(a) preferred:** Read the form's bound data from the store and pass it as payload. If the Form has `bind`, use `getData(surface, bind)` to get the form subtree and pass that Object as the second `sendAction` argument. If no `bind`, fall back to no-op (but keep the dispatch gated).
- **(b) fallback:** Remove the empty-payload dispatch entirely — no handler currently wires `Form.action`; Save flows dispatch via Button `action={submit}` instead.

Planner picks (a) per CONTEXT preference. See 15-UI-SPEC §Interaction Contracts: "Form submit now sends collected form values, not `{}`."

---

### 15. `frontend/tests/helpers/schema-validator.ts:1-6` (switch to `node:` prefix imports)

**Analog:** `frontend/tests/e2e/ci-guards.spec.ts:20-26`:
```typescript
import { test, expect } from '@playwright/test';
// @ts-expect-error — node:fs resolves under Playwright runtime but not in svelte-check
import { existsSync } from 'node:fs';
// @ts-expect-error — node:path resolves under Playwright runtime but not in svelte-check
import { resolve } from 'node:path';
// @ts-expect-error — node:url resolves under Playwright runtime but not in svelte-check
import { fileURLToPath } from 'node:url';
```

**Current pattern to FIX** (schema-validator.ts:1-6):
```typescript
import Ajv from 'ajv';
import addFormats from 'ajv-formats';
import * as yaml from 'js-yaml';
import * as fs from 'fs';
import * as path from 'path';
import { fileURLToPath } from 'url';
```

**Delta:** Rewrite lines 4-6 to:
```typescript
import * as fs from 'node:fs';
import * as path from 'node:path';
import { fileURLToPath } from 'node:url';
```
Node 18+ resolves `node:` prefix natively (per CONTEXT D-G4). After this change, **also remove the `@ts-expect-error` comments on ci-guards.spec.ts lines 21, 23, 25** — same underlying fix. Per 15-RESEARCH.md D-G4.

---

### 16. `frontend/tests/e2e/ci-guards.spec.ts` (extend with Flowbite grep + drop `@ts-expect-error`)

**Analog (existing TableScreen + FormScreen deletion guard):** `ci-guards.spec.ts:31-47`:
```typescript
test.describe('Phase 13 CI guards', () => {
    test('TableScreen.svelte is retired (D-A2)', () => {
        const p = resolve(
            FRONTEND_ROOT,
            'src/lib/components/screen/TableScreen.svelte',
        );
        expect(existsSync(p)).toBe(false);
    });

    test('TableScreen.browser-test.ts is retired (D-A2)', () => {
        const p = resolve(
            FRONTEND_ROOT,
            'src/lib/components/screen/TableScreen.browser-test.ts',
        );
        expect(existsSync(p)).toBe(false);
    });
});
```

**Delta:** Extend with a new test block per 15-RESEARCH.md Example 6 (lines 955-1003):
- Drop the three `@ts-expect-error` suppressions (lines 21, 23, 25) after D-G4.
- Add a `FormScreen.svelte` deletion guard block mirroring TableScreen.svelte (Phase 14 D-A1 — missing from the current file).
- Add a `No Flowbite residue in runtime code (Phase 15 D-F1)` block that shells `git grep -Iil 'flowbite' -- 'frontend/src/**' 'backend/crates/**' 'spec/**' CONCEPT.md TOOLING.md` via `child_process.execSync`. Use the `{ cwd: REPO_ROOT }` option. On exit code 1 (no matches) → `matches = []` (success). Any non-zero non-1 exit → rethrow. Expect `matches.length === 0`.
- `REPO_ROOT` const: `resolve(FRONTEND_ROOT, '..')` — one level up from the frontend root.

---

### 17-19. `frontend/tests/e2e/{company,user,interaction}-edit.spec.ts` (new E2E specs)

**Analog:** `frontend/tests/e2e/contact-edit.spec.ts` (full file — lines 1-80+ cited).

**Key excerpts:**

**Imports + login helper** (contact-edit.spec.ts:1-40):
```typescript
import { test, expect, type Page } from '@playwright/test';

async function login(page: Page): Promise<void> {
    await page.goto('/');
    const emailInput = page
        .locator('div[data-slot="field"]:has(label:has-text("Email"))')
        .locator('input')
        .first();
    const passwordInput = page
        .locator('div[data-slot="field"]:has(label:has-text("Password"))')
        .locator('input')
        .first();
    await emailInput.fill('admin@localhost');
    await passwordInput.fill('admin');
    await page.getByRole('button', { name: /log in/i }).click();
    await expect(page.getByText('Contact Management')).toBeVisible({ timeout: 10000 });
}
```

**`__mrnSendAction` hook pattern** (contact-edit.spec.ts:42-62):
```typescript
async function openEditContactForm(page: Page): Promise<void> {
    await page.evaluate(() => {
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        const hook = (window as any).__mrnSendAction as
            | ((name: string, payload?: Record<string, unknown>, source?: string) => void)
            | undefined;
        if (!hook) throw new Error('__mrnSendAction hook missing');
        hook('contact_edit', { contact_id: 1 }, 'contact-edit-1');
    });
    await expect(page.getByRole('heading', { name: 'Edit Contact' })).toBeVisible({
        timeout: 5000,
    });
}
```

**Test block shape** (contact-edit.spec.ts:71-80):
```typescript
test.describe('Contact edit form (Phase 14 Plan 08)', () => {
    test('renders FieldSet legends and action-row buttons (D-C1, D-D1)', async ({ page }) => {
        await login(page);
        await openEditContactForm(page);
        await expect(page.getByText('Contact information')).toBeVisible();
        await expect(page.getByText('Organisation')).toBeVisible();
        await expect(page.getByText('Notes and preferences')).toBeVisible();
```

**Delta per spec:**
- **company-edit.spec.ts:** Land screen via `hook('company_edit', { company_id: 1 }, 'company-edit-1')`. Heading: `'Edit Company'`. Legend: `'Company details'`. Action buttons: `'Cancel'`, `'Save company'`. Validation scenario clears `#company-form-name`, submits, expects a `[data-slot="field-error"]` with text matching `/required/i`.
- **user-edit.spec.ts:** Land via `hook('user_edit', { user_id: 1 }, 'user-edit-1')`. Legends: `'Account'` and `'Permissions'`. Action: `'Save user'`. Additional scenario: click each `preferred_contact_method` RadioGroup option and confirm `data-state="checked"` migrates.
- **interaction-edit.spec.ts:** Land via `hook('interaction_form', { contact_id: 1 }, 'interaction-form-1')` (or `interaction_edit` if such action is added). Legend: `'Interaction'`. Radio options: `Call`, `Email`, `Meeting`. Action: `'Save interaction'`. Additional scenario: `Textarea` `notes` full-width grid-column check (see uat-driver.spec.ts:106-118 for the grid-column inspection pattern).

---

### 20. `frontend/tests/uat/{company,user,interaction}-edit-uat.spec.ts` (new UAT specs)

**Analog:** `frontend/tests/uat/uat-driver.spec.ts:1-120` (Phase 14 Plan 08 UAT driver).

**Imports + EVIDENCE_DIR pattern** (uat-driver.spec.ts:1-41):
```typescript
import { test, expect, type Page } from '@playwright/test';
// @ts-expect-error — node:fs resolves under Playwright runtime but not in svelte-check
import * as fs from 'node:fs';
// @ts-expect-error — node:path resolves under Playwright runtime but not in svelte-check
import * as path from 'node:path';
const cwd = (globalThis as { process?: { cwd(): string } }).process?.cwd() ?? '.';

const EVIDENCE_DIR = path.resolve(
    cwd,
    '..',
    '.planning/phases/14-formscreen-enhancements/14-uat-evidence',
);
if (!fs.existsSync(EVIDENCE_DIR)) {
    fs.mkdirSync(EVIDENCE_DIR, { recursive: true });
}

function artifactPath(name: string): string {
    return path.join(EVIDENCE_DIR, name);
}
```

**Scenario shape with screenshot + JSON evidence** (uat-driver.spec.ts:89-119):
```typescript
test.describe('Phase 14 Plan 08 — Human-verify UAT', () => {
    test('UAT-01 Responsive grid @ 375px + 1024px (FORM-02)', async ({ browser }) => {
        const desktopCtx = await browser.newContext({ viewport: { width: 1024, height: 800 } });
        const desktopPage = await desktopCtx.newPage();
        await login(desktopPage);
        await openEditForm(desktopPage);
        await desktopPage.screenshot({
            path: artifactPath('01-responsive-1024.png'),
            fullPage: true,
        });
        // … DOM assertion via page.evaluate() …
    });
```

**Delta:**
- Each new UAT spec path: `frontend/tests/uat/{screen}-edit-uat.spec.ts`. The three new UAT evidence roots: `15-uat-evidence/company-edit/`, `user-edit/`, `interaction-edit/`; plus `contact-tag-add/` and `contact-note-add/` (smoke) per CONTEXT D-H1.
- Rewrite `EVIDENCE_DIR` path to phase-15 folder: `'..', '.planning/phases/15-crm-migration-validation/15-uat-evidence/<screen>'`.
- 3-4 scenarios per screen per 15-UI-SPEC §UAT Evidence Contract §Scope: render → validation (empty name) → save → (optional RadioGroup click-through for user + interaction).
- Each scenario saves `{N}-name.png` (screenshot) + `{N}-name.json` (DOM assertion snapshot) into the per-screen evidence folder.
- Assertion JSON must include the keys defined in 15-UI-SPEC §UAT Evidence Contract §assertions.json Required Keys (screen, viewport, fieldset_legends, fields_in_order, action_row, descriptions_present, validation_trigger, console_errors).

---

### 21. `frontend/tests/visual/form.spec.ts` (extend — 6 new snapshot cases)

**Analog:** same file, lines 59-79 (existing contact-edit-form + mobile snapshot tests).

**Existing test block** (form.spec.ts:59-79):
```typescript
test('contact edit form — desktop baseline', async ({ page }) => {
    await loginDemo(page);
    await openEditForm(page);
    await page.waitForSelector('[data-slot="field"]', { state: 'visible' });
    await expect(page).toHaveScreenshot('contact-edit-form.png', {
        fullPage: true,
        maxDiffPixels: 200,
    });
});

test('contact edit form — mobile 375px baseline', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 800 });
    await loginDemo(page);
    await openEditForm(page);
    await page.waitForSelector('[data-slot="field"]', { state: 'visible' });
    await expect(page).toHaveScreenshot('contact-edit-form-mobile.png', {
        fullPage: true,
        maxDiffPixels: 200,
    });
});
```

**Delta:** Add 6 new test blocks (3 screens × 2 viewports) mirroring the contact patterns. For each screen: one `openCompanyEditForm(page)` / `openUserEditForm(page)` / `openInteractionEditForm(page)` helper using `__mrnSendAction` (same pattern as lines 41-57). Snapshot filenames: `company-edit-form.png`, `company-edit-form-mobile.png`, `user-edit-form.png`, `user-edit-form-mobile.png`, `interaction-edit-form.png`, `interaction-edit-form-mobile.png`. First run uses `--update-snapshots`; second run must be green. `maxDiffPixels: 200` same tolerance as contact — matches `playwright.uat.config.ts:22` precedent.

---

### 22. `spec/PROTOCOL.md` (delete ll.803-819; add worked example near l.600)

**Analog (canonical section to PRESERVE):** `spec/PROTOCOL.md:593-600`:
```markdown
### Validation semantics

Per-field and form-level errors both flow through the data store via JSON Pointer paths:

- **Per-field errors**: `/_errors/{bind}` holds a `string`. When non-empty, the bound field renders with `data-invalid` on the wrapper, `aria-invalid="true"` on the control, and the message in a `Field.Error` below (replacing any `description`).
- **Form-level errors**: `/_errors/{form_bind}` holds a `string[]`. When the array is non-empty, `Form.svelte` renders a banner above its children listing each message.

Servers clear errors by patching the path to an empty string / empty array. There is no client-side validation — every error message is server-authoritative and flows through the standard patch mechanism.
```

**Analog (legacy section to DELETE):** `spec/PROTOCOL.md:803-819` (the `### Validation Errors as Data` block with the `/contactForm/errors` example).

**Delta:** Delete PROTOCOL.md lines 803-819 wholesale (per CONTEXT D-D2 — no back-compat). Under the existing §Validation semantics section (after line 600), insert a new `#### Worked example: multi-field validation on form submit` subsection with the JSON payload per 15-RESEARCH.md §Example 8 (lines 1030-1060). Include the explicit statement: "The save handler that produced this patch returns `Ok(vec![patch])` — NOT `Err(ActionError::BadPayload)`." This mirrors the existing authoritative tone at line 600.

---

### 23. `CONCEPT.md` ll.260, 268, 630 (edit — Flowbite → shadcn-svelte)

**Analog:** n/a — straight prose edit.

**Current strings** (verified):
- Line 260: `"…web with Flowbite, mobile with native widgets, TV with remote-friendly controls."`
- Line 268: ASCII-table column label `(Flowbite)`
- Line 630: `"### Phase 2: Marionette Frontend (Svelte 5 + Flowbite)"`

**Delta:** Replace as per 15-UI-SPEC §Doc Brand-Voice Sweep table:
- L.260 → `"…web with shadcn-svelte, mobile with native widgets, TV with remote-friendly controls."`
- L.268 → `(shadcn-svelte)`
- L.630 → `"### Phase 2: Marionette Frontend (Svelte 5 + shadcn-svelte)"`
Keep "shadcn-svelte" lowercase. Optional historical footnote at end of CONCEPT.md per 15-UI-SPEC §Historical Footnote — planner's discretion.

---

### 24. `TOOLING.md:39` (edit — Flowbite → shadcn-svelte)

**Analog:** n/a — single-line swap.

**Current string** (line 39): `"- **Flowbite Svelte** - Tailwind CSS component library"`

**Delta:** Replace with `"- **shadcn-svelte** - Tailwind CSS + bits-ui component library"` per 15-UI-SPEC §Doc Brand-Voice Sweep.

---

### 25. `.planning/codebase/STACK.md:47` (edit — Flowbite → shadcn-svelte)

**Analog:** n/a — single-line swap.

**Current string** (line 47): `"- \`flowbite-svelte 1.31\` + \`flowbite-svelte-icons 3.1\` - UI component library"`

**Delta:** Replace with `"- \`shadcn-svelte 1.2.7\` + \`bits-ui 2.17.3\` + \`@lucide/svelte 1.8.0\` - UI component library"` per 15-UI-SPEC §Doc Brand-Voice Sweep (use the verified versions from 15-RESEARCH.md Standard Stack table).

---

### 26. `.planning/phases/15-.../15-uat-evidence/{screen}/` (new evidence tree)

**Analog:** `.planning/phases/14-formscreen-enhancements/14-uat-evidence/` (Phase 14 precedent). Folder layout per 15-UI-SPEC §UAT Evidence Contract:

```
14-uat-evidence/
├── 01-responsive-1024.png
├── 01-responsive-375.png
├── 02-field-description.png
├── 03-validation-error.png
├── ...
└── {N}-{scenario}.json
```

**Delta:** Create 5 subfolders under `15-uat-evidence/`:
- `company-edit/` — 3 scenarios × 2 artifacts = 6 files
- `user-edit/` — 3-4 scenarios × 2 artifacts
- `interaction-edit/` — 3-4 scenarios × 2 artifacts (RadioGroup + datetime)
- `contact-tag-add/` — 2 scenarios × 2 artifacts (smoke)
- `contact-note-add/` — 2 scenarios × 2 artifacts (smoke)

Each folder contains `.png` screenshots + `.json` assertion snapshots + `.log` console capture. Total ~28 files per 15-UI-SPEC §UAT Evidence Contract §Folder Layout.

---

## Shared Patterns

### SP-1. Phase 14 Canonical Form Composition (applies to every edit-form rewrite — items 7, 8, 9)

**Source:** `backend/crates/crm-demo/src/handlers/contact.rs:519-673`.

**Structure (verbatim for every migrated form, adapted to each screen's field list):**
```rust
// 1. Heading
let heading = Heading::new(form_title).id("{screen}-form-heading").build();

// 2. Back button (outline variant)
let back_button = Button::new("← Back")
    .id("{screen}-form-back")
    .variant("outline")
    .action(ComponentAction::click("{screen}_list"))
    .build();

// 3. Leaf inputs with .bind(...).required(...).description(...) as appropriate

// 4. FieldSet wrapping each semantic group
let (set, set_descendants) = FieldSet::new()
    .id("{screen}-{group}-set")
    .legend("{Legend Copy}")
    .children(vec![…fields…])
    .build_tree();

// 5. FieldSeparator between sibling FieldSets (explicit node)
let sep = FieldSeparator::new().id("{screen}-form-separator-1").build();

// 6. Action row — plain Container, flex gap-2 justify-end, Cancel + Save
let (action_row, action_desc) = Container::new()
    .id("{screen}-form-actions")
    .class("flex gap-2 justify-end")
    .children(vec![cancel_button, save_button])
    .build_tree();

// 7. Form envelope wrapping FieldSets + separators + action row
let (form_child, form_descendants) = Form::new()
    .id("{screen}-form")
    .children(vec![set_1, sep, set_2, action_row])
    .build_tree();

// 8. Outer shell — USE form_shell() (D-B1) once available
let (root, nodes) = form_shell(
    "{screen}-form-root",
    heading, back_button, form_child,
    form_descendants + set_descendants + action_desc (concatenated),
);
```

**Apply to:** company-edit, user-edit, interaction-edit, contact-edit refactor (D-B2).

---

### SP-2. Per-Field Validation Write-Path (applies to every save handler — items 7, 8, 9, 11 + inline forms in 10)

**Source:** `backend/crates/crm-demo/src/handlers/contact.rs:1025-1041` (`nav_active_patch` — shape reference) + 15-RESEARCH.md §Pattern 3 (helper signature).

**Structure:**
```rust
pub async fn handle_{screen}_save(ctx: HandlerContext) -> ActionResult {
    // 1. Extract payload
    let data = payload.0.{screen}_form;

    // 2. Collect errors IN FORM FIELD ORDER (top-to-bottom)
    let mut errors: Vec<(String, String)> = Vec::new();
    if data.name.trim().is_empty() {
        errors.push(("/{screen}Form/name".into(), "Name is required.".into()));
    }
    // … additional checks …

    // 3. If any errors, return patch (Ok, NOT Err)
    if !errors.is_empty() {
        return Ok(vec![
            marionette::validation::validation_error_patch("content", errors),
        ]);
    }

    // 4. Proceed with DB write + render_list re-render
}
```

**Error copy tone (per 15-UI-SPEC §Copywriting):**
- Sentence case. Ends with full stop.
- Actionable. "Name is required." not "Please enter a name."
- Specific. "Password must be at least 8 characters." not "Too short."

**Apply to:** `handle_contact_save` (replacing current BadPayload branches at lines 1053-1065), `handle_company_save`, `handle_user_save`, `handle_interaction_save`, `handle_contact_tag_save`, `handle_note_save`. All six handler save paths use the same helper; `ActionError::BadPayload` is reserved for JSON parse / missing form_bind / auth / DB failures (D-D4).

---

### SP-3. Authentication (applies to every E2E and UAT spec — items 17-20)

**Source:** `frontend/tests/e2e/contact-edit.spec.ts:22-40`.

**Structure:**
```typescript
async function login(page: Page): Promise<void> {
    await page.goto('/');
    const emailInput = page
        .locator('div[data-slot="field"]:has(label:has-text("Email"))')
        .locator('input')
        .first();
    const passwordInput = page
        .locator('div[data-slot="field"]:has(label:has-text("Password"))')
        .locator('input')
        .first();
    await emailInput.fill('admin@localhost');
    await passwordInput.fill('admin');
    await page.getByRole('button', { name: /log in/i }).click();
    await expect(page.getByText('Contact Management')).toBeVisible({ timeout: 10000 });
}
```

**Apply to:** every new E2E and UAT spec unchanged (credentials + landing-screen match are environment-invariant).

---

### SP-4. `__mrnSendAction` Hook Usage (applies to every E2E and UAT spec — items 17-20)

**Source:** `frontend/tests/e2e/contact-edit.spec.ts:42-62`.

**Structure:**
```typescript
async function open{Screen}Edit(page: Page): Promise<void> {
    await page.evaluate(() => {
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        const hook = (window as any).__mrnSendAction as
            | ((name: string, payload?: Record<string, unknown>, source?: string) => void)
            | undefined;
        if (!hook) throw new Error('__mrnSendAction hook missing');
        hook('{screen}_edit', { {screen}_id: 1 }, '{screen}-edit-1');
    });
    await expect(page.getByRole('heading', { name: 'Edit {Screen}' })).toBeVisible({
        timeout: 5000,
    });
}
```

**Apply to:** every new `*-edit.spec.ts` + UAT spec. Relies on the dev-gated `window.__mrnSendAction` (D-G1 — the hook is present in dev, tree-shaken in production, which is why E2E tests always run under `make dev`).

---

### SP-5. Validation Scenario — `[data-slot="field-error"]` Assertion (applies to every E2E validation test — items 17-19)

**Source:** implied by 15-UI-SPEC §Validation Write-Path Visual Contract + `frontend/src/lib/components/ui/field/field-error.svelte` contract. There's no existing E2E test that asserts on `data-slot="field-error"` — Phase 15 writes the first.

**Structure:**
```typescript
test('per-field validation renders inline on empty name (D-D1)', async ({ page }) => {
    await login(page);
    await open{Screen}Edit(page);
    await page.locator('#{screen}-form-name').fill('');
    await page.getByRole('button', { name: 'Save {screen}' }).click();
    const error = page.locator('[data-slot="field-error"]').filter({ hasText: /required/i });
    await expect(error).toBeVisible();
    // Form-level banner must NOT appear for per-field validation (D-D4)
    await expect(page.locator('.bg-destructive\\/10')).toHaveCount(0);
});
```

**Apply to:** company-edit, user-edit, interaction-edit, contact-edit (extend existing). The `Field.Error` slot is guaranteed by 15-UI-SPEC §Validation Write-Path Visual Contract #1 (rendered INSIDE `Field.Field` below description).

---

## No Analog Found

Files with no direct in-codebase precedent — these rely on synthesis from research or are pure-prose edits:

| Target | Reason |
|--------|--------|
| `backend/crates/marionette/src/builders/standard.rs` `form_shell()` helper | Synthesis: composes the inline pattern from `contact.rs:664-674` (handler-hand-rolled envelope) with the `build_with_children` mechanics from `standard.rs:916-935`. The helper itself is new but both input patterns are in-repo. |
| `frontend/src/lib/components/form/Form.svelte:26-31` | No exact analog in repo — no other form component currently collects + dispatches a payload to `sendAction`. Planner uses native `FormData` collection or a dedicated `$derived` value reading the form subtree from the data store (option (a) in 15-RESEARCH.md D-G2 discussion). |
| `CONCEPT.md`, `TOOLING.md`, `.planning/codebase/STACK.md` line edits | Pure prose; no code analog needed. 15-UI-SPEC §Doc Brand-Voice Sweep provides the exact target strings. |

All three items are low-risk: the `form_shell()` signature is specified in detail in 15-RESEARCH.md §Pattern 2 (40 lines); Form.svelte has two clear options with a recommendation; doc edits are three lines.

---

## Metadata

**Analog search scope:**
- `backend/crates/crm-demo/src/handlers/` (contact.rs, company.rs, user.rs, interaction.rs)
- `backend/crates/crm-demo/src/entities/contact.rs`
- `backend/crates/crm-demo/src/migration/` (m20260323_000004 + mod.rs)
- `backend/crates/crm-demo/src/seed.rs`
- `backend/crates/marionette/src/{lib,error,builders/{mod,standard}}.rs`
- `backend/crates/marionette-macros/src/component_builder.rs`
- `frontend/src/lib/init.ts`
- `frontend/src/lib/components/form/Form.svelte`
- `frontend/src/lib/components/core/FallbackComponent.svelte`
- `frontend/tests/e2e/{contact-edit,ci-guards}.spec.ts`
- `frontend/tests/uat/uat-driver.spec.ts`
- `frontend/tests/visual/form.spec.ts`
- `frontend/tests/helpers/schema-validator.ts`
- `spec/PROTOCOL.md` (validation sections)
- `CONCEPT.md`, `TOOLING.md`, `.planning/codebase/STACK.md` (Flowbite lines)

**Files scanned:** ~22 files directly; ~8 more via grep cross-reference.

**Pattern extraction date:** 2026-04-18.

**Guidance for planner:**
- **Prefer in-repo analogs over RESEARCH.md prose examples.** RESEARCH.md's `Example 1-9` code blocks are templates; this PATTERNS.md ties each one to the exact file + line range the executor should read while implementing.
- **`form_shell()` and `validation_error_patch()` come first in any plan sequence** — items 7, 8, 9, 10, 11 all depend on both helpers existing. Build the helpers (items 5, 6) in the earliest wave.
- **Migration + entity + seed land together** (items 1, 2, 3, 4) — these four edits are atomic for the build to stay green. One plan, one wave.
- **Flowbite CI guard (item 16) must land AFTER doc edits (items 23-25)** — otherwise CI turns red mid-phase.
- **Dev-gate init.ts (item 13) must NOT land before Phase 15 E2E specs exist** — the specs use `__mrnSendAction`, so gating must not break the dev-mode pathway (it doesn't, per A7 in 15-RESEARCH.md; but order the waves defensively).
