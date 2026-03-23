# Phase 8: CRM Features - Research

**Researched:** 2026-03-23
**Domain:** CRM feature enrichment (notes, tags, search, filtering, interaction tracking) on existing Rust/SeaORM/SDUI stack
**Confidence:** HIGH

## Summary

Phase 8 extends the existing CRM contact and company CRUD (Phase 7) with six features: notes, tags, search, filtering, interaction logging, and interaction timeline. All decisions are locked in CONTEXT.md with detailed schema definitions and UI behavior. The implementation is entirely within the established stack (SeaORM 1.1 on SQLite, Axum action handlers, Marionette SDUI builders).

The primary challenge is query composition: the contact list handler must accept optional search terms, filter parameters (company, tags, date range), and combine them with AND logic. SeaORM's `Condition` builder handles this well. The secondary challenge is the many-to-many tag relationship via a junction table, which SeaORM supports via `Related` and `Linked` traits.

**Primary recommendation:** Implement in 4 plans: (1) migrations + entities for note/tag/contact_tag/interaction, (2) notes feature on contact and company detail views, (3) tags + search + filtering on contact list, (4) interaction logging + timeline on contact detail.

<user_constraints>

## User Constraints (from CONTEXT.md)

### Locked Decisions
- Notes: Plain text, append-only, `note` table with nullable FKs to contact/company, displayed chronologically below edit forms
- Tags: Free-form with auto-creation, `tag` + `contact_tag` junction tables, colored chips via name-hash palette
- Search: Server-side SQL LIKE on contact_name/email/company_name, submit-based (not instant), sent as action payload
- Filtering: Company select, tag multi-select, date range (from/to), AND logic, combine with search, collapsible panel with active filter chips
- Interactions: Three types (call/email/meeting), `interaction` table with type enum, subject, notes, user FK, date fields; "Log Interaction" opens form in main surface; timeline on contact detail, newest first, with type icons

### Claude's Discretion
- Exact tag color palette and hash algorithm
- Search debounce timing (if any)
- Filter panel collapse/expand animation
- Timeline entry visual layout details
- Whether notes section appears on company detail too (requirement says "contacts and companies")
- Date range picker implementation (two text inputs vs date picker component)

### Deferred Ideas (OUT OF SCOPE)
None

</user_constraints>

<phase_requirements>

## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| CRM-06 | User can add notes to contacts and companies | Note entity + migration, append-only save handler, chronological list rendering on contact/company detail |
| CRM-07 | User can search contacts by name, email, company | SQL LIKE conditions on contact_name, contact_email, joined company_name; search payload in contact_list handler |
| CRM-08 | User can tag/label contacts for categorization | Tag + contact_tag entities, free-form tag input, auto-create-if-new, colored chips via hash |
| CRM-09 | User can filter lists by company, tag, date range | SeaORM Condition builder for AND-combined filters, filter payload alongside search in contact_list |
| CRM-10 | User can log interactions per contact | Interaction entity with type enum, form handler, audit trail |
| CRM-11 | User can view interaction timeline per contact | Query interactions by contact ordered by date desc, render with type icons and metadata |

</phase_requirements>

## Standard Stack

### Core (already in project)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| sea-orm | 1.1 | ORM for SQLite entities, queries, relations | Already established in project |
| sea-orm-migration | 1.1 | Schema migrations | Already established |
| axum | (workspace) | HTTP/WS framework | Already established |
| marionette builders | (local crate) | SDUI component construction | Already established |
| serde/serde_json | (workspace) | Payload serialization | Already established |
| time | (workspace) | OffsetDateTime for timestamps | Already used in Phase 7 |

### No New Dependencies Required
Phase 8 requires no new crate dependencies. All features are implementable with existing SeaORM query capabilities, existing SDUI builders (Container, Heading, Text, TextInput, Select, Button, Form, DataTable), and standard Rust.

## Architecture Patterns

### New Entity Files
```
backend/crates/crm-demo/src/
├── entities/
│   ├── mod.rs              # Add: note, tag, contact_tag, interaction
│   ├── note.rs             # NEW
│   ├── tag.rs              # NEW
│   ├── contact_tag.rs      # NEW junction table
│   └── interaction.rs      # NEW
├── handlers/
│   ├── mod.rs              # Add: note, interaction
│   ├── contact.rs          # MODIFY: add search/filter/tags to list, add notes/interactions sections to edit
│   ├── company.rs          # MODIFY: add notes section to edit form
│   ├── note.rs             # NEW: note_save handler
│   └── interaction.rs      # NEW: interaction_save handler
├── migration/
│   ├── mod.rs              # Add 4 new migrations
│   ├── m20260323_000005_create_note.rs
│   ├── m20260323_000006_create_tag.rs
│   ├── m20260323_000007_create_contact_tag.rs
│   └── m20260323_000008_create_interaction.rs
├── seed.rs                 # Add: seed_tags, seed_notes, seed_interactions
└── main.rs                 # Register new action handlers
```

