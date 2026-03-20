# Roadmap: OpenSDUI + Marionette

## Overview

This roadmap delivers OpenSDUI protocol and Marionette implementation through 9 phases: infrastructure setup, protocol specification, frontend library with tests, backend toolkit with tests, Axum-Svelte integration, then a 4-phase CRM demo covering authentication/foundation, core CRUD, enhanced features, and Listmonk integration. Testing is bundled with implementation phases rather than separate, following the principle that tests validate the code they accompany.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [ ] **Phase 1: Project Infrastructure** - Makefile, CI/CD workflows, project structure, dev tooling
- [x] **Phase 2: Protocol Specification** - OpenAPI 3.1 spec + protocol manual defining messages, components, data binding (completed 2026-03-18)
- [ ] **Phase 3: Frontend Library** - Svelte 5 + Flowbite infrastructure, components, and tests (Vitest, vitest-browser-svelte, Playwright)
- [ ] **Phase 4: Backend Toolkit** - Rust + Axum infrastructure, macros, and tests (cargo test)
- [ ] **Phase 5: Integration** - Axum serves Svelte app, end-to-end protocol validation
- [ ] **Phase 6: CRM Auth & Foundation** - User management, roles, authentication, audit trail
- [ ] **Phase 7: CRM Core** - Contacts, companies, CRUD, data tables, forms
- [ ] **Phase 8: CRM Features** - Notes, tags, search, filtering, interactions
- [ ] **Phase 9: CRM Listmonk** - Subscriber sync and mailing history integration

## Phase Details

### Phase 1: Project Infrastructure
**Goal**: Development environment is fully operational with build system, CI/CD, and project structure
**Depends on**: Nothing (first phase)
**Requirements**: INFRA-01, INFRA-02, INFRA-03, INFRA-04, INFRA-05
**Success Criteria** (what must be TRUE):
  1. `make dev` starts both frontend and backend development servers
  2. `make test` runs all test suites (even if empty initially)
  3. `make lint` checks code quality for Rust and Svelte
  4. `make build` produces production-ready artifacts
  5. GitHub Actions run tests and builds on every PR
**Plans**: 3 plans

Plans:
- [ ] 01-01-PLAN.md -- Scaffold project directory structure (backend workspace + SvelteKit frontend + spec)
- [ ] 01-02-PLAN.md -- Create Makefile with all standard targets (dev, build, test, lint, clean, format)
- [ ] 01-03-PLAN.md -- Configure ESLint flat config and GitHub Actions CI workflow

### Phase 2: Protocol Specification
**Goal**: Complete OpenSDUI protocol specification that any implementation can follow
**Depends on**: Phase 1
**Requirements**: PROT-01, PROT-02, PROT-03, PROT-04, PROT-05, PROT-06, PROT-07, PROT-08, PROT-09, PROT-10, PROT-11, PROT-12, PROT-13, PROT-14, DOC-01, DOC-02
**Success Criteria** (what must be TRUE):
  1. OpenAPI 3.1 spec validates and can be viewed in Swagger UI
  2. All message types (render, patch, action, event) are fully specified with examples
  3. Component adjacency list structure is specified with JSON Schema
  4. Data binding via JSON Pointer is documented with keyed collection patterns
  5. Protocol manual explains concepts clearly enough for a developer to implement from scratch
**Plans**: 3 plans

Plans:
- [ ] 02-01-PLAN.md -- Spec tooling, JSON Schema definitions (common, component, data, message), OpenAPI 3.1 entry point
- [ ] 02-02-PLAN.md -- Protocol manual (spec/PROTOCOL.md) with transport, messages, data binding, worked examples
- [ ] 02-03-PLAN.md -- Example YAML files for all message types + human verification of spec rendering

### Phase 3: Frontend Library
**Goal**: Complete Marionette Svelte library with all infrastructure, components, and comprehensive tests
**Depends on**: Phase 2
**Requirements**: FRONT-01, FRONT-02, FRONT-03, FRONT-04, FRONT-05, FRONT-06, FRONT-07, FRONT-08, FRONT-10, FRONT-11, FRONT-12, FRONT-13, FRONT-14, FRONT-15, FRONT-16, FRONT-20, FRONT-21, FRONT-22, FRONT-23, FRONT-24, FRONT-25, FRONT-26, FRONT-27
**Success Criteria** (what must be TRUE):
  1. Frontend can render any valid component adjacency list from the protocol spec
  2. Data binding updates UI reactively when data changes via JSON Pointer paths
  3. Dirty field tracking prevents server patches from clobbering actively edited fields
  4. All component types (nav, form, table, popup, feedback) render with Flowbite styling
  5. WebSocket reconnects automatically after connection loss
  6. Vitest unit tests pass for data store, message handling, and component logic
  7. vitest-browser-svelte component tests validate each component in real browser
  8. Playwright E2E framework is configured and ready for integration tests
**Plans**: 6 plans

