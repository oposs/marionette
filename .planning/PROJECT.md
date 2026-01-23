# OpenSDUI + Marionette

## What This Is

OpenSDUI is an open protocol specification for server-driven UI. Marionette is its reference implementation: a Svelte 5 + Flowbite frontend library paired with a Rust + Axum backend toolkit. The backend controls what the frontend renders using three primitives — components, data, and messages. A demo CRM validates the protocol and showcases the implementation quality.

## Core Value

The protocol must be clean, well-specified, and demonstrate that server-driven UI can be done right — enabling rapid business app development where backend developers control UI without requiring frontend expertise.

## Requirements

### Validated

(None yet — ship to validate)

### Active

**Protocol**
- [ ] OpenAPI 3.1 specification defining components, data, messages
- [ ] Protocol manual explaining concepts, patterns, and rationale

**Frontend Library (Marionette Svelte)**
- [ ] Svelte 5 + Flowbite component library
- [ ] Renders component adjacency lists from server
- [ ] Handles data binding via JSON Pointers
- [ ] Handles message passing to/from server

**Backend Toolkit (Marionette Rust)**
- [ ] Axum integration for serving SDUI responses
- [ ] SeaORM patterns for data persistence
- [ ] Rust macros for ergonomic component construction

**Demo CRM**
- [ ] Multi-user authentication with roles
- [ ] Companies management (CRUD)
- [ ] Contacts management (CRUD, belong to companies)
- [ ] Interactions/activity tracking per contact
- [ ] Listmonk integration: sync contacts to lists
- [ ] Listmonk integration: view mailing history per contact

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
| Three primitives: Components, Data, Messages | Minimal surface area, covers all SDUI needs | — Pending |
| Components as flat adjacency list | Simpler than nested trees, easier to diff and patch, inspired by A2UI | — Pending |
| Data binding via JSON Pointers | Standard, well-understood, enables precise targeting | — Pending |
| Stateless protocol | Simplifies debugging, enables horizontal scaling, session state lives in backend | — Pending |
| Component types as open strings | Extensibility without protocol changes, frontend validates | — Pending |
| No capability negotiation | Frontend library bundled with app, always knows full component set | — Pending |
| Rust macros for component construction | Ergonomic DX, avoids verbose JSON/YAML in backend code | — Pending |

---
*Last updated: 2026-01-23 after initialization*
