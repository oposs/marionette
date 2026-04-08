# Phase 9: CRM Listmonk - Context

**Gathered:** 2026-03-23
**Status:** Ready for planning

<domain>
## Phase Boundary

Integrate the CRM with Listmonk for newsletter management. Users can sync contacts to Listmonk subscriber lists, view mailing campaign history per contact, see sync status with error details, and have contact changes propagate to Listmonk (create, update, unsubscribe). This is the final phase — it adds external service integration on top of the complete CRM.

</domain>

<decisions>
## Implementation Decisions

### Listmonk connection
- Configuration via environment variables: `LISTMONK_URL`, `LISTMONK_USER`, `LISTMONK_PASSWORD`
- Basic auth against Listmonk's REST API (Listmonk's standard auth method)
- Connection validated on startup — log warning if Listmonk is unreachable but don't prevent CRM from starting
- HTTP client: `reqwest` crate (async, already compatible with tokio)

### Sync behavior
- Manual sync per contact — "Sync to Listmonk" button on contact detail view
- Bulk sync — "Sync All" button on contact list (syncs all contacts with email addresses)
- Sync creates/updates Listmonk subscriber using contact email as identifier
- Contact name maps to Listmonk subscriber name
- Tags map to Listmonk subscriber lists (each CRM tag = a Listmonk list)
- On contact email change, update the Listmonk subscriber
- On contact delete, mark Listmonk subscriber as "blocklisted" (not deleted — preserves history)

### Sync status tracking
- `listmonk_sync` table: `listmonk_sync_contact` (FK), `listmonk_sync_status` (success/error), `listmonk_sync_error` (nullable text), `listmonk_sync_subscriber_id` (Listmonk's ID), `listmonk_sync_at` (timestamp)
- Sync status shown as badge on contact list and detail view (synced/error/never synced)
- Error details visible on hover or click

### Mailing history
- Fetch campaign send history from Listmonk API per subscriber ID
- Display as a read-only timeline on contact detail (below interactions)
- Shows: campaign name, sent date, status (sent/opened/clicked if available)
- Cached locally to avoid repeated API calls — refresh on demand

### Claude's Discretion
- Exact Listmonk API endpoint paths and payload formats
- Retry logic for failed API calls
- Cache duration for mailing history
- How to handle Listmonk being down (graceful degradation UI)
- Whether to add a Listmonk settings/status page in admin
- Bulk sync progress reporting

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### CRM entities (what gets synced)
- `backend/crates/crm-demo/src/entities/contact.rs` — Contact entity (email is the sync key)
- `backend/crates/crm-demo/src/entities/tag.rs` — Tags map to Listmonk lists
- `backend/crates/crm-demo/src/entities/contact_tag.rs` — Junction table

### CRM handlers (where sync integrates)
- `backend/crates/crm-demo/src/handlers/contact.rs` — Contact CRUD, detail view
- `backend/crates/crm-demo/src/main.rs` — Action routing

### Backend toolkit
- `backend/crates/marionette/src/builders/standard.rs` — Component builders
- `backend/crates/marionette/src/router.rs` — ActionRouter

### Conventions
- `TOOLING.md` — SQL conventions

### Prior contexts
- `.planning/phases/07-crm-core/07-CONTEXT.md` — Contact entity fields
- `.planning/phases/08-crm-features/08-CONTEXT.md` — Tags, interactions, timeline pattern

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- Contact CRUD handlers — add sync button to detail view, sync status to list view
- Tag entity — map to Listmonk lists during sync
- Interaction timeline pattern — reuse for mailing history display
- `record_audit` — log sync operations
- `AppState` — extend with Listmonk HTTP client

### Established Patterns
- Action handlers return `Result<Vec<ProtocolMessage>, ActionError>`
- SeaORM entities + migrations for new tables
- DataTable for list views, Form for detail views

### Integration Points
- `main.rs` — Register sync action handlers, initialize Listmonk client
- `handlers/contact.rs` — Add sync button, sync status badge, mailing history section
- `entities/mod.rs` — Add listmonk_sync entity
- New `listmonk.rs` module for API client

</code_context>

<specifics>
## Specific Ideas

- Tags → Listmonk lists mapping is the natural bridge between CRM categorization and newsletter segmentation
- Blocklisting on delete (not deleting) preserves Listmonk's mailing history for the contact
- Manual sync gives the user control — no surprise newsletter subscriptions

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 09-crm-listmonk*
*Context gathered: 2026-03-23*
