# Milestones

## v1.0 MVP (Shipped: 2026-04-08)

**Phases completed:** 9 phases, 32 plans, 62 tasks

**Key accomplishments:**

- Cargo workspace with 4 crates (protocol, macros, lib, crm-demo) and SvelteKit frontend with Tailwind v4 + Flowbite Svelte, all compiling and building cleanly
- Top-level Makefile with dev/build/test/lint/clean/format targets orchestrating Rust backend and SvelteKit frontend
- ESLint flat config for Svelte + TypeScript and GitHub Actions CI with parallel frontend/backend jobs, npm and Cargo caching
- OpenAPI 3.1 spec with tagged union for 6 message types, component adjacency list schema, and JSON Pointer data binding using Redocly lint tooling
- Authoritative OpenSDUI protocol manual with 12 sections covering transport, messages, components, data binding, keyed collections, optimistic updates, and error handling
- Six realistic YAML example files covering all protocol message types (hello, render, patch, action, event, error) with CRM contact management theme, validated against OpenAPI spec
- Reactive data store with JSON Pointer binding, dirty field tracking, optimistic snapshot/rollback, and 20 passing unit tests using json-ptr and Svelte 5 $state runes
- WebSocket transport with exponential backoff reconnection, message dispatcher routing by type, and URL router with history.pushState sync -- 24 unit tests passing
- Component registry, recursive NodeRenderer, surface containers, error boundaries, and initMarionette wiring store+transport+rendering together
- 9 Flowbite-wrapped SDUI components (nav, layout, feedback) with data binding and navigate action dispatch, registered in default registry
- 10 Svelte SDUI components: form inputs with dirty tracking, virtual-scroll data table with sort, modal/toast/confirm popups, all registered in component registry
- 19 browser component tests in Chromium via vitest-browser-svelte, Playwright E2E smoke tests, visual regression baselines for sidebar/form/table/full-page, and demo page rendering all component types without backend
- Serde-tagged ProtocolMessage enum with 6 variants, Component/Data structs, and 15 round-trip tests matching OpenAPI spec
- ComponentBuilder derive macro with darling, action/requires attribute macros, and all 18 standard component builders with 12 passing tests
- ActionRouter with name-based dispatch, typed extractors (Payload/Db/Session), and auth checking (None/Authenticated/Role)
- Axum WebSocket handler with mpsc channel pattern, session tracking, hello/action/error message flow, and 5 integration tests
- SeaORM entity framework with SQLite persistence, migration runner, session entity following SQL conventions, and in-memory test DB pattern
- Axum server serving SvelteKit static files with SPA fallback, WebSocket dispatch with hello handling, and demo navigate/click round-trip
- Playwright E2E tests validating WebSocket round-trip, component rendering, SPA fallback, and protocol conformance against OpenAPI schemas using AJV
- Bcrypt login with HTTP-only session cookies, WebSocket cookie auth on upgrade, and SDUI login form for unauthenticated sessions
- Admin-only user CRUD handlers with DataTable list, create/edit form, delete protection, and role-based sidebar nav
- Automatic audit trail with record_audit helper, field-level JSON diffs, and admin-only filterable audit log viewer
- Company and contact SeaORM entities with FK relation, SQLite migrations, and demo seed data for CRM core
- Company list/form/save/delete handlers with contact count, audit logging, and sidebar navigation for all authenticated users
- Contact CRUD with company FK joins, select dropdown, linked sub-table, and default view delegation
- 4 new SeaORM entities (note, tag, contact_tag, interaction) with migrations and seed data for CRM feature enrichment
- Append-only note_save handler with notes section integrated into contact and company edit forms showing author, timestamp, and text
- Server-side contact search/filtering with SeaORM Condition builder, tag display in list rows, and free-form tag editing on contact form
- Interaction logging form (call/email/meeting) with timeline DataTable on contact detail, handler delegation for post-save re-render
- ListmonkClient HTTP wrapper with reqwest, listmonk_sync/cache entities and migrations, type-erased AppState extension field
- Contact sync handlers with tag-to-list mapping, sync status badges, blocklist-on-delete, and email-change propagation using wiremock tests
- Per-contact mailing history from Listmonk subscriber export with 15-minute local cache, DataTable display, and on-demand refresh

---

## v1.1 shadcn-svelte + High-Level Components (Shipped: 2026-04-18)

**Phases completed:** 6 phases (10–15), 38 plans

**Key accomplishments:**

- shadcn-svelte initialised as the sole component framework; all Flowbite packages, imports, and plugin references removed; app.css rewritten to OKLCH semantic colour tokens on the Zinc palette with shadcn theme plumbing; CI guard prevents Flowbite regressions
- All 20+ SDUI leaf components (Button, TextInput, SelectInput, Checkbox, Textarea, RadioGroup, Switch, ModalSurface, ConfirmDialog, ToastSurface, DataTable, NavItem, Sidebar pieces, feedback primitives, FormScreen/TableScreen) rebuilt on shadcn-svelte primitives and lucide-svelte icons with passing component tests
- Protocol version bumped to 1.1.0 with node-patch operations (`set-node`, `delete-node`, `set-children`) applied atomically alongside data ops; spec YAMLs and PROTOCOL.md updated to match; focus-preservation proven via browser test
- AppShell delivered as a first-class SDUI component on top of shadcn Sidebar — collapsible desktop sidebar, mobile sheet, header/footer slots, `--sidebar-*` token theming — registered in defaults.ts with hand-written backend builder and slot children by name
- Surface-scoped patches (SurfaceMount + fine-grained surfaces store) so a patch to one surface does not remount siblings; retired the legacy ConnectionBanner
- DataTable rewritten to the shadcn recipe shape: server-driven filter bar, virtualised infinite scroll via `createRuneVirtualizer` wrapper around `@tanstack/virtual-core`, column visibility dropdown, per-kind cell rendering, stale-response discard, generic `fetch_rows` handler with source dispatch; CRM list handlers migrated; TableScreen retired
- Form primitives rewritten around a shared Field.Field anatomy (label + description + error) with FieldSet (responsive 2-column grid + cols override) and FieldSeparator structural primitives; Textarea, RadioGroup, Switch added; FormScreen orphan deleted
- CRM demo fully migrated to the new stack — contact schema extended (country/notes/opt_in), handler sweeps across company/user/interaction/contact/note/tag forms rewired for validation, Form payload + Button builder + node: prefix scope-closure bundle, documentation brand-voice sweep, E2E + Chrome-MCP UAT across every screen

---
