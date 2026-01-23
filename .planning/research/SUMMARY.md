# Project Research Summary

**Project:** OpenSDUI Protocol + Marionette Reference Implementation
**Domain:** Server-Driven UI (SDUI) Framework and CRM Demo Application
**Researched:** 2026-01-23
**Confidence:** MEDIUM-HIGH

## Executive Summary

This project develops OpenSDUI, an open protocol specification for server-driven UI, along with Marionette, a reference implementation combining a Rust backend with a Svelte 5 frontend to demonstrate the protocol through a simple CRM application. The research validates this is a production-ready stack with mature tooling: Rust 1.85+ with Axum 0.8, SeaORM 1.1, and utoipa 5.4 on the backend; Svelte 5.48 with Flowbite and Tailwind CSS 4.0 on the frontend.

The recommended approach follows a flat adjacency list component model (not nested trees) with JSON Pointer data binding, optimized for LLM generation, incremental patching, and streaming updates. The architecture separates protocol concerns from business logic cleanly, with both backend and frontend having parallel protocol implementation layers. The CRM demo should start minimal (contacts, companies, basic CRUD) and prove the SDUI pattern before adding advanced features like real-time WebSocket updates or complex permission models.

Critical risks center on state synchronization race conditions (dirty field tracking is essential), component granularity mismatches (60fps interactions need leaf components with internal state), and async Rust pitfalls (blocking operations starve Tokio runtime). Mitigation strategies are well-documented: implement dirty field detection from day one, design component boundaries thoughtfully, use spawn_blocking for CPU work, and build debugging tools alongside features not as afterthoughts.

## Key Findings

### Recommended Stack

The stack is mature and battle-tested, with all major versions released in the last 6 months. Rust with Axum provides memory safety and excellent async performance for the protocol server. SeaORM offers async-first ORM capabilities supporting both SQLite (development) and PostgreSQL (production). Svelte 5's new runes system provides predictable reactivity ideal for SDUI data binding. The combination is production-ready with high confidence.

**Core technologies:**
- **Rust 1.85+ with Axum 0.8:** Memory-safe async server, excellent for protocol correctness, new path syntax cleaner than 0.7
- **SeaORM 1.1:** Async-first ORM with migrations, query builder DSL, supports both SQLite and PostgreSQL
- **utoipa 5.4:** Code-first OpenAPI 3.1 generation from Rust types, integrates cleanly with Axum
- **Svelte 5.48:** Runes system ($state, $derived, $effect) provides explicit reactivity perfect for data binding
- **Flowbite Svelte 1.31:** 63+ accessible Tailwind components, MIT licensed, reduces custom component work
- **Tailwind CSS 4.0:** CSS-first config, 5x faster builds than v3, modern CSS features

**Critical version notes:**
- Use Svelte 5 runes exclusively, not Svelte 4 patterns (runes enable cleaner state management)
- Use Tailwind 4.0 for performance improvements and modern CSS
- Avoid Diesel (sync-only, incompatible with Axum async), use SeaORM
- Use rustls not native-tls (pure Rust, consistent across platforms)

### Expected Features

Research identified clear table stakes vs. differentiators for both the SDUI protocol and the CRM demo. The protocol MVP requires 8-9 core features; anything beyond that is enhancement territory. The CRM should remain simple to keep focus on demonstrating the SDUI pattern, not building a complete CRM product.

**Must have (table stakes) - SDUI Protocol:**
- Component rendering (text, container, button, image minimum)
- Data binding via JSON Pointer (RFC 6901)
- Form inputs (text-input, select, checkbox, button)
- Action handling (click, submit, navigate)
- Layout containers (grid, flex, row, column)
- Error display (errors as bound data paths)
- Loading states (bind to boolean paths)
- Flat adjacency list component model (not nested trees)

**Must have (table stakes) - CRM Demo:**
- Contact records (CRUD: name, email, phone, company link)
- Company records (CRUD with linked contacts)
- Notes (timestamp + text on records)
- Data tables (sortable, paginated lists)
- Search (by name, email, company)
- Basic user authentication (login, sessions)
- Record detail views (form-based edit)

**Should have (competitive advantage):**
- Real-time updates via WebSocket (push UI changes without polling)
- Optimistic updates (actions include optimistic patches for responsive feel)
- Multi-surface rendering (main, modal, toast, sidebar)
- Listmonk integration (CRM differentiator: newsletter management built-in)
- Interaction history (chronological timeline of touchpoints)
- Tags/labels (flexible categorization)

