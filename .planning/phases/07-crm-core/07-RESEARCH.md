# Phase 7: CRM Core - Research

**Researched:** 2026-03-23
**Domain:** CRUD entity management (contacts + companies) in an SDUI architecture
**Confidence:** HIGH

## Summary

Phase 7 adds contact and company CRUD to the CRM demo application. The implementation is heavily template-driven: Phase 6 established a complete User CRUD pattern (entity, migration, handlers for list/form/save/delete, audit integration, nav registration) that can be replicated almost verbatim for contacts and companies. The only new complexity is the foreign key relationship between contacts and companies (nullable FK, company select dropdown in contact form, linked contacts sub-table on company detail).

All required infrastructure exists: SeaORM entities, migration framework, builder API (DataTable, Form, TextInput, Select, Button, Container, Heading), action routing with auth requirements, and audit logging. No new libraries or architectural patterns are needed.

**Primary recommendation:** Replicate the `handlers/user.rs` pattern for both `handlers/contact.rs` and `handlers/company.rs`, adding FK-aware join queries for the contact list (company name column) and company detail view (linked contacts sub-table).

<user_constraints>

## User Constraints (from CONTEXT.md)

### Locked Decisions
- Contact entity fields: `contact_id` (auto-increment PK), `contact_name` (required), `contact_email` (required, email format), `contact_phone` (optional), `contact_title` (optional), `contact_company` (optional FK to company), `contact_created_at`, `contact_updated_at` (timestamps)
- Company entity fields: `company_id` (auto-increment PK), `company_name` (required, unique), `company_website` (optional, URL string), `company_address` (optional), `company_created_at`, `company_updated_at` (timestamps)
- Contact-company relationship: one company per contact (nullable FK), company detail shows linked contacts sub-table, contact form has company select dropdown
- List views: Contact list columns (name, email, phone, company name joined, created date), Company list columns (name, website, contact count aggregated, created date), both use virtual scroll with server-side sort, default sort name ascending
- Form views: Full-page editing (not modal), required fields show inline errors, email format validated
- Navigation: Sidebar nav with Users (admin only), Contacts, Companies, Audit Log (admin only); contact/company lists as default authenticated view
- Action naming: `{entity}_list`, `{entity}_new`, `{entity}_edit`, `{entity}_save`, `{entity}_delete`

### Claude's Discretion
- Exact validation error messages and positioning
- Whether delete confirmation uses a confirm-dialog or inline confirmation
- Table row click behavior (navigate to edit vs expand inline)
- Empty state messaging for empty contact/company lists
- How the company select in the contact form works (simple select vs autocomplete)

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope

</user_constraints>

<phase_requirements>

## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| CRM-01 | User can create, view, edit, delete contacts | Replicate user.rs handler pattern: `contact_list`, `contact_new`/`contact_edit` (shared form handler), `contact_save`, `contact_delete` with audit |
| CRM-02 | User can create, view, edit, delete companies | Same pattern as contacts: `company_list`, `company_new`/`company_edit`, `company_save`, `company_delete` with audit |
| CRM-03 | User can link contacts to companies | Nullable FK `contact_company` on contact table; Select dropdown in contact form populated from company list; company detail view shows linked contacts sub-table |
| CRM-04 | User can view paginated, sortable data tables | Existing DataTable builder with virtual scroll + server-side sort (established Phase 3/5); add sort parameter handling in list handlers |
| CRM-05 | User can view and edit records in form views | Existing Form/TextInput/Select/Button builders; form renders in main surface; validation via ActionError::BadPayload |

</phase_requirements>

## Standard Stack

### Core (already in project)
| Library | Purpose | Why Standard |
|---------|---------|--------------|
| sea-orm | Entity definitions, CRUD queries, FK relationships | Already used for user entity; supports RelationDef for FK joins |
| sea-orm-migration | Schema migrations with raw SQL | Established pattern with execute_unprepared for SQLite |
| marionette builders | DataTable, Form, TextInput, Select, Button, Container, Heading | Full builder API available from Phase 4 |
| marionette extractors | Db, Session, Payload, HandlerContext | Established handler pattern from Phase 6 |
| marionette router | ActionRouter with AuthRequirement | Route registration pattern in main.rs |
| serde/serde_json | Payload deserialization, JSON data construction | Standard Rust serialization |

