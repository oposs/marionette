# Project Retrospective

*A living document updated after each milestone. Lessons feed forward into future planning.*

## Milestone: v1.0 — MVP

**Shipped:** 2026-04-08
**Phases:** 9 | **Plans:** 32 | **Tasks:** 62

### What Was Built
- OpenAPI 3.1 protocol specification with 6 message types, adjacency list components, JSON Pointer data binding
- Svelte 5 + Flowbite frontend library: reactive data store, dirty tracking, WebSocket transport, 20+ SDUI components
- Rust + Axum backend toolkit: derive macros, action routing, WebSocket sessions, SeaORM persistence
- End-to-end integration with protocol conformance validation against OpenAPI schemas
- Full CRM demo: auth/roles, company/contact CRUD, notes, tags, search/filtering, interaction timeline, Listmonk integration

### What Worked
- Fine-grained plan decomposition (32 plans across 9 phases) kept each unit small and focused
- Testing bundled with implementation phases rather than separate — tests validated immediately
- Parallel frontend/backend phases (3 and 4) with integration phase (5) as convergence point
- Protocol-first approach: specification in Phase 2 anchored all subsequent implementation
- Derive macros for component builders dramatically reduced boilerplate in CRM handlers
- Handler delegation pattern (re-rendering parent after child mutation) emerged naturally and scaled well

### What Was Inefficient
- Phase 1 infrastructure could have been lighter — some tooling was set up that wasn't needed until later
- ROADMAP.md progress table required manual updates and sometimes drifted from reality
- OnceLock pattern for external service clients (Phase 9) works but limits testability

### Patterns Established
- serde tagged enum for protocol message discriminator
- mpsc channel pattern for WebSocket reader/writer split
- Handler delegation: child handler re-renders parent by constructing new HandlerContext
- Type-erased AppState extension field for app-specific services
- NotSet pattern for SQLite auto-increment PKs and timestamp defaults
- find_also_related for SeaORM joins avoiding N+1 queries

### Key Lessons
1. Protocol-first design pays off: having the spec before implementation eliminated ambiguity and enabled parallel frontend/backend work
2. Svelte 5 runes ($state, $derived) simplify reactive data stores significantly compared to store-based approaches
3. SeaORM's ActiveModel with NotSet/Set semantics maps well to optional-update patterns in CRUD handlers
4. SQLite is excellent for demos but raw SQL in migrations (execute_unprepared) is needed for DEFAULT expressions
5. Adjacency list representation is genuinely simpler than nested trees for diff/patch operations

### Cost Observations
- Model mix: primarily opus for planning/architecture, sonnet for execution
- Sessions: ~32 execution sessions (roughly 1 per plan)
- Notable: average plan execution under 5 minutes — fine granularity kept context windows manageable

---

## Cross-Milestone Trends

### Process Evolution

| Milestone | Phases | Plans | Key Change |
|-----------|--------|-------|------------|
| v1.0 | 9 | 32 | Initial process — protocol-first, bundled testing |

### Top Lessons (Verified Across Milestones)

1. Protocol-first design enables parallel workstreams and eliminates integration ambiguity
2. Fine-grained plans (< 5 min each) keep context manageable and reduce rework