**Defer to v2+ (adds complexity without proportional MVP value):**
- Template system (wait for reusable patterns to emerge organically)
- Custom fields (add after user feedback, not speculation)
- Bulk operations (scale feature, not essential for validation)
- Role-based access beyond basic admin/user (start simple, expand based on need)
- Import/export (data portability is v2 concern)
- Sales pipeline/deals (scope creep - keep CRM simple)

### Architecture Approach

The architecture uses a clean separation between protocol and application layers. Backend and frontend each have parallel protocol implementation layers for consistency. Components are represented as a flat adjacency list (map of nodes with ID references) rather than nested trees, enabling O(1) lookup, easy patching, and streaming. Data binding uses JSON Pointer (RFC 6901) with stable keyed collections (not array indices) to avoid race conditions.

**Major components:**

1. **Protocol Layer (Backend)** — Encodes render/patch/event messages to JSON, validates incoming actions, manages WebSocket connections. Isolates OpenSDUI spec from application logic.

2. **Component Builder (Backend)** — Type-safe Rust API using derive macros to construct UI component trees. Enforces required props at compile time, generates adjacency list format.

3. **Action Handlers (Backend)** — Route incoming actions to business logic, return render or patch responses. Each handler is a separate function/module for maintainability.

4. **Business Logic (CRM Domain)** — Contact, company, deal operations, Listmonk integration. Knows nothing about protocol, returns domain types that handlers convert to messages.

5. **Data Layer (Backend)** — SeaORM entities, queries, migrations. Application-level audit logging via ActiveModelBehavior trait.

6. **Protocol Layer (Frontend)** — Decodes messages, dispatches to renderer, sends actions upstream. TypeScript interfaces match backend message types.

7. **Data Store (Frontend)** — Svelte 5 runes-based reactive state with JSON Pointer path resolution. Implements dirty field tracking to avoid clobbering user edits during server patches.

8. **Component Registry (Frontend)** — Maps component type strings to Svelte components. Dynamic dispatch enables extensibility without switch statements.

9. **Renderer (Frontend)** — Recursive component that walks adjacency list and renders tree to DOM. Handles unknown component types gracefully.

10. **Surface Manager (Frontend)** — Manages multiple render targets (main, modal, sidebar, toast). Backend specifies surface in render messages.

**Key patterns:**
- Adjacency list component trees (flat map, not nested)
- JSON Pointer data binding with dirty field protection
- Type-safe builders with Rust macros
- Surface-based rendering for multi-pane UIs
- Keyed collections with stable IDs (never array indices)

### Critical Pitfalls

Research identified 7 critical pitfalls and numerous technical debt patterns. The top three have the highest impact and require early attention in specific phases.

1. **SDUI Component Granularity Mismatch** — Teams design components at the wrong abstraction level (too granular = massive payloads; too coarse = no flexibility). Follow CONCEPT.md: 60fps interactions (drag-drop, charts) are single leaf components with internal state. Start with 10-15 core components. **Warning signs:** Payloads >50KB, component count >100 per screen, UI jank. **Address in:** Phase 1 (spec defines granularity principles), Phase 2 (validate with real components).

2. **State Synchronization Race Conditions** — User edits field while server patch arrives for same path, clobbering input. Or patches arrive out of order causing inconsistent state. **Prevention:** Implement dirty field tracking (mark actively edited fields, skip/queue patches to those paths). Use keyed collections not array indices. Include timestamps/versions on patches. Implement optimistic update rollback. **Address in:** Phase 2 (core state management), Phase 4 (load testing validation).

3. **Blocking Async Code in Rust Backend** — Synchronous operations inside async functions starve Tokio runtime. Compiles fine, fails catastrophically under load. **Prevention:** Use spawn_blocking for ANY CPU-intensive or blocking I/O. Prefer tokio::sync::Mutex for locks spanning .await. Use tokio::fs not std::fs. Audit all dependencies for blocking behavior. Load test early with p99 latency monitoring. **Address in:** Phase 3 (backend patterns), ongoing code review.

4. **SeaORM N+1 Queries** — Loading entities with relations generates one query per item (50 users + 50 role queries). **Prevention:** Use SeaORM's LoaderTrait for batching, find_with_related() for joins, enable SQL query logging in dev, test with 100+ records not 3.

5. **CRM Permission Model Under-Design** — Simple role checks (isAdmin) insufficient, need record ownership, hierarchies, field-level permissions. Retrofitting expensive. **Prevention:** Design permission model BEFORE features (role + record + field-based). Centralize authorization logic. Deny by default. **Address in:** Phase 1 (define requirements), Phase 3 (implement before features).