Plans:
- [ ] 03-01-PLAN.md -- Install deps, test infra, TypeScript protocol types, reactive data store with dirty tracking and optimistic updates + unit tests
- [ ] 03-02-PLAN.md -- WebSocket transport with reconnection, message dispatcher, URL routing + unit tests
- [ ] 03-03-PLAN.md -- Component registry, NodeRenderer, Surface, ErrorBoundary, FallbackComponent, init module, library exports
- [ ] 03-04-PLAN.md -- Navigation components (SideNav, NavItem, NavGroup), layout components (Container, Grid, Heading, Text), feedback components (Spinner, ErrorDisplay)
- [ ] 03-05-PLAN.md -- Form components (Form, TextInput, Select, Checkbox, Button), DataTable with virtual scroll, popup components (Modal, Toast, ConfirmDialog)
- [ ] 03-06-PLAN.md -- Browser component tests (vitest-browser-svelte), Playwright E2E framework, visual regression screenshots, demo page

### Phase 4: Backend Toolkit
**Goal**: Complete Marionette Rust toolkit with all infrastructure, macros, and comprehensive tests
**Depends on**: Phase 2
**Requirements**: BACK-01, BACK-02, BACK-03, BACK-04, BACK-05, BACK-06, BACK-07, BACK-10, BACK-11, BACK-12, BACK-13, BACK-14, BACK-15
**Success Criteria** (what must be TRUE):
  1. Rust macros enable ergonomic component construction without verbose JSON
  2. Axum handlers can serve render and patch responses following protocol spec
  3. WebSocket sessions maintain connection state and handle reconnection
  4. Action routing dispatches incoming actions to appropriate handlers
  5. SeaORM patterns are established for entity persistence
  6. `cargo test` passes for all component builders, message encoding, and action routing
  7. Integration tests validate Axum handlers respond correctly
  8. WebSocket session tests verify connection lifecycle
**Plans**: TBD

Plans:
- [ ] 04-01: TBD

### Phase 5: Integration
**Goal**: Frontend and backend work together end-to-end with Axum serving the Svelte app
**Depends on**: Phase 3, Phase 4
**Requirements**: INTEG-01, INTEG-02, INTEG-03
**Success Criteria** (what must be TRUE):
  1. Axum serves the built Svelte application as static files
  2. Frontend can send actions and receive render/patch responses from backend
  3. WebSocket connection establishes and maintains communication
  4. End-to-end flow works: backend sends component tree, frontend renders, user interacts, action dispatches, backend responds
  5. Protocol messages match specification exactly (validated by comparing against OpenAPI schemas)
**Plans**: TBD

Plans:
- [ ] 05-01: TBD

### Phase 6: CRM Auth & Foundation
**Goal**: Users can securely access the CRM with role-based permissions
**Depends on**: Phase 5
**Requirements**: CRM-12, CRM-13, CRM-14
**Success Criteria** (what must be TRUE):
  1. User can log in with username/password and stay logged in across sessions
  2. Admin can create, view, edit, and delete user accounts
  3. Admin can assign roles (admin, user) to control access levels
  4. System records who changed what and when (audit trail queryable)
  5. Unauthorized users are denied access to protected features
**Plans**: TBD

Plans:
- [ ] 06-01: TBD

### Phase 7: CRM Core
**Goal**: Users can manage contacts and companies with full CRUD operations
**Depends on**: Phase 6
**Requirements**: CRM-01, CRM-02, CRM-03, CRM-04, CRM-05
**Success Criteria** (what must be TRUE):
  1. User can create, view, edit, and delete contacts
  2. User can create, view, edit, and delete companies
  3. User can link contacts to companies and view the relationship
  4. Data tables display contacts and companies with sorting and pagination
  5. Form views allow editing all fields with validation feedback
**Plans**: TBD

Plans:
- [ ] 07-01: TBD

### Phase 8: CRM Features
**Goal**: CRM has notes, tagging, search, filtering, and interaction tracking
**Depends on**: Phase 7
**Requirements**: CRM-06, CRM-07, CRM-08, CRM-09, CRM-10, CRM-11
**Success Criteria** (what must be TRUE):
  1. User can add notes to contacts and companies with timestamps
  2. User can search contacts by name, email, or company name
  3. User can create tags and apply them to contacts
  4. User can filter contact lists by company, tags, or date range
  5. User can log interactions (calls, emails, meetings) on contacts
  6. User can view chronological interaction timeline per contact
**Plans**: TBD

Plans:
- [ ] 08-01: TBD

### Phase 9: CRM Listmonk
**Goal**: CRM integrates with Listmonk for newsletter management
**Depends on**: Phase 8
**Requirements**: CRM-15, CRM-16
**Success Criteria** (what must be TRUE):
  1. User can sync selected contacts to Listmonk subscriber lists
  2. User can view mailing campaign history per contact from Listmonk
  3. Sync status indicates success/failure with error details
  4. Contact changes propagate to Listmonk (create, update, unsubscribe)
**Plans**: TBD

Plans:
- [ ] 09-01: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 1 -> 2 -> 3 -> 4 -> 5 -> 6 -> 7 -> 8 -> 9

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Project Infrastructure | 0/3 | Planning complete | - |
| 2. Protocol Specification | 3/3 | Complete   | 2026-03-18 |
| 3. Frontend Library | 0/6 | Planning complete | - |
| 4. Backend Toolkit | 0/TBD | Not started | - |
| 5. Integration | 0/TBD | Not started | - |
| 6. CRM Auth & Foundation | 0/TBD | Not started | - |
| 7. CRM Core | 0/TBD | Not started | - |
| 8. CRM Features | 0/TBD | Not started | - |
| 9. CRM Listmonk | 0/TBD | Not started | - |

---
*Created: 2026-01-24*
