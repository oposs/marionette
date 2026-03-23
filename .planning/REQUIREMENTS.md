# Requirements

## v1 Requirements

### Project Infrastructure (INFRA)
*Build system, CI/CD, and development environment*

- [x] **INFRA-01**: Makefile with standard targets (dev, build, test, lint, clean)
- [x] **INFRA-02**: Project directory structure (frontend/, backend/, spec/)
- [x] **INFRA-03**: GitHub Actions workflow for CI (test, lint, build on PR)
- [x] **INFRA-04**: Development server configuration (frontend + backend concurrent)
- [x] **INFRA-05**: Code formatting and linting configuration (Rust + Svelte)

### Protocol Specification (PROT)
*The universal contract — what any OpenSDUI implementation must follow*

- [x] **PROT-01**: Message envelope format (type, payload, correlation ID)
- [x] **PROT-02**: Component structure (id, type, props, children, bind, action)
- [x] **PROT-03**: Adjacency list pattern (flat nodes with ID references, root pointer)
- [x] **PROT-04**: Data binding via JSON Pointer (RFC 6901)
- [x] **PROT-05**: Keyed collections pattern (stable keys, not array indices)
- [x] **PROT-06**: Render message type (backend -> frontend: components + data)
- [x] **PROT-07**: Patch message type (backend -> frontend: data updates)
- [x] **PROT-08**: Action message type (frontend -> backend: user interactions)
- [x] **PROT-09**: Event message type (backend -> frontend: notifications)
- [x] **PROT-10**: Error format (path + message, errors as data)
- [x] **PROT-11**: Surface concept (named render targets)
- [x] **PROT-12**: Optimistic update mechanism (action includes optimistic patch)
- [x] **PROT-13**: REST endpoint definitions
- [x] **PROT-14**: WebSocket transport definition

### Protocol Documentation (DOC)

- [x] **DOC-01**: OpenAPI 3.1 specification for all protocol messages
- [x] **DOC-02**: Protocol manual explaining concepts, patterns, and rationale

### Frontend Library — Marionette Svelte (FRONT)
*The component vocabulary and rendering engine*

**Core Infrastructure:**
- [x] **FRONT-01**: Reactive data store with JSON Pointer binding
- [x] **FRONT-02**: Component registry with dynamic rendering from adjacency list
- [x] **FRONT-03**: Message handling (send actions, receive renders/patches/events)
- [x] **FRONT-04**: Multi-surface renderer (main, sidebar, modal, toast)
- [x] **FRONT-05**: WebSocket connection management with reconnection
- [x] **FRONT-06**: Optimistic update handling with rollback on failure
- [x] **FRONT-07**: Dirty field tracking (skip patches to actively edited fields)
- [x] **FRONT-08**: URL routing (reflect route in URL, handle browser nav)

**Component Vocabulary:**
- [x] **FRONT-10**: Navigation components (side-nav, nav-item, nav-group)
- [x] **FRONT-11**: Form components (form, text-input, select, checkbox, button)
- [x] **FRONT-12**: Layout components (container, grid/flex, heading, text)
- [x] **FRONT-13**: Table components (data-table: sortable, paginated, keyed rows)
- [x] **FRONT-14**: Popup components (modal, toast, confirm-dialog)
- [x] **FRONT-15**: Feedback components (spinner/loading, error display)
- [x] **FRONT-16**: Flowbite styling integration

**Testing Infrastructure:**
- [x] **FRONT-20**: Unit test framework (Vitest) for component logic
- [x] **FRONT-21**: Component tests using vitest-browser-svelte + Playwright (real browser)
- [x] **FRONT-22**: Data store unit tests (binding, patching, dirty tracking)
- [x] **FRONT-23**: Message handling unit tests (action dispatch, render processing)
- [x] **FRONT-24**: E2E test framework (Playwright) for user flows
- [x] **FRONT-25**: Visual regression testing with Playwright screenshots
- [x] **FRONT-26**: Component visual snapshots (each component state captured)
- [x] **FRONT-27**: Full-page visual snapshots for key screens

### Backend Toolkit — Marionette Rust (BACK)
*The component builders and server infrastructure*

**Core Infrastructure:**
- [x] **BACK-01**: Axum handlers for serving SDUI responses
- [x] **BACK-02**: Rust macros for ergonomic component construction
- [x] **BACK-03**: Protocol message encoding/decoding
- [x] **BACK-04**: Action routing and handler dispatch
- [x] **BACK-05**: SeaORM entity patterns for persistence
- [x] **BACK-06**: WebSocket session management
- [x] **BACK-07**: Permission/authorization utilities

**Testing Infrastructure:**
- [x] **BACK-10**: Unit test framework for component builders and macros
- [x] **BACK-11**: Unit tests for message encoding/decoding
- [x] **BACK-12**: Unit tests for action routing and dispatch
- [x] **BACK-13**: Integration tests for Axum handlers
- [x] **BACK-14**: Integration tests for WebSocket session management
- [x] **BACK-15**: SeaORM entity tests with test database

### Integration (INTEG)
*Frontend-backend end-to-end validation*

- [x] **INTEG-01**: Axum serves built Svelte app as static files
- [x] **INTEG-02**: End-to-end message flow (action -> backend -> render -> frontend)
- [x] **INTEG-03**: Protocol conformance validation against OpenAPI schemas

### Demo CRM (CRM)
*Proof that OpenSDUI + Marionette works for real apps*