### Supporting
| Library | Purpose | When to Use |
|---------|---------|-------------|
| async-trait | MigrationTrait impl | Migration files |
| bcrypt | N/A for this phase | Not needed (no passwords on contacts/companies) |

### No New Dependencies Required
This phase requires zero new crate additions. Everything needed is already in the workspace.

## Architecture Patterns

### Recommended Project Structure
```
backend/crates/crm-demo/src/
  entities/
    mod.rs           # Add: pub mod contact; pub mod company;
    contact.rs       # NEW: SeaORM entity with Relation to company
    company.rs       # NEW: SeaORM entity
    user.rs          # Existing
    audit_log.rs     # Existing
  handlers/
    mod.rs           # Add: pub mod contact; pub mod company;
    contact.rs       # NEW: list, form, save, delete handlers
    company.rs       # NEW: list, form, save, delete handlers
    user.rs          # Existing
    audit.rs         # Existing
    auth.rs          # Existing
  migration/
    mod.rs                              # Add new migrations to vec
    m20260323_000003_create_company.rs  # NEW: company table
    m20260323_000004_create_contact.rs  # NEW: contact table (depends on company)
    ...existing...
  main.rs            # Add action routes + nav items
  seed.rs            # Optionally add demo seed data
  audit.rs           # Existing, reuse as-is
```

### Pattern 1: Entity with Foreign Key (Contact -> Company)
**What:** SeaORM entity with a nullable FK relation
**When to use:** Contact entity referencing company
**Example:**
```rust
// entities/contact.rs
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
    pub contact_company: Option<i32>,  // nullable FK
    pub contact_created_at: String,
    pub contact_updated_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::company::Entity",
        from = "Column::ContactCompany",
        to = "super::company::Column::CompanyId"
    )]
    Company,
}

impl Related<super::company::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Company.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
```

### Pattern 2: Handler with Joined Data (Contact List with Company Name)
**What:** List handler that joins related data for display
**When to use:** Contact list showing company name column
**Example:**
```rust
// In handlers/contact.rs - render_contact_list
let contacts = contact::Entity::find()
    .find_also_related(company::Entity)
    .all(&*db.0)
    .await
    .map_err(|e| ActionError::Internal(e.to_string()))?;

let rows: Vec<serde_json::Value> = contacts
    .iter()
    .map(|(c, company)| {
        let company_name = company.as_ref()
            .map(|co| co.company_name.as_str())
            .unwrap_or("-");
        serde_json::json!({
            "id": c.contact_id,
            "name": c.contact_name,
            "email": c.contact_email,
            "phone": c.contact_phone.as_deref().unwrap_or("-"),
            "company": company_name,
            "created": c.contact_created_at,
            "actions": [
                { "label": "Edit", "action": { "type": "click", "name": "contact_edit", "payload": { "contact_id": c.contact_id } } },
                { "label": "Delete", "action": { "type": "click", "name": "contact_delete", "payload": { "contact_id": c.contact_id } } }
            ]
        })
    })
    .collect();
```

### Pattern 3: Company Detail with Linked Contacts Sub-Table
**What:** Company form/detail view includes a DataTable of linked contacts
**When to use:** Company edit view showing relational data
**Example approach:**
```rust
// After rendering company form fields, add a sub-table of linked contacts
let linked_contacts = contact::Entity::find()
    .filter(contact::Column::ContactCompany.eq(company_id))
    .all(&*db.0)
    .await?;

// Add a second DataTable below the company form showing linked contacts
```

