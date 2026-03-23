# Phase 7: CRM Core - Context

**Gathered:** 2026-03-23
**Status:** Ready for planning

<domain>
## Phase Boundary

Implement contact and company CRUD for the CRM demo. Users can create, view, edit, and delete contacts and companies, link contacts to companies, view data in sortable virtual-scroll tables, and edit records in form views with validation feedback. Notes, tags, search, filtering, and interaction tracking are Phase 8 — this phase builds the core entity management.

</domain>

<decisions>
## Implementation Decisions

### Contact entity fields
- `contact_id` (auto-increment PK)
- `contact_name` (required, string)
- `contact_email` (required, email format)
- `contact_phone` (optional, string)
- `contact_title` (optional, string — job title)
- `contact_company` (optional FK to company — a contact can exist without a company)
- `contact_created_at`, `contact_updated_at` (timestamps)

### Company entity fields
- `company_id` (auto-increment PK)
- `company_name` (required, string, unique)
- `company_website` (optional, URL string)
- `company_address` (optional, string)
- `company_created_at`, `company_updated_at` (timestamps)

### Contact-company relationship
- One company per contact (FK `contact_company` → `company.company_id`)
- Contact can exist without a company (nullable FK)
- Company detail view shows linked contacts as a sub-table
- Contact form has a company select dropdown (searchable if many companies)

### List views (data tables)
- Contact list: columns — name, email, phone, company name (joined), created date
- Company list: columns — name, website, contact count (aggregated), created date
- Both use virtual scroll with server-side sort (established in Phase 3/5)
- Default sort: name ascending

### Form views
- Contact form: name, email, phone, title, company select, save/cancel buttons
- Company form: name, website, address, save/cancel buttons
- Validation: required fields show inline errors, email format validated
- Forms render in main surface (not modal) — full-page editing

### Navigation
- Sidebar nav: Users (admin only), Contacts, Companies, Audit Log (admin only)
- Contact/company lists are the default authenticated view
- Clicking a row in the table navigates to the edit form
- "New Contact" / "New Company" button above the table

### Claude's Discretion
- Exact validation error messages and positioning
- Whether delete confirmation uses a confirm-dialog or inline confirmation
- Table row click behavior (navigate to edit vs expand inline)
- Empty state messaging for empty contact/company lists
- How the company select in the contact form works (simple select vs autocomplete)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Established CRM patterns (Phase 6 as template)
- `backend/crates/crm-demo/src/handlers/user.rs` — User CRUD pattern to replicate for contacts/companies
- `backend/crates/crm-demo/src/entities/user.rs` — SeaORM entity pattern
- `backend/crates/crm-demo/src/audit.rs` — record_audit helper for mutations
- `backend/crates/crm-demo/src/main.rs` — Action routing, nav sidebar, auth wiring

### Backend toolkit
- `backend/crates/marionette/src/builders/standard.rs` — Component builders (DataTable, Form, TextInput, Select, Button)
- `backend/crates/marionette/src/router.rs` — ActionRouter
- `backend/crates/marionette/src/db.rs` — SeaORM patterns

### Conventions
- `TOOLING.md` — SQL conventions (singular tables, prefixed fields, FK naming)

### Protocol
- `spec/PROTOCOL.md` — Message types, data binding, keyed collections

### Prior contexts
- `.planning/phases/03-frontend-library/03-CONTEXT.md` — Virtual scroll table, server-side sort
- `.planning/phases/04-backend-toolkit/04-CONTEXT.md` — Builder pattern, action routing, SeaORM
- `.planning/phases/06-crm-auth-foundation/06-CONTEXT.md` — Auth, user CRUD pattern, audit trail

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `handlers/user.rs` — Complete CRUD pattern: list handler with DataTable, form handler with TextInput/Select/Button, save handler with validation, delete handler with audit. Direct template for contact/company handlers.
- `entities/user.rs` — SeaORM entity pattern with DeriveEntityModel
- `audit.rs` — `record_audit` + `compute_changes` — call after all mutations
- Standard builders: DataTable, Form, TextInput, Select, Button, Container, Heading
- `seed.rs` — Pattern for seeding initial data

### Established Patterns
- Action names: `{entity}_list`, `{entity}_new`, `{entity}_edit`, `{entity}_save`, `{entity}_delete`
- All mutations protected with `#[requires(authenticated)]`
- Admin-only actions use `AuthRequirement::Role("admin")`
- Sidebar nav built in main.rs navigate handler

### Integration Points
- `main.rs` — Register new action handlers, add nav items for contacts/companies
- `entities/mod.rs` — Add contact and company entity modules
- `handlers/mod.rs` — Add contact and company handler modules
- Migrations — New migration files for contact and company tables

</code_context>

<specifics>
## Specific Ideas

- Follow the user CRUD pattern exactly — same handler structure, same builder patterns, same audit integration
- Company detail view showing linked contacts demonstrates the relational aspect of SDUI
- Virtual scroll on contact list is important since contacts will be the largest table

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 07-crm-core*
*Context gathered: 2026-03-23*