6. **Svelte 5 Runes Migration Confusion** — Mixing Svelte 4 and 5 patterns, let/const inconsistencies, stores vs runes interop issues. **Prevention:** Start fresh with runes, use const for $state (unless $bindable needs let), rename .ts to .svelte.ts only when runes used directly, use {@render children()} not slots, onclick not on:click.

7. **SDUI Debugging Black Hole** — Distributed system makes root cause hard (backend sending wrong tree? Data binding broken? Frontend bug? Network issue?). **Prevention:** Build debugging tools alongside framework, log complete messages with correlation IDs, frontend dev mode shows component tree/bindings/patches, message validation at both ends. **Address in:** Phase 2 (frontend debug overlay), Phase 3 (backend tracing).

## Implications for Roadmap

Based on research, the natural build order follows protocol → rendering pipeline → components → application logic → domain features. This sequence minimizes rework and enables incremental validation.

### Phase 1: Protocol Specification & Foundation
**Rationale:** Everything depends on the protocol. Define message formats, component structure, and data binding semantics first as the single source of truth. Get this right before any code.

**Delivers:**
- OpenAPI 3.1 spec for REST endpoints
- AsyncAPI spec for WebSocket messages (optional but recommended)
- JSON Schema definitions for message envelopes (render, patch, action, event)
- Component node structure schema
- Data binding specification (JSON Pointer paths)
- Component granularity guidelines (address Pitfall #1)
- Authorization model requirements (address Pitfall #5)

**Avoids:**
- Component granularity mismatch (spec defines 60fps rule)
- Permission under-design (model defined before implementation)

**Research flag:** Standard patterns. OpenAPI/AsyncAPI are well-documented. Skip phase-specific research.

### Phase 2: Frontend Rendering Pipeline
**Rationale:** Frontend must render before backend can send meaningful content. Build the reactive data store, component registry, and renderer to validate the protocol design works in practice.

**Delivers:**
- Data store with JSON Pointer resolution and dirty field tracking (addresses Pitfall #2)
- Component registry (type string → Svelte component mapping)
- Recursive adjacency list renderer
- Surface manager (main, modal, sidebar, toast)
- Debug overlay/inspector (addresses Pitfall #7)
- Base Flowbite components (TextInput, Button, Select, Checkbox)

**Uses:**
- Svelte 5.48 with runes exclusively (addresses Pitfall #6)
- Flowbite Svelte 1.31 for accessible components
- Tailwind CSS 4.0

**Implements:** Protocol Layer (Frontend), Data Store, Renderer, Surface Manager

**Avoids:**
- State sync race conditions (dirty field tracking from day one)
- Svelte 4/5 pattern mixing (enforce runes-only)
- Debugging black hole (debug tools built alongside)

**Research flag:** Some research likely needed on dirty field detection patterns and Svelte 5 runes advanced usage (relatively new, fewer examples).

### Phase 3: Backend Protocol & Infrastructure
**Rationale:** With frontend rendering proven, build backend to send valid messages. Establish patterns for async code, data loading, and authorization before domain features.

**Delivers:**
- Protocol Layer (message encoding, validation, WebSocket)
- Component Builder API with Rust macros (type-safe builders)
- Action Handlers (routing infrastructure)
- SeaORM entities and migrations (Contact, Company, User)
- Permission system (centralized, deny-by-default)
- Request tracing with correlation IDs (addresses Pitfall #7)

**Uses:**
- Rust 1.85+ with Axum 0.8
- SeaORM 1.1 with migration support
- utoipa 5.4 for OpenAPI generation
- tokio 1.49 with spawn_blocking patterns (addresses Pitfall #3)

**Implements:** Protocol Layer (Backend), Component Builders, Action Handlers, Data Layer

**Avoids:**
- Blocking async code (spawn_blocking from start, load testing)
- N+1 queries (LoaderTrait, query logging enabled)
- Permission under-design (system built before features)
- Giant monolithic handlers (route to dedicated modules)

**Research flag:** Standard patterns for Axum + SeaORM. Skip phase-specific research unless Listmonk API needs investigation.

### Phase 4: CRM Core Features (MVP)
**Rationale:** Infrastructure complete, now build minimum viable CRM features to prove SDUI pattern works end-to-end. Focus on basic CRUD to validate the approach before adding complexity.

**Delivers:**
- Contacts CRUD (create, read, update, delete)
- Companies CRUD with contact linking
- Notes on contacts/companies
- Data tables with pagination
- Search (name, email, company)
- Basic authentication (login, sessions)
- Record detail views (form-based edit)

**Validates:**
- Full request/response cycle works
- Data binding feels natural
- Form workflows are smooth
- Component granularity choices were correct
- Performance acceptable with realistic data (100+ records)

**Avoids:**
- Scope creep (no deals/pipeline yet)
- Premature optimization (WebSocket can wait)
- Feature bloat (custom fields, bulk ops deferred)

**Research flag:** No research needed. Standard CRUD patterns.

### Phase 5: Enhanced Features (Post-MVP)
**Rationale:** Core validated, now add differentiating features. Real-time updates, Listmonk integration, and interaction history prove SDUI's advantages over traditional approaches.

**Delivers:**
- Real-time WebSocket updates (server pushes patches)
- Optimistic updates (responsive feel)
- Multi-surface rendering (modals, toasts)
- Listmonk integration (subscriber sync, campaign management)
- Interaction history (activity timeline)
- Tags/labels (categorization)
- Basic filtering

**Validates:**
- WebSocket reliability (reconnection logic)
- State sync under concurrent edits
- Integration patterns

**Avoids:**
- Complexity before core is solid
- "Real-time everything" (use judiciously)

**Research flag:** Listmonk API integration will need research. WebSocket reliability patterns are well-documented but may need refresh.

### Phase 6: Polish & Production Readiness
**Rationale:** Feature complete, focus on robustness, performance, and operational concerns.

**Delivers:**
- Role-based access control (admin, manager, user)
- Audit trail (who changed what when)
- Import/export (CSV)
- Bulk operations
- Performance optimization (caching, query tuning)
- Production deployment configuration
- Monitoring and observability

**Research flag:** No research needed. Standard production concerns.

### Phase Ordering Rationale

- **Protocol first (Phase 1):** Prevents rework. Backend and frontend build to same spec.
- **Frontend before backend (Phase 2 before 3):** Validates protocol is renderable before backend sends it.
- **Infrastructure before features (Phase 3 before 4):** Patterns established early prevent technical debt.
- **MVP before enhancements (Phase 4 before 5):** Prove core concept before adding complexity.
- **Polish last (Phase 6):** Feature set validated before optimization investment.

This order follows component dependencies from architecture research and addresses pitfalls at the earliest phase where mitigation is practical.

### Research Flags

**Phases likely needing /gsd:research-phase:**
- **Phase 2:** Dirty field detection patterns and Svelte 5 runes edge cases (relatively new, evolving patterns)
- **Phase 5:** Listmonk API integration specifics (documentation review, API surface area, auth patterns)

**Phases with standard patterns (skip research):**
- **Phase 1:** OpenAPI/AsyncAPI are well-documented standards
- **Phase 3:** Axum + SeaORM patterns are mature, extensively documented
- **Phase 4:** CRUD operations are standard web development
- **Phase 6:** Production deployment is well-trodden territory

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | All versions verified via docs.rs and official sites as of 2026-01-23. Axum 0.8 released Jan 2025, Svelte 5.48 current stable, SeaORM 1.1 production-ready. |
| Features | MEDIUM | SDUI patterns validated against DivKit, A2UI, Airbnb Ghost documentation. CRM features cross-referenced with 3+ simple CRM products. MVP definition is clear but enhancements could shift based on usage. |
| Architecture | MEDIUM-HIGH | Adjacency list pattern well-documented in A2UI spec. JSON Pointer is RFC standard. Axum/SeaORM integration patterns verified in official docs. Some uncertainty on optimal dirty field implementation. |
| Pitfalls | MEDIUM | Async Rust pitfalls verified across multiple production postmortems. SDUI pitfalls corroborated by Nativeblocks, Duolingo, Apollo sources. Svelte 5 migration issues from community discussions (newer, less historical data). |

**Overall confidence:** MEDIUM-HIGH

The stack choices and architectural patterns have high confidence due to official documentation and multiple corroborating sources. Feature priorities and pitfall severity assessments have medium confidence because they depend on domain expertise and production experience that can't be fully verified from documentation alone. The roadmap implications are well-founded based on component dependencies and pitfall timing.

### Gaps to Address

**Dirty field tracking implementation details:** Research identified the need but not the optimal implementation pattern. During Phase 2, investigate whether to use:
- Per-field dirty flags with manual clearing
- Timestamp-based conflict resolution
- User intent detection (click away = commit)
Validate chosen approach with prototype testing.

**Listmonk API surface area:** Research confirmed Listmonk has REST API for subscribers, lists, and campaigns. During Phase 5 planning, review actual API endpoints, authentication mechanism (basic auth vs API keys), rate limits, and webhook support for bidirectional sync.

**WebSocket reconnection edge cases:** Research identified need for queuing and replay, but exact sequencing (gap detection, patch ordering, conflict resolution) needs investigation during Phase 5. Consider: Do we track sequence numbers? Version vectors? Last-event-id headers?

**Permission model granularity:** Research established need for role + record + field-based permissions, but exact implementation (middleware? decorators? policy engine?) should be prototyped during Phase 3 planning. Validate chosen approach handles hierarchy traversal efficiently.

**Component granularity validation:** Research provides the "60fps rule" guideline but actual component boundaries (when to split, when to combine) can only be validated by building real screens in Phase 2. Be prepared to refactor component library based on findings.

## Sources

### Primary (HIGH confidence)

**Official Documentation:**
- Rust crate versions verified via docs.rs (axum 0.8.8, tokio 1.49.0, sea-orm 1.1.19, utoipa 5.4.0, all verified 2026-01-23)
- [Tokio Blog - Announcing axum 0.8.0](https://tokio.rs/blog/2025-01-01-announcing-axum-0-8-0)
- [Svelte 5 - What's New January 2026](https://svelte.dev/blog/whats-new-in-svelte-january-2026)
- [Flowbite Svelte Documentation](https://flowbite-svelte.com/docs/pages/introduction)
- [Tailwind CSS 4.0 Release](https://tailwindcss.com/blog/tailwindcss-v4)
- [OpenAPI 3.1.0 Specification](https://swagger.io/specification/)
- [RFC 6901 - JSON Pointer](https://tools.ietf.org/html/rfc6901)
- [SeaORM Documentation](https://www.sea-ql.org/SeaORM/)

**A2UI Protocol:**
- [A2UI Specification v0.8](https://a2ui.org/specification/v0.8-a2ui/) - Adjacency list pattern source
- [A2UI Data Binding Concepts](https://a2ui.org/concepts/data-binding/)

### Secondary (MEDIUM confidence)

**SDUI Patterns:**
- [Airbnb Ghost Platform Deep Dive - Medium](https://medium.com/airbnb-engineering/a-deep-dive-into-airbnbs-server-driven-ui-system-842244c5f5)
- [DivKit GitHub](https://github.com/divkit/divkit) - Yandex's open source SDUI framework
- [Apollo GraphQL SDUI Schema Design](https://www.apollographql.com/docs/graphos/schema-design/guides/sdui/schema-design)
- [Nativeblocks - SDUI Best Practices and Common Pitfalls](https://nativeblocks.io/blog/best-practices-and-common-pitfalls/)
- [Duolingo - How server-driven UI keeps our shop fresh](https://blog.duolingo.com/server-driven-ui/)

**Rust/Backend:**
- [Leapcell - Rust Concurrency: Common Async Pitfalls](https://leapcell.medium.com/rust-concurrency-common-async-pitfalls-explained-8f80d90b9a43)
- [Qovery - Common Mistakes with Rust Async](https://www.qovery.com/blog/common-mistakes-with-rust-async)
- [Axum WebSocket Documentation](https://docs.rs/axum/latest/axum/extract/ws/index.html)

**Frontend:**
- [Svelte 5 Migration Guide](https://svelte.dev/docs/svelte/v5-migration-guide)
- [Loopwerk - First thoughts on Svelte 5 runes](https://www.loopwerk.io/articles/2025/svelte-5-runes/)
- [Loopwerk - Refactoring Svelte stores to $state runes](https://www.loopwerk.io/articles/2025/svelte-5-stores/)

**CRM Domain:**
- [Less Annoying CRM](https://www.lessannoyingcrm.com/) - Simple CRM reference
- [Capsule CRM Features](https://capsulecrm.com/features/contact-management-software/)
- [OnePageCRM Features Blog](https://www.onepagecrm.com/blog/crm-features/)
- [Listmonk Documentation](https://listmonk.app/docs/)

### Tertiary (LOW confidence - needs validation)

- [Server-Driven UI Design Patterns - Medium](https://devcookies.medium.com/server-driven-ui-design-patterns-a-professional-guide-with-examples-a536c8f9965f)
- [CRM Database Schema Guide - DragonflyDB](https://www.dragonflydb.io/databases/schema/crm)
- [WebSocket Reconnect Strategies - Apidog](https://apidog.com/blog/websocket-reconnect/)

---
*Research completed: 2026-01-23*
*Ready for roadmap: yes*
