# Pitfalls Research

**Domain:** SDUI Protocol + CRM Reference Implementation (OpenSDUI + Marionette)
**Researched:** 2026-01-23
**Confidence:** MEDIUM-HIGH (multiple sources corroborated, some domain-specific items from training data)

---

## Critical Pitfalls

### Pitfall 1: SDUI Component Granularity Mismatch

**What goes wrong:**
Teams design components at the wrong level of abstraction. Either too granular (sending individual SVG elements, every button separately) creating massive payloads and 60fps impossibility, OR too coarse (entire screens as single components) losing the flexibility that makes SDUI valuable.

**Why it happens:**
- Trying to make "everything server-driven" from day one
- Not distinguishing between structural components (forms, layouts) and interactive leaf components (charts, drag-drop zones)
- Copying mobile SDUI patterns (screen-level) when fine-grained web patterns are needed, or vice versa

**How to avoid:**
- Follow CONCEPT.md guidance: "60fps interactions (drag-drop, charts, canvas, animation) should be single leaf components with internal state"
- Start with 10-15 core components, expand based on actual need
- Rule of thumb: If a component needs sub-16ms updates, it's a leaf component with internal state
- Design components like "LEGO blocks" - composable but not too small

**Warning signs:**
- Payload sizes growing >50KB for simple screens
- UI jank or stuttering on interactions
- Backend needing to track UI micro-state (hover, animation frames)
- Component count per screen >100 nodes

**Phase to address:**
Phase 1 (Protocol Specification) - Define component granularity principles in the spec
Phase 2 (Frontend Library) - Validate with real component implementations

---

### Pitfall 2: State Synchronization Race Conditions

**What goes wrong:**
User edits a field while server patch arrives for same path. Field value gets clobbered, user loses input. Or: multiple patches arrive out of order, causing inconsistent UI state. Or: reconnection after disconnect loses intermediate state.

**Why it happens:**
- WebSocket messages can arrive out of order or during user interaction
- "Dirty field" detection not implemented or incomplete
- No versioning/timestamps on patches
- Optimistic updates without proper rollback

**How to avoid:**
- Implement dirty field tracking per CONCEPT.md: "When user is actively editing a field, frontend marks it 'dirty' and skips/queues incoming patches to that path"
- Use keyed collections (not array indices) for stable references
- Include timestamps or version counters in patches - "conflicts occur in nearly 15% of collaborative applications without sequencing"
- Implement optimistic update rollback: `optimistic: { patch: [...] }` with server revert on failure
- On WebSocket reconnect, fetch full state snapshot before resuming patches

**Warning signs:**
- User complaints about "lost typing" or "field jumping"
- Inconsistent UI after tab-switching or network blips
- Tests passing locally but failing under load
- Cursor position resetting during edits

**Phase to address:**
Phase 2 (Frontend Library) - Core state management implementation
Phase 4 (Working Application) - Real-world validation under load

---

### Pitfall 3: Blocking Async Code in Rust Backend

**What goes wrong:**
Synchronous operations (CPU-intensive work, blocking I/O, certain library calls) inside async functions starve the Tokio runtime. "Tokio's default scheduler uses one OS thread per CPU core, and blocking any of them" causes cascading latency. Code compiles, tests pass, then fails catastrophically under production load.

**Why it happens:**
- Rust's type system doesn't prevent blocking in async contexts
- Using `std::fs` instead of `tokio::fs`
- Third-party libraries with hidden blocking calls
- CPU-bound computation without `spawn_blocking`
- Lock contention with synchronous mutexes held across `.await`

**How to avoid:**
- Use `spawn_blocking` for ANY CPU-intensive or blocking I/O operation
- Prefer `tokio::sync::Mutex` for locks that span `.await` points, BUT consider if you can restructure to avoid the await inside the critical section
- Use `try_send` instead of `send` on channels when possible to avoid awaiting
- Audit all dependencies for blocking behavior
- Load test early and monitor per-request latency distributions, not just averages

**Warning signs:**
- p99 latency spikes while p50 stays normal
- Latency increases linearly with concurrent connections
- `tokio-console` showing long-running tasks blocking
- Works fine in dev, falls apart at 100+ concurrent users