### Pattern 4: Company Select Dropdown in Contact Form
**What:** Populate a Select component with all companies for FK assignment
**When to use:** Contact create/edit form
**Example:**
```rust
let companies = company::Entity::find()
    .order_by_asc(company::Column::CompanyName)
    .all(&*db.0)
    .await?;

let mut company_options = vec![SelectOption {
    value: String::new(),
    label: "No Company".into(),
}];
for co in &companies {
    company_options.push(SelectOption {
        value: co.company_id.to_string(),
        label: co.company_name.clone(),
    });
}

let company_select = Select::new("Company", company_options)
    .id("contact-form-company")
    .bind("/contactForm/company")
    .build();
```

### Pattern 5: Migration with Foreign Key
**What:** SQLite migration creating table with FK constraint
**When to use:** Contact table creation
**Example:**
```rust
// m20260323_000004_create_contact.rs
manager.get_connection().execute_unprepared(
    "CREATE TABLE contact (
        contact_id INTEGER PRIMARY KEY AUTOINCREMENT,
        contact_name TEXT NOT NULL,
        contact_email TEXT NOT NULL,
        contact_phone TEXT,
        contact_title TEXT,
        contact_company INTEGER REFERENCES company(company_id) ON DELETE SET NULL,
        contact_created_at TEXT NOT NULL DEFAULT (datetime('now')),
        contact_updated_at TEXT NOT NULL DEFAULT (datetime('now'))
    )"
).await?;
```

### Anti-Patterns to Avoid
- **Nested entity loading in loops:** Do NOT query company for each contact row individually. Use `find_also_related` for a single joined query.
- **Forgetting audit on all mutations:** Every create, update, delete MUST call `record_audit` after success.
- **Hardcoding company ID as string in select:** The company select value is a string in the form data; parse to `Option<i32>` in the save handler (empty string = None).
- **Creating contact migration before company migration:** Company table must exist first since contact references it with a FK.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| FK join queries | Manual SQL joins | `find_also_related()` | SeaORM handles the LEFT JOIN automatically |
| Audit trail | Custom change tracking | `record_audit` + `compute_changes` | Already battle-tested in Phase 6 |
| Form validation feedback | Custom error rendering | `ActionError::BadPayload` | Frontend already handles error display |
| Table with sort/pagination | Custom list rendering | `DataTable` builder | Virtual scroll + sort already implemented |
| Navigation items | Manual nav construction | `NavItem` + `SideNav` builders | Established sidebar pattern |

**Key insight:** This phase is almost entirely pattern replication. The user.rs CRUD pattern is the exact template. The only novel work is the FK relationship handling (join queries, select population, cascading behavior).

## Common Pitfalls

### Pitfall 1: Migration Order
**What goes wrong:** Contact migration runs before company migration, FK constraint fails
**Why it happens:** Migration files are ordered by filename
**How to avoid:** Number company migration BEFORE contact migration (e.g., 000003 for company, 000004 for contact)
**Warning signs:** Migration error mentioning "no such table: company"

### Pitfall 2: Nullable FK Handling in Forms
**What goes wrong:** Empty string from select is saved as 0 instead of NULL
**Why it happens:** Deserializing empty string to i32 gives 0, not None
**How to avoid:** Use `Option<String>` in the save payload for company field, then parse: empty/null -> None, numeric string -> Some(id)
**Warning signs:** Contacts showing as linked to company_id 0 (nonexistent)

### Pitfall 3: Forgetting to Update Sidebar Navigation
**What goes wrong:** New entities exist but users can't navigate to them
**Why it happens:** Nav items are built in `handle_navigate` in main.rs
**How to avoid:** Add Contacts and Companies nav items for all authenticated users (not admin-only)
**Warning signs:** Working handlers but no way to reach them from UI

### Pitfall 4: Stale updated_at Timestamps
**What goes wrong:** `contact_updated_at` stays at creation time after edits
**Why it happens:** SQLite DEFAULT only applies on INSERT, not UPDATE
**How to avoid:** Explicitly set `contact_updated_at` to current datetime in the save handler's update branch
**Warning signs:** All records showing same created/updated time

