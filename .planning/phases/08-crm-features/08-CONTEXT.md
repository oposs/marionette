# Phase 8: CRM Features - Context

**Gathered:** 2026-03-23
**Status:** Ready for planning

<domain>
## Phase Boundary

Add notes, tags, search, filtering, and interaction tracking to the CRM. Users can add timestamped notes to contacts and companies, create/apply tags to contacts, search contacts by name/email/company, filter contact lists by company/tags/date range, log interactions (calls, emails, meetings) per contact, and view a chronological interaction timeline. Listmonk integration is Phase 9.

</domain>

<decisions>
## Implementation Decisions

### Notes (CRM-06)
- Plain text notes (no rich text editor for the demo)
- `note` table: `note_id`, `note_contact` (nullable FK), `note_company` (nullable FK), `note_text`, `note_user` (FK to user — who wrote it), `note_created_at`
- Notes displayed as a chronological list below the contact/company edit form
- "Add Note" is a text area + submit button at the top of the notes section
- Notes are append-only (no edit/delete for simplicity)

### Tags (CRM-08)
- Free-form tags — user types a tag name, system creates if new
- `tag` table: `tag_id`, `tag_name` (unique)
- `contact_tag` junction table: `contact_tag_contact`, `contact_tag_tag`
- Tags displayed as colored chips on the contact list and detail view
- Contact form has a tag input field (type to add, click to remove)
- Tag colors auto-assigned from a fixed palette based on tag name hash

### Search (CRM-07)
- Search bar above the contact list table
- Server-side search via SQL LIKE on contact_name, contact_email, and joined company_name
- Submit-based (user types, presses Enter or clicks search icon) — not instant search
- Search query sent as action payload, backend returns filtered contact list
- Clear search button to reset to full list

### Filtering (CRM-09)
- Filter panel above the contact list (collapsible)
- Filters: company select dropdown, tag multi-select, date range (created_at from/to)
- Filters combine with AND logic (company AND tags AND date range)
- Filters combine with search (search AND filters)
- Active filters shown as removable chips above the table
- Filter state sent as action payload alongside search query

### Interactions (CRM-10, CRM-11)
- Three interaction types: Call, Email, Meeting
- `interaction` table: `interaction_id`, `interaction_contact` (FK), `interaction_type` (enum: call/email/meeting), `interaction_subject` (string), `interaction_notes` (text), `interaction_user` (FK — who logged it), `interaction_date` (when it happened), `interaction_created_at`
- "Log Interaction" button on contact detail opens a form (in main surface, not modal)
- Timeline displayed on contact detail below notes — chronological, newest first
- Timeline entries show: type icon, subject, date, logged-by user name, notes preview
- Interaction types use distinct visual indicators (phone icon, email icon, calendar icon)

### Claude's Discretion
- Exact tag color palette and hash algorithm
- Search debounce timing (if any)
- Filter panel collapse/expand animation
- Timeline entry visual layout details
- Whether notes section appears on company detail too (requirement says "contacts and companies")
- Date range picker implementation (two text inputs vs date picker component)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### CRM patterns (Phase 7 as template)
- `backend/crates/crm-demo/src/handlers/contact.rs` — Contact CRUD pattern to extend with notes/tags/search/filter
- `backend/crates/crm-demo/src/handlers/company.rs` — Company CRUD to extend with notes
- `backend/crates/crm-demo/src/entities/contact.rs` — Contact entity to add relations
- `backend/crates/crm-demo/src/entities/company.rs` — Company entity
- `backend/crates/crm-demo/src/main.rs` — Action routing, nav

### Auth & audit (Phase 6)
- `backend/crates/crm-demo/src/audit.rs` — record_audit for mutations
- `backend/crates/crm-demo/src/handlers/auth.rs` — Auth patterns

### Backend toolkit
- `backend/crates/marionette/src/builders/standard.rs` — Component builders
- `backend/crates/marionette/src/router.rs` — ActionRouter

### Conventions
- `TOOLING.md` — SQL conventions

### Prior contexts
- `.planning/phases/07-crm-core/07-CONTEXT.md` — Contact/company entity fields, CRUD patterns

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- Contact/company CRUD handlers — direct template for new entity handlers (note, tag, interaction)
- `record_audit` helper — use for all mutations
- DataTable builder — for interaction timeline (or custom list rendering)
- Form/TextInput/Select/Button builders — for note input, interaction form, filter panel
- `find_also_related` pattern from contact→company join — reuse for contact→tags, contact→interactions

### Established Patterns
- Action naming: `{entity}_list`, `{entity}_save`, `{entity}_delete`
- All mutations protected with `AuthRequirement::Authenticated`
- Sidebar nav structure in main.rs
- Migration numbering: sequential `m20260323_000NNN`

### Integration Points
- `handlers/contact.rs` — Add search/filter params to list handler, add notes/tags/interactions sections to detail view
- `handlers/company.rs` — Add notes section to company detail
- `entities/mod.rs` — Add note, tag, contact_tag, interaction entity modules
- `migration/mod.rs` — Add new migration files
- `main.rs` — Register new action handlers

</code_context>

<specifics>
## Specific Ideas

- Notes are append-only for simplicity — no edit/delete complexity
- Tags are free-form with auto-creation — no separate tag management screen needed
- Search + filters combine with AND — simple and predictable
- Interaction timeline is the most visually distinctive feature — type icons differentiate entries

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 08-crm-features*
*Context gathered: 2026-03-23*