**Phase to address:**
Phase 3 (Backend Implementation) - Establish patterns from the start
Ongoing - Code review checklist item for all async code

---

### Pitfall 4: SeaORM N+1 Queries and Relation Loading

**What goes wrong:**
Loading entities with relations generates one query per related item instead of batched queries. A page showing 50 users with their roles becomes 51 queries (1 for users + 50 for roles). Performance degrades linearly with data volume.

**Why it happens:**
- Naive iteration: `for user in users { user.find_related(Role).one().await }`
- Not understanding SeaORM's `LoaderTrait` for batch loading
- ORM convenience hiding query explosion
- Lack of query logging during development

**How to avoid:**
- Use SeaORM's Entity Loader which "intelligently uses join for 1-1 and data loader for 1-N relations"
- Enable SQL query logging in development
- Use `find_with_related()` or explicit joins for list views
- Set up query count assertions in integration tests
- Profile with realistic data volumes (100+ records) not just 3 test records

**Warning signs:**
- Page load time scales with record count
- Database CPU spikes on list views
- Query logs showing repetitive similar queries
- "Works fine in dev" syndrome with small datasets

**Phase to address:**
Phase 3 (Backend Implementation) - Establish data loading patterns early
Phase 4 (Working Application) - Validate with realistic data volumes

---

### Pitfall 5: CRM Permission Model Under-Design

**What goes wrong:**
Starting with simple role checks (isAdmin, isSalesRep) then discovering you need: per-record ownership, team hierarchies, field-level permissions, and regional restrictions. Retrofitting authorization is expensive. "Broken access control has climbed to #1 in the OWASP Top 10."

**Why it happens:**
- "We'll add permissions later" mentality
- Underestimating CRM complexity: territory rules, manager hierarchies, custom sharing
- Scattering authorization logic across controllers instead of centralizing
- Simple RBAC when data model needs ABAC (attribute-based) aspects

**How to avoid:**
- Design permission model BEFORE building features, not after
- Plan for: role-based (Admin/Manager/Rep), record-based (owner, team), field-based (hide salary from reps)
- Consider hierarchy traversal: "VP > Manager > Rep" patterns
- Centralize authorization logic - don't scatter `if user.is_admin` checks
- Implement "deny by default" - explicit grants required
- Test with realistic permission scenarios from day one

**Warning signs:**
- Permission checks duplicated across multiple handlers
- "Just add an admin bypass" becoming common
- No clear answer to "can user X see record Y's field Z?"
- Security decisions made in frontend code

**Phase to address:**
Phase 1 (Protocol) - Define authorization model requirements
Phase 3 (Backend) - Implement centralized permission system before features
Phase 4 (Application) - Validate with realistic multi-tenant scenarios

---

### Pitfall 6: Svelte 5 Runes Migration Confusion

**What goes wrong:**
Mixing Svelte 4 and Svelte 5 patterns in the same codebase. Using `let` for `$state` but ESLint wants `const`. Stores vs runes interop breaking. Binding to imported module-level state no longer working. Typescript props becoming verbose boilerplate.

**Why it happens:**
- Svelte 5 is relatively new, old examples/tutorials still dominate search results
- Migration tooling uses `let` but ESLint rules expect `const`
- Different mental model: stores were subscribe-based, runes are proxy-based
- "In Svelte 5, we can no longer bind directly to an imported module"

**How to avoid:**
- Start fresh with Svelte 5 runes, don't migrate from Svelte 4 patterns
- Use `const` for `$state` unless using `$bindable()` (where `let` is required)
- Rename `.ts` files to `.svelte.ts` only when runes are used directly in that file
- Use `{@render children()}` for slots, not old slot syntax
- Use `onclick` not `on:click` for event handlers
- When `$derived` feels restrictive, use a plain function: `() => someRuneState`

**Warning signs:**
- VS Code showing "Legacy mode" on files you thought were migrated
- ESLint flooding with reassignment warnings
- Stores not updating as expected when mixed with runes
- "Object is not reactive" errors

**Phase to address:**
Phase 2 (Frontend Library) - Establish runes-only patterns from project start
All phases - Code review for Svelte 4 pattern leakage

---

### Pitfall 7: SDUI Debugging Black Hole