### Pattern 1: SeaORM Condition Builder for Search + Filters
**What:** Dynamic query composition using `Condition::all()` with optional clauses
**When to use:** Contact list handler when search/filter params are present
**Example:**
```rust
use sea_orm::{Condition, ColumnTrait, QueryFilter};

// Build dynamic WHERE clause
let mut condition = Condition::all();

if let Some(ref query) = params.search {
    let like_pattern = format!("%{}%", query);
    condition = condition.add(
        Condition::any()
            .add(contact::Column::ContactName.contains(&query))
            .add(contact::Column::ContactEmail.contains(&query))
            // company name requires join — handle via subquery or post-filter
    );
}

if let Some(company_id) = params.company_filter {
    condition = condition.add(contact::Column::ContactCompany.eq(company_id));
}

if let Some(ref from_date) = params.date_from {
    condition = condition.add(contact::Column::ContactCreatedAt.gte(from_date.clone()));
}

if let Some(ref to_date) = params.date_to {
    condition = condition.add(contact::Column::ContactCreatedAt.lte(to_date.clone()));
}

let contacts = contact::Entity::find()
    .find_also_related(company::Entity)
    .filter(condition)
    .order_by_asc(contact::Column::ContactName)
    .all(&*db.0)
    .await?;
```

### Pattern 2: Many-to-Many Tag Relationship
**What:** Junction table pattern with SeaORM for contact<->tag
**When to use:** Loading tags for a contact, filtering contacts by tags
**Example:**
```rust
// Entity: contact_tag.rs
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "contact_tag")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub contact_tag_contact: i32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub contact_tag_tag: i32,
}

// To filter contacts by tag IDs, use a subquery or join:
// SELECT DISTINCT contact_tag_contact FROM contact_tag WHERE contact_tag_tag IN (...)
// Then: contact::Column::ContactId.is_in(contact_ids_with_tag)
```

### Pattern 3: Append-Only Notes Rendering
**What:** Notes displayed as a chronological list below forms, with "Add Note" textarea at top
**When to use:** Contact edit and company edit views
**Example:**
```rust
// In contact form handler, after form fields:
let notes = note::Entity::find()
    .filter(note::Column::NoteContact.eq(contact_id))
    .order_by_desc(note::Column::NoteCreatedAt)
    .all(&*db.0)
    .await?;

// Render notes section: Heading + TextInput (for new note) + Button + list of Text components
let notes_heading = Heading::new("Notes").id("notes-heading").build();
let note_input = TextInput::new("Add a note...")
    .id("note-input")
    .bind("/noteForm/text")
    .build();
let note_submit = Button::new("Add Note")
    .id("note-submit")
    .action(ComponentAction::submit("note_save"))
    .build();
// Each existing note rendered as a Text component with timestamp
```

### Pattern 4: Interaction Timeline Rendering
**What:** Chronological list of interactions with type icons, rendered on contact detail
**When to use:** Contact edit/detail view
**Example:**
```rust
// Query interactions for this contact
let interactions = interaction::Entity::find()
    .filter(interaction::Column::InteractionContact.eq(contact_id))
    .order_by_desc(interaction::Column::InteractionDate)
    .all(&*db.0)
    .await?;

// Each interaction becomes a Container with:
// - type indicator (text showing "Call" / "Email" / "Meeting")
// - subject heading
// - date + logged-by info
// - notes preview as Text component
```

### Anti-Patterns to Avoid
- **N+1 tag queries per contact in list view:** Load all contact_tag rows for the displayed contacts in one query, then distribute in Rust. Do not query tags per contact row.
- **Storing tag names directly on contact:** Use the junction table. Free-form means auto-create the tag entity, not skip normalization.
- **Complex client-side filtering:** All filtering is server-side. The frontend sends filter params as action payload; the backend returns filtered results.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Dynamic query conditions | Manual SQL string building | SeaORM `Condition::all()` / `Condition::any()` | Type-safe, prevents SQL injection |
| Tag color assignment | Complex color picker or random generation | Deterministic hash of tag name to fixed palette index | Consistent colors across views, no storage needed |
| Date range validation | Custom parsing logic | Simple string comparison (SQLite TEXT dates in ISO format sort correctly) | SQLite stores dates as TEXT in `datetime('now')` format which is ISO-sortable |
| Search across joined tables | Raw SQL | SeaORM `find_also_related` + post-filter on company name | Maintains ORM safety; for demo scale, filtering in Rust after join is fine |

