# Phase 2: Protocol Specification - Context

**Gathered:** 2026-03-18
**Status:** Ready for planning

<domain>
## Phase Boundary

Define the complete OpenSDUI protocol specification: OpenAPI 3.1 YAML with JSON schemas for all message types, component structures, and data binding patterns. Write a protocol manual (spec/PROTOCOL.md) that supersedes CONCEPT.md as the authoritative reference. The spec defines the contract — implementations (frontend library, backend toolkit) are separate phases.

</domain>

<decisions>
## Implementation Decisions

### Spec file organization
- Split by concern: `spec/openapi.yaml` as entry point with `$ref` to `spec/schemas/` files
- Schema files: `component.yaml`, `message.yaml`, `data.yaml`, `common.yaml`
- Examples in `spec/examples/` as YAML files (human-readable, consistent with spec)
- Generate standalone JSON Schema files from OpenAPI for runtime validation (both formats available)
- Rust types in marionette-protocol crate are hand-written, validated against the spec (not generated)

### Message envelope
- Five message types: `render`, `patch`, `action`, `event`, `error` (error is a dedicated type, not errors-as-data)
- Optional `id` field on all messages for correlation — frontend sets on actions, backend echoes on responses
- Protocol versioning via handshake only — server sends `type: "hello"` with version on WebSocket connect
- Optimistic updates are a core spec requirement — the `optimistic` field on action messages is part of the spec

### Protocol manual
- Lives at `spec/PROTOCOL.md` — authoritative protocol reference
- Supersedes CONCEPT.md (which remains as vision/motivation document)
- Implementor-level audience: assumes web dev, JSON, REST, WebSocket knowledge
- Fresh examples written for precision — does not carry forward CONCEPT.md examples
- Reference with worked examples, not a tutorial

### Transport & endpoints
- WebSocket only for all protocol communication — single connection at `/ws`
- Initial HTTP GET serves the SvelteKit app (static files from Axum), then WebSocket takes over
- No REST endpoints for protocol messages (no GET /api/render/:surface)
- JSON messages over single WebSocket connection with same envelope structure
- Server sends `{ type: "hello", version }` on connect (handshake)
- Client sends actions: `{ type: "action", name, payload, id? }`
- Server sends: render, patch, event, error messages
- Spec defines reconnection strategy: exponential backoff, server re-sends current render state on new connection

### Claude's Discretion
- Exact JSON Schema draft version within OpenAPI 3.1 constraints
- Schema naming conventions and $ref organization
- Reconnection backoff parameters (initial delay, max delay, jitter)
- Whether to include a `type: "ping"` / `type: "pong"` keepalive in the spec
- Component prop schema expressiveness (how much to constrain open component types)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Protocol vision and design
- `CONCEPT.md` — Full protocol vision, three primitives, component catalog examples, worked flows. The manual supersedes this but should preserve the core design decisions.

### Project definition
- `TOOLING.md` — Tech stack decisions (utoipa for backend API docs, not for protocol spec)
- `.planning/REQUIREMENTS.md` — PROT-01 through PROT-14 and DOC-01, DOC-02 requirements

### Prior phase
- `.planning/phases/01-project-infrastructure/01-CONTEXT.md` — Infrastructure decisions, spec/ directory structure

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `spec/` directory exists (empty, has .gitkeep) — ready for schema files
- `backend/crates/marionette-protocol/` exists with stub `src/lib.rs` — will consume the spec as hand-written Rust types

### Established Patterns
- Cargo workspace with edition 2024 and serde/serde_json dependencies already in workspace
- marionette-protocol crate already depends on serde + serde_json

### Integration Points
- `spec/` directory is the output target for all schema files
- `spec/PROTOCOL.md` will be the new authoritative protocol reference
- marionette-protocol crate will implement types matching the schemas (Phase 4)
- Frontend library will consume the message types (Phase 3)

</code_context>

<specifics>
## Specific Ideas

- WebSocket-only protocol communication (after initial static file load) keeps things simple — one transport, one message format
- Dedicated error message type was chosen over errors-as-data for clearer generic error handling in the frontend
- Optimistic updates as core spec (not optional pattern) ensures all implementations support it
- Hand-written Rust types validated against spec gives idiomatic code while maintaining conformance

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 02-protocol-specification*
*Context gathered: 2026-03-18*