**What goes wrong:**
Something renders wrong. Is it: backend sending wrong component tree? Data binding misconfigured? Frontend component bug? Network issue? Serialization problem? The distributed nature of SDUI makes root cause analysis exponentially harder.

**Why it happens:**
- No standardized tooling for SDUI debugging
- Messages flow through multiple transformation layers
- Traditional debuggers assume code locality
- "Debugging an SDUI layout might involve inspecting JSON and logs to understand what went wrong"

**How to avoid:**
- Build debugging tools alongside the framework, not as afterthought
- Log complete message payloads with correlation IDs
- Frontend dev mode: show component tree, data bindings, pending patches
- Backend dev mode: trace action -> handler -> response pipeline
- Message validation at both ends (schema violations caught immediately)
- Visual debugging: highlight which component came from which message

**Warning signs:**
- Engineers spending 4x longer debugging than coding
- "Works in isolation" but breaks when integrated
- No one can explain why a specific component appeared
- Logs exist but don't connect request to response

**Phase to address:**
Phase 2 (Frontend) - Build debugging overlay/inspector
Phase 3 (Backend) - Implement tracing/correlation from start
Phase 4 (Application) - Validate debugging workflow with real bugs

---

## Technical Debt Patterns

Shortcuts that seem reasonable but create long-term problems.

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Skip dirty field tracking | Ship faster | User data loss, support tickets | Never in production |
| Array indices for collections | Simpler initial code | Race conditions, wrong-item updates | Never (use keyed objects) |
| Scatter permission checks | Quick feature delivery | Security holes, inconsistent enforcement | Never |
| Hardcode component types | Faster prototyping | Can't extend without code changes | Phase 1 prototype only |
| Skip message validation | Fewer moving parts | Silent corruption, hard debugging | Dev/prototype only |
| Global error handler only | Less error handling code | No graceful degradation | MVP with TODO to fix |
| Sync mutexes in async | Simpler code | Performance cliff under load | Never in hot paths |

---

## Integration Gotchas

Common mistakes when connecting components of the system.

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| Flowbite + SSR | Using `document` in server context | Check for browser context, use dynamic imports |
| SeaORM + Axum | Not enabling connection pooling | Configure pool in production: min/max connections |
| WebSocket + Load Balancer | Connections not sticky | Use sticky sessions or pub/sub for state sync |
| Tailwind + Flowbite | Missing content paths | Include `node_modules/flowbite-svelte/**/*` in content array |
| Rust Macros + IDE | IDE not understanding expanded code | Use `cargo expand` for debugging, keep macros simple |
| TypeScript + Svelte 5 | Verbose prop types | Accept the boilerplate, it's the current trade-off |

---

## Performance Traps

Patterns that work at small scale but fail as usage grows.

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| Full state render on every action | Increasing response sizes | Send patches not full renders for updates | >10 concurrent users |
| Unbatched database queries | Linear slowdown with data | Use joins, data loaders, pagination | >100 records per query |
| No WebSocket message queuing | Lost messages on reconnect | Queue during disconnect, replay on reconnect | Any network instability |
| Component tree re-render on any patch | UI jank on frequent updates | Granular reactivity, memoization | >5 patches/second |
| JSON.parse on large payloads | Main thread blocking | Streaming JSON parser or web worker | Payloads >100KB |
| Single-server WebSocket | All connections on one process | Horizontal scaling with pub/sub | >1000 concurrent connections |

---

## Security Mistakes

Domain-specific security issues beyond general web security.

| Mistake | Risk | Prevention |
|---------|------|------------|
| Trusting component types from client | Malicious component injection | Server defines all component types, validate strictly |
| No payload size limits | DoS via massive JSON | Set max tree depth, max node count, max payload size |
| Permissions checked in frontend only | Data exposure | All authorization server-side, frontend is presentation |
| WebSocket without auth | Session hijacking | Authenticate on connect, validate session on each message |
| Action handlers trusting payload | Injection attacks | Validate ALL action payloads against schema |
| Exposing internal IDs in patches | Enumeration attacks | Use opaque tokens or validate access per ID |

---

## UX Pitfalls

