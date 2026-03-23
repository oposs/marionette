---
status: awaiting_human_verify
trigger: "Two CRM Issues: DataTable not rendering rows + Company save does nothing"
created: 2026-03-23T00:00:00Z
updated: 2026-03-23T00:00:00Z
---

## Current Focus

hypothesis: Two independent root causes found - see Resolution
test: Trace data format mismatch for Bug 1 and payload structure mismatch for Bug 2
expecting: Both confirmed by code analysis
next_action: Document root causes and apply fixes

## Symptoms

expected: Bug1 - Seed data should appear in DataTable. Bug2 - Company save should create record and navigate to list.
actual: Bug1 - Tables show headers but no rows. Bug2 - Save click does nothing visible.
errors: No error messages reported
reproduction: Bug1 - Login, navigate to Contacts/Companies/Users. Bug2 - Companies > New Company > fill form > Save.
started: Since current implementation

## Eliminated

(none yet)

## Evidence

- timestamp: 2026-03-23T00:01:00Z
  checked: DataTable.svelte virtual scroll logic
  found: totalRows defaults to 0 when not in props. visibleEnd = Math.min(totalRows=0, ...) = 0, so visibleRows is always empty slice.
  implication: Even if data arrives correctly, zero rows would be visible because totalRows is 0.

- timestamp: 2026-03-23T00:02:00Z
  checked: Backend DataTable builder (standard.rs)
  found: DataTable struct only has `columns` and `page_size` fields. No `totalRows` prop is ever sent.
  implication: Frontend always sees totalRows=0, confirming Bug 1 root cause.

- timestamp: 2026-03-23T00:03:00Z
  checked: Backend data format for contacts (contact.rs line 393-424)
  found: Data is sent as `"contacts": [array of objects]` (a JSON array). DataTable.svelte expects a keyed object and does Object.entries(rawData) which works on arrays but produces ["0", obj] pairs.
  implication: Secondary issue - array vs keyed-object format. But primary blocker is totalRows=0.

- timestamp: 2026-03-23T00:04:00Z
  checked: Backend data format for companies (company.rs line 104-125)
  found: Same pattern - `"companies": [array of objects]` sent as JSON array.
  implication: Same issue applies to companies.

- timestamp: 2026-03-23T00:05:00Z
  checked: Button.svelte handleClick (line 23-41)
  found: Payload = {...action.payload, ...getAllData(surface)}. Surface data has companyForm nested: {companyForm: {id, name, website, address}}
  implication: CompanySavePayload expects top-level {id, name, website, address} but receives {companyForm: {id:..., name:..., ...}}. Deserialization fails.

- timestamp: 2026-03-23T00:06:00Z
  checked: Backend company save handler (company.rs line 376-381)
  found: `Payload::<CompanySavePayload>::from_context(&ctx)` - expects flat {id, name, website, address}
  implication: Payload extraction fails, returns error. But error may not be visible to user (need to check error handling).

## Resolution

root_cause: |
  BUG 1 (DataTable no rows): TWO issues combine:
  (a) The backend DataTable builder has no `totalRows` property. The frontend DataTable.svelte reads `props.totalRows` which defaults to 0. Virtual scroll uses `Math.min(totalRows=0, ...)` for visibleEnd, so visibleRows is always an empty slice.
  (b) Data is sent as JSON arrays but DataTable does Object.entries() expecting a keyed object. This is secondary since even with correct data format, totalRows=0 blocks rendering.

  BUG 2 (Company save does nothing): Payload structure mismatch. The form binds fields to `/companyForm/name`, `/companyForm/website`, etc. Button.svelte sends ALL surface data as payload, so the backend receives `{companyForm: {id, name, ...}}` but `CompanySavePayload` expects flat `{id, name, ...}`. Deserialization fails.

fix: |
  Bug 1: In DataTable.svelte, made totalRows fall back to rows.length when not provided via props (or when 0).
  This allows the virtual scroll to work correctly whether the backend sends totalRows or not.
  Also guarded prefetch trigger to only fire when explicitTotalRows > 0 (server-paginated mode).

  Bug 2: Wrapped all save payload structs to match the actual frontend payload structure.
  The frontend sends all surface data with form fields nested under their bind prefix
  (e.g. companyForm, contactForm, userForm, etc.). Changed all *SavePayload structs to
  use a wrapper with #[serde(rename = "...")] pointing to the nested form data.
  Fixed: CompanySavePayload, ContactSavePayload, UserSavePayload, NoteSavePayload,
  InteractionSavePayload, TagSavePayload.

verification: Backend compiles (cargo check). Frontend type-checks (svelte-check, no new errors). Needs manual verification in browser.
files_changed:
  - frontend/src/lib/components/table/DataTable.svelte
  - backend/crates/crm-demo/src/handlers/company.rs
  - backend/crates/crm-demo/src/handlers/contact.rs
  - backend/crates/crm-demo/src/handlers/user.rs
  - backend/crates/crm-demo/src/handlers/note.rs
  - backend/crates/crm-demo/src/handlers/interaction.rs