**Key insight:** At demo scale (hundreds of contacts, not millions), some patterns that would be unacceptable at production scale are fine. For example, filtering company name matches in Rust after a `find_also_related` join, or loading all contact_tag rows for a page of contacts. Keep it simple.

## Common Pitfalls

### Pitfall 1: Search on Joined Company Name
**What goes wrong:** SeaORM `contains()` / `like()` filters work on columns of the primary entity. Searching on the joined company name requires either raw SQL, a subquery, or post-filtering.
**Why it happens:** `find_also_related` returns `(contact::Model, Option<company::Model>)` tuples, but the filter condition only applies to the primary entity's columns.
**How to avoid:** For demo scale, apply company name search as a post-filter in Rust after the query returns. For the search string, query contacts with LIKE on name/email, then also include contacts whose company name matches by doing a separate company name lookup and filtering by company_id.
**Warning signs:** Empty search results when searching by company name.

### Pitfall 2: Tag Filter with AND vs OR Semantics
**What goes wrong:** When filtering by multiple tags, unclear whether contacts must have ALL selected tags (AND) or ANY selected tag (OR).
**Why it happens:** The CONTEXT says filters combine with AND, but multiple tags within the tag filter likely means "any of these tags" (OR within tag filter, AND between filter types).
**How to avoid:** Interpret tag multi-select as OR (contact has any of the selected tags). The AND logic applies between filter dimensions (company AND tags AND date range).
**Warning signs:** Zero results when multiple tags are selected if using AND within tags.

### Pitfall 3: Migration Ordering with FK Dependencies
**What goes wrong:** The `note` table references both `contact` and `company`. The `contact_tag` table references both `contact` and `tag`. These must be created after their referenced tables.
**Why it happens:** SQLite enforces FK constraints at insert time (not CREATE TABLE time by default), but SeaORM migrations should still be ordered correctly.
**How to avoid:** Migration order: note (after contact + company), tag (no FKs), contact_tag (after contact + tag), interaction (after contact).

### Pitfall 4: Composite Primary Key in Junction Table
**What goes wrong:** SeaORM needs composite primary keys declared correctly for the `contact_tag` junction table.
**Why it happens:** By default, `#[sea_orm(primary_key)]` expects auto-increment. Composite keys need `auto_increment = false` on both columns.
**How to avoid:** Use `#[sea_orm(primary_key, auto_increment = false)]` on both `contact_tag_contact` and `contact_tag_tag` columns.

### Pitfall 5: Expanding the Contact Edit View
**What goes wrong:** The contact form handler already builds a full render. Adding notes, tags, and interaction sections makes it very large.
**Why it happens:** SDUI renders entire views from the backend.
**How to avoid:** Keep the handler well-organized with helper functions for each section (form fields, notes section, tags section, interaction timeline). Compose them into a single Container at the end.

## Code Examples

### Migration: Note Table
```rust
// m20260323_000005_create_note.rs
"CREATE TABLE note (
    note_id INTEGER PRIMARY KEY AUTOINCREMENT,
    note_contact INTEGER REFERENCES contact(contact_id) ON DELETE CASCADE,
    note_company INTEGER REFERENCES company(company_id) ON DELETE CASCADE,
    note_text TEXT NOT NULL,
    note_user INTEGER NOT NULL REFERENCES user(user_id),
    note_created_at TEXT NOT NULL DEFAULT (datetime('now'))
)"
```

### Migration: Tag + Contact_Tag Tables
```rust
// m20260323_000006_create_tag.rs
"CREATE TABLE tag (
    tag_id INTEGER PRIMARY KEY AUTOINCREMENT,
    tag_name TEXT NOT NULL UNIQUE
)"

// m20260323_000007_create_contact_tag.rs
"CREATE TABLE contact_tag (
    contact_tag_contact INTEGER NOT NULL REFERENCES contact(contact_id) ON DELETE CASCADE,
    contact_tag_tag INTEGER NOT NULL REFERENCES tag(tag_id) ON DELETE CASCADE,
    PRIMARY KEY (contact_tag_contact, contact_tag_tag)
)"
```

### Migration: Interaction Table
```rust
// m20260323_000008_create_interaction.rs
"CREATE TABLE interaction (
    interaction_id INTEGER PRIMARY KEY AUTOINCREMENT,
    interaction_contact INTEGER NOT NULL REFERENCES contact(contact_id) ON DELETE CASCADE,
    interaction_type TEXT NOT NULL CHECK(interaction_type IN ('call', 'email', 'meeting')),
    interaction_subject TEXT NOT NULL,
    interaction_notes TEXT,
    interaction_user INTEGER NOT NULL REFERENCES user(user_id),
    interaction_date TEXT NOT NULL,
    interaction_created_at TEXT NOT NULL DEFAULT (datetime('now'))
)"
```