Common user experience mistakes in SDUI and CRM domains.

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| Loading spinners everywhere | Feels slow even when fast | Skeleton screens, optimistic updates |
| Full page reload on errors | Loses user context | Component-level error boundaries |
| No offline indication | Confusion when disconnected | Clear offline state, queue actions for retry |
| Form resets on validation error | Lost user input | Keep input, highlight errors in-place |
| Modal for everything | Workflow disruption | In-line editing, slide-overs for context |
| CRM requiring exact input | High error rate | Fuzzy search, suggestions, autocomplete |

---

## "Looks Done But Isn't" Checklist

Things that appear complete but are missing critical pieces.

- [ ] **Data binding:** Often missing dirty field handling - verify patches don't clobber active edits
- [ ] **WebSocket:** Often missing reconnection logic - verify behavior after network drop
- [ ] **Permissions:** Often missing field-level checks - verify sensitive fields hidden appropriately
- [ ] **Error handling:** Often missing recovery path - verify user can continue after any error
- [ ] **Forms:** Often missing validation feedback - verify all error states visible to user
- [ ] **Tables:** Often missing empty state - verify graceful display with zero records
- [ ] **Actions:** Often missing loading state - verify feedback during async operations
- [ ] **Navigation:** Often missing browser back button - verify history works correctly
- [ ] **Accessibility:** Often missing keyboard navigation - verify tab order and screen readers

---

## Recovery Strategies

When pitfalls occur despite prevention, how to recover.

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| State sync race conditions | MEDIUM | Add version counters to patches, implement conflict resolution |
| N+1 query performance | LOW | Add data loader pattern, query logging, test with volume |
| Permission model too simple | HIGH | Design new model, migrate data, update all handlers |
| Component granularity wrong | HIGH | Redesign component boundaries, may require protocol changes |
| Blocking async code | MEDIUM | Profile to find, wrap in spawn_blocking, may need restructure |
| Svelte 4/5 pattern mixing | LOW | Automated migration tool + manual review pass |
| Missing debugging tools | MEDIUM | Build incrementally, add logging/tracing retroactively |

---

## Pitfall-to-Phase Mapping

How roadmap phases should address these pitfalls.

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| Component granularity | Phase 1 (Spec) | Spec includes granularity guidelines with examples |
| State sync races | Phase 2 (Frontend) | Tests for concurrent edit + patch scenarios |
| Blocking async | Phase 3 (Backend) | Load tests pass, p99 latency acceptable |
| N+1 queries | Phase 3 (Backend) | Query count assertions in tests, query logging enabled |
| Permission under-design | Phase 3 (Backend) | Permission model documented before first feature |
| Svelte 5 confusion | Phase 2 (Frontend) | Linting rules enforced, no Svelte 4 patterns |
| Debugging black hole | Phase 2-3 | Debug tools built alongside features |
| WebSocket reliability | Phase 2 (Frontend) | Reconnection tests, offline behavior documented |

---

## OpenSDUI-Specific Considerations

Based on CONCEPT.md, these pitfalls are particularly relevant:

### Adjacency List vs Nested Tree
**Risk:** Developers familiar with nested component trees may fight the flat adjacency list model.
**Prevention:** Strong documentation, examples showing why flat is better for patching/streaming.

### JSON Pointer Path Errors
**Risk:** Path escaping (`~0` for `~`, `~1` for `/`) forgotten, causing silent failures.
**Prevention:** Use library for pointer construction, never build paths with string concatenation.

### Surface Confusion
**Risk:** Multiple render surfaces (main, modal, toast) with unclear lifecycle.
**Prevention:** Document surface semantics clearly: what happens when modal closes? When does toast auto-dismiss?

### CallBackery Legacy Patterns
**Risk:** Prior experience with CallBackery may lead to carrying forward patterns that CONCEPT.md specifically improves upon.
**Prevention:** Treat CONCEPT.md as authoritative, explicitly document where new patterns differ from CallBackery.

---

## Sources