### Pitfall 5: Company Delete with Linked Contacts
**What goes wrong:** Deleting a company leaves orphaned contact FK references (or fails with FK constraint)
**Why it happens:** FK constraint behavior depends on SQLite PRAGMA foreign_keys setting
**How to avoid:** Use `ON DELETE SET NULL` in the FK constraint so deleting a company nullifies the contact's company link. Also consider checking for linked contacts before delete and warning the user.
**Warning signs:** Foreign key constraint errors on company delete, or contacts with dangling company references

### Pitfall 6: Missing `async-trait` Import in Migration
**What goes wrong:** Compilation error in migration file
**Why it happens:** `MigrationTrait` requires `#[async_trait::async_trait]` attribute
**How to avoid:** Copy the existing migration file structure exactly (includes the attribute)
**Warning signs:** Compiler error about async trait methods

## Code Examples

### Contact Entity (entities/contact.rs)
```rust
// Based on established user.rs pattern + FK relation
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

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::company::Entity",
        from = "Column::ContactCompany",
        to = "super::company::Column::CompanyId"
    )]
    Company,
}

impl Related<super::company::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Company.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
```

### Company Entity (entities/company.rs)
```rust
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "company")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub company_id: i32,
    #[sea_orm(unique)]
    pub company_name: String,
    pub company_website: Option<String>,
    pub company_address: Option<String>,
    pub company_created_at: String,
    pub company_updated_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::contact::Entity")]
    Contacts,
}

impl Related<super::contact::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Contacts.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
```

### Contact Save Handler (validation pattern)
```rust
#[derive(Deserialize)]
struct ContactSavePayload {
    id: Option<i32>,
    name: String,
    email: String,
    phone: Option<String>,
    title: Option<String>,
    company: Option<String>,  // String from select, parse to Option<i32>
}

// In handle_contact_save:
if data.name.trim().is_empty() {
    return Err(ActionError::BadPayload("Name is required".into()));
}
if data.email.trim().is_empty() {
    return Err(ActionError::BadPayload("Email is required".into()));
}
// Basic email format check
if !data.email.contains('@') {
    return Err(ActionError::BadPayload("Invalid email format".into()));
}

// Parse company FK: empty string or None -> None, numeric string -> Some(id)
let company_id: Option<i32> = data.company
    .as_deref()
    .and_then(|s| if s.is_empty() { None } else { s.parse().ok() });
```

### Navigation Update (main.rs)
```rust
// In handle_navigate, after existing nav items:
let contacts_item = NavItem::new("Contacts", "/contacts")
    .id("nav-contacts")
    .action(ComponentAction::click("contact_list"))
    .build();
nav_items.push(contacts_item);

let companies_item = NavItem::new("Companies", "/companies")
    .id("nav-companies")
    .action(ComponentAction::click("company_list"))
    .build();
nav_items.push(companies_item);
```