- [ ] **CRM-01**: User can create, view, edit, delete contacts
- [ ] **CRM-02**: User can create, view, edit, delete companies
- [ ] **CRM-03**: User can link contacts to companies
- [ ] **CRM-04**: User can view paginated, sortable data tables
- [ ] **CRM-05**: User can view and edit records in form views
- [ ] **CRM-06**: User can add notes to contacts and companies
- [ ] **CRM-07**: User can search contacts by name, email, company
- [ ] **CRM-08**: User can tag/label contacts for categorization
- [ ] **CRM-09**: User can filter lists by company, tag, date range
- [ ] **CRM-10**: User can log interactions (calls, emails, meetings) per contact
- [ ] **CRM-11**: User can view interaction timeline per contact
- [ ] **CRM-12**: Admin can manage users and assign roles
- [ ] **CRM-13**: User can log in and access features based on role
- [ ] **CRM-14**: System records audit trail (who changed what when)
- [ ] **CRM-15**: User can sync contacts to Listmonk subscriber lists
- [ ] **CRM-16**: User can view mailing history per contact from Listmonk

---

## v2 Requirements (Deferred)

- [ ] Custom fields on contacts/companies
- [ ] Bulk operations (tag, export, delete)
- [ ] Import/Export (CSV)
- [ ] Template system for reusable component patterns

---

## Out of Scope

- **Sales pipeline/deals** — separate tool, scope creep
- **Full email client** — complexity, deliverability issues
- **Calendar integration** — OAuth complexity
- **AI auto-logging** — unreliable, privacy concerns
- **Social media tracking** — API restrictions, legal issues
- **Complex workflows** — requires workflow engine
- **Built-in calling** — telecom complexity
- **Mobile native** — web-first, Svelte only
- **Offline support** — always-connected assumption

---

## Traceability

| REQ-ID | Phase | Status |
|--------|-------|--------|
| INFRA-01 | Phase 1 | Complete |
| INFRA-02 | Phase 1 | Complete |
| INFRA-03 | Phase 1 | Complete |
| INFRA-04 | Phase 1 | Complete |
| INFRA-05 | Phase 1 | Complete |
| PROT-01 | Phase 2 | Complete |
| PROT-02 | Phase 2 | Complete |
| PROT-03 | Phase 2 | Complete |
| PROT-04 | Phase 2 | Complete |
| PROT-05 | Phase 2 | Complete |
| PROT-06 | Phase 2 | Complete |
| PROT-07 | Phase 2 | Complete |
| PROT-08 | Phase 2 | Complete |
| PROT-09 | Phase 2 | Complete |
| PROT-10 | Phase 2 | Complete |
| PROT-11 | Phase 2 | Complete |
| PROT-12 | Phase 2 | Complete |
| PROT-13 | Phase 2 | Complete |
| PROT-14 | Phase 2 | Complete |
| DOC-01 | Phase 2 | Complete |
| DOC-02 | Phase 2 | Complete |
| FRONT-01 | Phase 3 | Complete |
| FRONT-02 | Phase 3 | Complete |
| FRONT-03 | Phase 3 | Complete |
| FRONT-04 | Phase 3 | Complete |
| FRONT-05 | Phase 3 | Complete |
| FRONT-06 | Phase 3 | Complete |
| FRONT-07 | Phase 3 | Complete |
| FRONT-08 | Phase 3 | Complete |
| FRONT-10 | Phase 3 | Complete |
| FRONT-11 | Phase 3 | Complete |
| FRONT-12 | Phase 3 | Complete |
| FRONT-13 | Phase 3 | Complete |
| FRONT-14 | Phase 3 | Complete |
| FRONT-15 | Phase 3 | Complete |
| FRONT-16 | Phase 3 | Complete |
| FRONT-20 | Phase 3 | Complete |
| FRONT-21 | Phase 3 | Complete |
| FRONT-22 | Phase 3 | Complete |
| FRONT-23 | Phase 3 | Complete |
| FRONT-24 | Phase 3 | Complete |
| FRONT-25 | Phase 3 | Complete |
| FRONT-26 | Phase 3 | Complete |
| FRONT-27 | Phase 3 | Complete |
| BACK-01 | Phase 4 | Complete |
| BACK-02 | Phase 4 | Complete |
| BACK-03 | Phase 4 | Complete |
| BACK-04 | Phase 4 | Complete |
| BACK-05 | Phase 4 | Complete |
| BACK-06 | Phase 4 | Complete |
| BACK-07 | Phase 4 | Complete |
| BACK-10 | Phase 4 | Complete |
| BACK-11 | Phase 4 | Complete |
| BACK-12 | Phase 4 | Complete |
| BACK-13 | Phase 4 | Complete |
| BACK-14 | Phase 4 | Complete |
| BACK-15 | Phase 4 | Complete |
| INTEG-01 | Phase 5 | Complete |
| INTEG-02 | Phase 5 | Complete |
| INTEG-03 | Phase 5 | Complete |
| CRM-12 | Phase 6 | Pending |
| CRM-13 | Phase 6 | Pending |
| CRM-14 | Phase 6 | Pending |
| CRM-01 | Phase 7 | Pending |
| CRM-02 | Phase 7 | Pending |
| CRM-03 | Phase 7 | Pending |
| CRM-04 | Phase 7 | Pending |
| CRM-05 | Phase 7 | Pending |
| CRM-06 | Phase 8 | Pending |
| CRM-07 | Phase 8 | Pending |
| CRM-08 | Phase 8 | Pending |
| CRM-09 | Phase 8 | Pending |
| CRM-10 | Phase 8 | Pending |
| CRM-11 | Phase 8 | Pending |
| CRM-15 | Phase 9 | Pending |
| CRM-16 | Phase 9 | Pending |

---
*Last updated: 2026-01-24 after roadmap regeneration*