**SDUI Protocol & Design:**
- [Nativeblocks - SDUI Best Practices and Common Pitfalls](https://nativeblocks.io/blog/best-practices-and-common-pitfalls/)
- [Nativeblocks - Server-driven UI pros and cons](https://nativeblocks.io/blog/server-driven-ui-pros-cons/)
- [Medium - Server Driven UI: The Necessary Evil](https://medium.com/digia-studio/server-driven-ui-sdui-the-necessary-evil-for-scalable-mobile-apps-80c650a2c8de)
- [Apollo GraphQL - SDUI Basics](https://www.apollographql.com/docs/graphos/schema-design/guides/sdui/basics)
- [Duolingo - How server-driven UI keeps our shop fresh](https://blog.duolingo.com/server-driven-ui/)
- [GitHub - MobileNativeFoundation SDUI Discussion](https://github.com/MobileNativeFoundation/discussions/discussions/47)

**Rust Async & Backend:**
- [Leapcell - Rust Concurrency: Common Async Pitfalls](https://leapcell.medium.com/rust-concurrency-common-async-pitfalls-explained-8f80d90b9a43)
- [Qovery - Common Mistakes with Rust Async](https://www.qovery.com/blog/common-mistakes-with-rust-async)
- [Medium - Async Rust in Production: 7 Mistakes](https://ritik-chopra28.medium.com/async-rust-in-production-the-7-mistakes-that-cost-us-2-weeks-of-debugging-63699587a878)
- [Tokio - Announcing axum 0.8.0](https://tokio.rs/blog/2025-01-01-announcing-axum-0-8-0)
- [corrode - The State of Async Rust](https://corrode.dev/blog/async/)

**SeaORM:**
- [SeaQL - SeaORM Official](https://www.sea-ql.org/SeaORM/)
- [SeaQL - How we made SeaORM synchronous (2.0 announcement)](https://www.sea-ql.org/blog/2025-12-12-sea-orm-2.0/)
- [Shuttle - Guide to Rust ORMs 2025](https://www.shuttle.dev/blog/2024/01/16/best-orm-rust)

**Svelte 5:**
- [Svelte - Official Migration Guide](https://svelte.dev/docs/svelte/v5-migration-guide)
- [Loopwerk - First thoughts on Svelte 5 runes](https://www.loopwerk.io/articles/2025/svelte-5-runes/)
- [Loopwerk - Refactoring Svelte stores to $state runes](https://www.loopwerk.io/articles/2025/svelte-5-stores/)
- [GitHub - Svelte Discussion on Non-Obvious Runes Details](https://github.com/sveltejs/svelte/discussions/14835)

**CRM & Permissions:**
- [Hyegro - Top 10 CRM Implementation Mistakes 2026](https://www.hyegro.com/blog/crm-implementation-mistakes)
- [Panorama Consulting - 8 Common CRM Mistakes](https://www.panorama-consulting.com/mistakes-in-implementing-crm/)
- [ERP Software Blog - CRM Implementation Pitfalls](https://erpsoftwareblog.com/2025/04/solving-the-most-common-crm-implementation-pitfalls/)
- [Oso - Role-Based Access Control Guide](https://www.osohq.com/learn/rbac-role-based-access-control)
- [NocoBase - How to Design an RBAC System](https://www.nocobase.com/en/blog/how-to-design-rbac-role-based-access-control-system)

**Rust Macros:**
- [Ferrous Systems - Testing Procedural Macros](https://ferrous-systems.com/blog/testing-proc-macros/)
- [Towards Data Science - Nine Rules for Procedural Macros](https://towardsdatascience.com/nine-rules-for-creating-procedural-macros-in-rust-595aa476a7ff/)

**WebSocket & State:**
- [Ably - WebSocket Architecture Best Practices](https://ably.com/topic/websocket-architecture-best-practices)
- [cetra3 - Synchronizing state with WebSockets and JSON Patch](https://cetra3.github.io/blog/synchronising-with-websocket/)
- [Apidog - WebSocket Reconnect Strategies](https://apidog.com/blog/websocket-reconnect/)

**Flowbite:**
- [Flowbite Svelte - Official](https://flowbite-svelte.com/)
- [Flowbite Svelte - FAQ and Tips](https://flowbite-svelte.com/docs/pages/faq-and-tips)
- [GitHub - Flowbite Svelte Issues](https://github.com/themesberg/flowbite-svelte/issues)

---
*Pitfalls research for: OpenSDUI Protocol + Marionette CRM Implementation*
*Researched: 2026-01-23*