### Action Router Registration (main.rs)
```rust
// Contact actions (all authenticated users)
.action("contact_list", box_handler(handlers::contact::handle_contact_list), AuthRequirement::Authenticated)
.action("contact_new", box_handler(handlers::contact::handle_contact_form), AuthRequirement::Authenticated)
.action("contact_edit", box_handler(handlers::contact::handle_contact_form), AuthRequirement::Authenticated)
.action("contact_save", box_handler(handlers::contact::handle_contact_save), AuthRequirement::Authenticated)
.action("contact_delete", box_handler(handlers::contact::handle_contact_delete), AuthRequirement::Authenticated)
// Company actions (all authenticated users)
.action("company_list", box_handler(handlers::company::handle_company_list), AuthRequirement::Authenticated)
.action("company_new", box_handler(handlers::company::handle_company_form), AuthRequirement::Authenticated)
.action("company_edit", box_handler(handlers::company::handle_company_form), AuthRequirement::Authenticated)
.action("company_save", box_handler(handlers::company::handle_company_save), AuthRequirement::Authenticated)
.action("company_delete", box_handler(handlers::company::handle_company_delete), AuthRequirement::Authenticated)
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| N/A | This is the first CRM entity phase | Phase 7 | Establishes the entity CRUD pattern for all future CRM entities |

**Nothing deprecated/outdated** -- all patterns established in Phase 6 remain current.

## Open Questions

1. **Company contact count in list view**
   - What we know: Company list needs a "contact count" column (aggregated)
   - What's unclear: Whether to use a subquery count, a separate query, or a raw SQL count
   - Recommendation: Use SeaORM's `find_with_related` or a raw `SELECT company.*, COUNT(contact.contact_id)` approach. A separate count query per company is acceptable at demo scale but a grouped query is cleaner.

2. **Default authenticated view**
   - What we know: Context says "contact/company lists are the default authenticated view"
   - What's unclear: Whether `navigate` action should render contact list directly or show a dashboard
   - Recommendation: Change `handle_navigate` to redirect to `contact_list` (or render it inline) since that is the primary CRM view.

3. **Email validation depth**
   - What we know: Email format must be validated
   - What's unclear: How strict (contains '@' vs full RFC 5322)
   - Recommendation: Simple `contains('@')` check plus `contains('.')` after the '@'. This is a demo app, not a production email validator.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | cargo test (Rust) + Playwright (E2E) |
| Config file | `backend/Cargo.toml` (workspace) + `frontend/playwright.e2e.config.ts` |
| Quick run command | `cargo test -p crm-demo` |
| Full suite command | `cd backend && cargo test && cd ../frontend && npx playwright test --config=playwright.e2e.config.ts` |

### Phase Requirements to Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| CRM-01 | Contact CRUD (create, view, edit, delete) | E2E | `npx playwright test --config=playwright.e2e.config.ts -g "contact"` | No - Wave 0 |
| CRM-02 | Company CRUD (create, view, edit, delete) | E2E | `npx playwright test --config=playwright.e2e.config.ts -g "company"` | No - Wave 0 |
| CRM-03 | Link contacts to companies | E2E | `npx playwright test --config=playwright.e2e.config.ts -g "link"` | No - Wave 0 |
| CRM-04 | Sortable data tables | E2E | `npx playwright test --config=playwright.e2e.config.ts -g "sort"` | No - Wave 0 |
| CRM-05 | Form views with validation | E2E | `npx playwright test --config=playwright.e2e.config.ts -g "form"` | No - Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test -p crm-demo` + `cargo clippy -p crm-demo`
- **Per wave merge:** Full backend test suite `cd backend && cargo test`
- **Phase gate:** Full suite + E2E tests green

### Wave 0 Gaps
- [ ] `frontend/tests/e2e/crm-contacts.spec.ts` -- covers CRM-01, CRM-03, CRM-05
- [ ] `frontend/tests/e2e/crm-companies.spec.ts` -- covers CRM-02, CRM-04
- [ ] E2E tests depend on seeded data -- extend `seed.rs` with demo contacts/companies

## Sources

### Primary (HIGH confidence)
- `backend/crates/crm-demo/src/handlers/user.rs` -- Complete CRUD handler pattern (list, form, save, delete)
- `backend/crates/crm-demo/src/entities/user.rs` -- SeaORM entity definition pattern
- `backend/crates/crm-demo/src/audit.rs` -- Audit logging pattern (record_audit + compute_changes)
- `backend/crates/crm-demo/src/main.rs` -- Action router registration, sidebar nav, app state
- `backend/crates/crm-demo/src/migration/m20260323_000001_create_user.rs` -- Migration pattern with raw SQL
- `backend/crates/marionette/src/builders/standard.rs` -- All 18 component builders

### Secondary (MEDIUM confidence)
- SeaORM `find_also_related` for FK join queries -- based on SeaORM documentation patterns
- SQLite `ON DELETE SET NULL` FK behavior -- standard SQL, verified in SQLite docs

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- all libraries already in use, zero new dependencies
- Architecture: HIGH -- direct replication of user.rs pattern with FK additions
- Pitfalls: HIGH -- based on direct code inspection of existing patterns and SQL conventions

**Research date:** 2026-03-23
**Valid until:** 2026-04-23 (stable -- internal project patterns, no external dependency changes)