### Tag Color Hash Function
```rust
/// Assign a color from a fixed palette based on tag name hash.
fn tag_color(name: &str) -> &'static str {
    const PALETTE: &[&str] = &[
        "blue", "green", "red", "yellow", "indigo", "purple", "pink", "teal",
    ];
    let hash = name.bytes().fold(0u32, |acc, b| acc.wrapping_mul(31).wrapping_add(u32::from(b)));
    PALETTE[(hash as usize) % PALETTE.len()]
}
```

### Contact List with Search/Filter Payload
```rust
#[derive(Deserialize, Default)]
struct ContactListPayload {
    search: Option<String>,
    company_filter: Option<i32>,
    tag_ids: Option<Vec<i32>>,
    date_from: Option<String>,
    date_to: Option<String>,
}
```

### Tag Save (Auto-Create-If-New)
```rust
/// Find or create a tag by name, return its ID.
async fn find_or_create_tag(db: &DatabaseConnection, name: &str) -> Result<i32, ActionError> {
    use sea_orm::ActiveValue::{NotSet, Set};

    if let Some(existing) = tag::Entity::find()
        .filter(tag::Column::TagName.eq(name.trim()))
        .one(db).await.map_err(|e| ActionError::Internal(e.to_string()))?
    {
        return Ok(existing.tag_id);
    }

    let new_tag = tag::ActiveModel {
        tag_id: NotSet,
        tag_name: Set(name.trim().to_owned()),
    };
    let result = new_tag.insert(db).await
        .map_err(|e| ActionError::Internal(e.to_string()))?;
    Ok(result.tag_id)
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Client-side search/filter | Server-side search/filter via action payloads | Project design decision | All logic stays backend; frontend just sends params |
| Rich text notes | Plain text notes | Phase 8 CONTEXT decision | No WYSIWYG editor needed; keeps scope small |
| Tag management screen | Free-form auto-create tags | Phase 8 CONTEXT decision | No separate tag CRUD; tags created on-the-fly |

## Open Questions

1. **Company name search implementation**
   - What we know: SQL LIKE on contact_name and contact_email is straightforward. Company name is from a JOIN.
   - What's unclear: Whether SeaORM can filter on the related entity's column in `find_also_related`.
   - Recommendation: Use post-filter in Rust after the join query. At demo scale this is fine. Alternatively, do a two-step query: find matching company IDs first, then OR that into the contact condition.

2. **Notes on company detail view**
   - What we know: CRM-06 says "contacts and companies". CONTEXT says notes on both. Claude's discretion mentions this.
   - Recommendation: Yes, add notes section to company detail too. The note table already has nullable FKs for both.

3. **Tag display on contact list rows**
   - What we know: Tags should show as colored chips on the list view.
   - What's unclear: How to render inline tag chips in DataTable rows (DataTable rows are JSON data, not components).
   - Recommendation: Include tag names as a comma-separated string or JSON array in the row data. The frontend DataTable can render arrays as chips if supported, or fall back to a comma-separated string display.

## Sources

### Primary (HIGH confidence)
- Existing codebase: `backend/crates/crm-demo/src/handlers/contact.rs` -- established CRUD handler pattern
- Existing codebase: `backend/crates/crm-demo/src/handlers/company.rs` -- established form + sub-table pattern
- Existing codebase: `backend/crates/crm-demo/src/entities/contact.rs` -- SeaORM entity pattern with relations
- Existing codebase: `backend/crates/crm-demo/src/migration/` -- raw SQL migration pattern
- Existing codebase: `backend/crates/marionette/src/builders/standard.rs` -- all 18 component builders
- `TOOLING.md` -- SQL conventions (prefixed fields, singular tables, FK naming)
- `.planning/phases/08-crm-features/08-CONTEXT.md` -- locked decisions

### Secondary (MEDIUM confidence)
- SeaORM 1.1 `Condition` builder API -- verified via crate version in workspace Cargo.toml, API shape from training data consistent with 1.x

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- no new dependencies, all within established project stack
- Architecture: HIGH -- follows exact patterns from Phase 7 handlers/entities/migrations
- Pitfalls: HIGH -- based on direct code reading of existing patterns and known SeaORM behaviors

**Research date:** 2026-03-23
**Valid until:** 2026-04-23 (stable -- no external dependencies changing)
