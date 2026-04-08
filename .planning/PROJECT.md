# OpenSDUI + Marionette

## What This Is

OpenSDUI is an open protocol specification for server-driven UI. Marionette is its reference implementation: a Svelte 5 + Flowbite frontend library paired with a Rust + Axum backend toolkit. The backend controls what the frontend renders using three primitives — components, data, and messages. A demo CRM validates the protocol end-to-end with authentication, CRUD, search/filtering, and Listmonk integration.

## Core Value

The protocol must be clean, well-specified, and demonstrate that server-driven UI can be done right — enabling rapid business app development where backend developers control UI without requiring frontend expertise.

## Current State

**Shipped:** v1.0 MVP (2026-04-08)

**What's built:**
- OpenAPI 3.1 protocol specification (6 message types, adjacency list components, JSON Pointer data binding)
- Svelte 5 frontend library: 20+ SDUI components, reactive data store, dirty tracking, WebSocket transport, URL routing
- Rust backend toolkit: derive macros, action routing, WebSocket sessions, SeaORM persistence
- CRM demo: auth/roles, company/contact CRUD, notes, tags, search/filtering, interaction timeline, Listmonk sync

**Tech stack:** Rust (Axum, SeaORM, tokio), Svelte 5 (Flowbite, Vite), SQLite, ~53k LOC across 273 files

## Requirements

### Validated

- ✓ OpenAPI 3.1 specification defining components, data, messages — v1.0
- ✓ Protocol manual explaining concepts, patterns, and rationale — v1.0
- ✓ Svelte 5 + Flowbite component library — v1.0
- ✓ Renders component adjacency lists from server — v1.0
- ✓ Handles data binding via JSON Pointers — v1.0
- ✓ Handles message passing to/from server — v1.0
- ✓ Axum integration for serving SDUI responses — v1.0
- ✓ SeaORM patterns for data persistence — v1.0
- ✓ Rust macros for ergonomic component construction — v1.0
- ✓ Multi-user authentication with roles — v1.0
- ✓ Companies management (CRUD) — v1.0
- ✓ Contacts management (CRUD, belong to companies) — v1.0
- ✓ Interactions/activity tracking per contact — v1.0
- ✓ Listmonk integration: sync contacts to lists — v1.0
- ✓ Listmonk integration: view mailing history per contact — v1.0

### Active

(None yet — define for next milestone)

### Out of Scope

- Complex workflow engine — simple navigation and forms only for v1
- Offline support — always-connected assumption
- Mobile-native implementations — web-first, Svelte only for v1
- Custom component authoring by end users — fixed component library

## Context

**Prior Art:**
- CallBackery (Perl-based SDUI) — learned from its inconsistencies and ad-hoc evolution
- A2UI's adjacency list pattern — adopted for component tree representation
- Existing SDUI solutions (Lona, Server-Driven UI frameworks) — mostly proprietary or incomplete

**Design Principles:**
- LLM-friendliness: protocol should be easy for AI to understand and generate
- Stateless protocol: no session state embedded in the protocol itself
- "Puppet master / smart puppet" mental model: backend orchestrates, frontend renders intelligently
- Component types as open strings (not enums) for extensibility
- No capability negotiation: frontend library is bundled, knows all components

**Purpose:**
- Showcase company's ability to deliver first-rate code
- Fill the gap in open source SDUI solutions
- Enable internal use for future projects

## Constraints

- **Tech Stack (Backend)**: Rust, Axum, SeaORM, utoipa, tokio — per TOOLING.md
- **Tech Stack (Frontend)**: Svelte 5, Flowbite, Vite — per TOOLING.md
- **Build System**: Makefile-based builds with standard targets
- **Testing**: Unit (Vitest), Component (Playwright component), E2E (Playwright)
- **CI/CD**: GitHub Actions workflows
- **Quality Bar**: Showcase-quality — clean architecture, comprehensive tests, documentation that explains thinking

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Three primitives: Components, Data, Messages | Minimal surface area, covers all SDUI needs | ✓ Good |
| Components as flat adjacency list | Simpler than nested trees, easier to diff and patch, inspired by A2UI | ✓ Good |
| Data binding via JSON Pointers | Standard, well-understood, enables precise targeting | ✓ Good |
| Stateless protocol | Simplifies debugging, enables horizontal scaling, session state lives in backend | ✓ Good |
| Component types as open strings | Extensibility without protocol changes, frontend validates | ✓ Good |
| No capability negotiation | Frontend library bundled with app, always knows full component set | ✓ Good |
| Rust macros for component construction | Ergonomic DX, avoids verbose JSON/YAML in backend code | ✓ Good |
| serde tagged enum for protocol messages | Clean serialization with type discriminator | ✓ Good |
| mpsc channel for WebSocket reader/writer | Avoids Arc<Mutex> complexity on sender | ✓ Good |
| Type-erased AppState extension | Avoids leaking app types into library crate | ✓ Good |
| OnceLock for external service clients | Simple global access from handlers | ⚠️ Revisit — consider DI pattern for testability |
| SQLite for demo CRM | Simplest persistence for demo, zero config | ✓ Good |

---
*Last updated: 2026-04-08 after v1.0 milestone*
