# Phase 2: Protocol Specification - Research

**Researched:** 2026-03-18
**Domain:** OpenAPI 3.1 specification, JSON Schema, WebSocket protocol design, RFC 6901 JSON Pointer
**Confidence:** HIGH

## Summary

Phase 2 produces the complete OpenSDUI protocol specification: an OpenAPI 3.1 YAML document with JSON Schemas for all message types, a component adjacency list structure, data binding patterns, and a protocol manual (`spec/PROTOCOL.md`). The output is purely documentation and schema files -- no implementation code.

The key architectural challenge is that OpenAPI 3.1 does not natively support WebSocket message definitions. The solution is to use OpenAPI 3.1 as a **schema registry** -- defining all message types, component structures, and data patterns as reusable JSON Schema components -- while the WebSocket transport semantics are documented in the protocol manual. This is a well-established pattern: OpenAPI 3.1 aligns 100% with JSON Schema draft 2020-12, so the schemas serve double duty as both documentation and runtime-validatable contracts.

Six message types need full schema definitions: `hello` (handshake), `render`, `patch`, `action`, `event`, and `error`. These use a `discriminator` on the `type` field with `oneOf` to create a tagged union. The adjacency list component structure, keyed collections, and JSON Pointer data binding patterns each need dedicated schemas with worked examples.

**Primary recommendation:** Start with the JSON Schema definitions for all message and component types (`spec/schemas/`), then build the OpenAPI entry point (`spec/openapi.yaml`) referencing them, then write the protocol manual (`spec/PROTOCOL.md`) last -- since the manual references the schemas and benefits from them being finalized.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Split by concern: `spec/openapi.yaml` as entry point with `$ref` to `spec/schemas/` files
- Schema files: `component.yaml`, `message.yaml`, `data.yaml`, `common.yaml`
- Examples in `spec/examples/` as YAML files (human-readable, consistent with spec)
- Generate standalone JSON Schema files from OpenAPI for runtime validation (both formats available)
- Rust types in marionette-protocol crate are hand-written, validated against the spec (not generated)
- Five message types: `render`, `patch`, `action`, `event`, `error` (error is a dedicated type, not errors-as-data)
- Optional `id` field on all messages for correlation -- frontend sets on actions, backend echoes on responses
- Protocol versioning via handshake only -- server sends `type: "hello"` with version on WebSocket connect
- Optimistic updates are a core spec requirement -- the `optimistic` field on action messages is part of the spec
- Protocol manual lives at `spec/PROTOCOL.md` -- authoritative protocol reference
- Supersedes CONCEPT.md (which remains as vision/motivation document)
- Implementor-level audience: assumes web dev, JSON, REST, WebSocket knowledge
- Fresh examples written for precision -- does not carry forward CONCEPT.md examples
- Reference with worked examples, not a tutorial
- WebSocket only for all protocol communication -- single connection at `/ws`
- Initial HTTP GET serves the SvelteKit app (static files from Axum), then WebSocket takes over
- No REST endpoints for protocol messages
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

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| PROT-01 | Message envelope format (type, payload, correlation ID) | Tagged union pattern with discriminator on `type`; optional `id` for correlation |
| PROT-02 | Component structure (id, type, props, children, bind, action) | Component schema in `spec/schemas/component.yaml` with adjacency list node definition |
| PROT-03 | Adjacency list pattern (flat nodes with ID references, root pointer) | ComponentTree schema: `root` string + `nodes` map of string to Component |
| PROT-04 | Data binding via JSON Pointer (RFC 6901) | `bind` field as JSON Pointer string; format: "json-pointer" in JSON Schema |
| PROT-05 | Keyed collections pattern (stable keys, not array indices) | Data schema documenting object-as-map pattern with `orderPath` for display order |
| PROT-06 | Render message type (backend -> frontend) | RenderMessage schema: type="render", surface, root, nodes, data |
| PROT-07 | Patch message type (backend -> frontend) | PatchMessage schema: type="patch", array of operations with path + value |
| PROT-08 | Action message type (frontend -> backend) | ActionMessage schema: type="action", name, source?, payload?, id?, optimistic? |
| PROT-09 | Event message type (backend -> frontend) | EventMessage schema: type="event", name, surface?, hint? |
| PROT-10 | Error format (path + message, errors as data) | ErrorMessage schema: type="error", errors array with path + message; also per-field validation in data |
| PROT-11 | Surface concept (named render targets) | Surface as string enum or open string in render/event messages |
| PROT-12 | Optimistic update mechanism | `optimistic` field on ActionMessage containing patch operations to apply immediately |
| PROT-13 | REST endpoint definitions | **N/A per user decision** -- WebSocket only, no REST endpoints. Requirement superseded by PROT-14 |
| PROT-14 | WebSocket transport definition | Transport section in PROTOCOL.md; connection lifecycle, handshake, reconnection strategy |
| DOC-01 | OpenAPI 3.1 specification for all protocol messages | `spec/openapi.yaml` with $ref to schema files; validates with Spectral/Redocly |
| DOC-02 | Protocol manual explaining concepts, patterns, and rationale | `spec/PROTOCOL.md` -- implementor-level reference with worked examples |
</phase_requirements>

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| OpenAPI 3.1.0 | 3.1.0 | API specification format | Full JSON Schema 2020-12 alignment; discriminator support for tagged unions |
| JSON Schema | draft 2020-12 | Schema validation language | Default dialect for OpenAPI 3.1; supports `const`, `if/then/else`, type arrays |
| YAML 1.2 | 1.2 | Spec file format | Human-readable, standard for OpenAPI; consistent with project conventions |

### Tooling (dev dependencies)

| Tool | Version | Purpose | When to Use |
|------|---------|---------|-------------|
| @redocly/cli | 2.24.0 | Lint, validate, bundle OpenAPI specs | Validate multi-file spec; bundle into single file for distribution |
| @stoplight/spectral-cli | 6.15.0 | OpenAPI style linting | Enforce naming conventions, completeness rules |
| swagger-ui-dist | 5.32.1 | View spec in browser | Visual verification that spec renders correctly |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| OpenAPI 3.1 for WebSocket | AsyncAPI 3.0 | AsyncAPI is designed for event-driven/WebSocket APIs, but has less tooling maturity and less JSON Schema alignment. OpenAPI 3.1 used as schema registry is simpler for this project since schemas are the primary output, not transport bindings. |
| Redocly CLI | Swagger CLI | Redocly supports OpenAPI 3.1 natively, handles multi-file $ref, and has bundle command. Swagger CLI has more limited 3.1 support. |
| YAML examples | JSON examples | YAML is more readable for documentation. Both formats are equivalent. JSON can be generated from YAML. |

**Installation (in project root or spec/):**
```bash
npm install -D @redocly/cli @stoplight/spectral-cli
```

**Version verification:** All versions confirmed via `npm view` on 2026-03-18.

## Architecture Patterns

### Recommended Spec File Structure

```
spec/
  openapi.yaml              # Entry point -- info, servers, paths (minimal), components refs
  schemas/
    common.yaml             # Shared types: Surface, JsonPointer, MessageId
    message.yaml            # All message types: HelloMessage, RenderMessage, PatchMessage, etc.
    component.yaml          # Component, ComponentTree, ComponentAction, Props
    data.yaml               # DataObject, PatchOperation, KeyedCollection, ValidationError
  examples/
    render-user-table.yaml  # Full render message for user table scenario
    patch-update-name.yaml  # Patch message updating a field
    action-submit-form.yaml # Action message with optimistic update
    hello-handshake.yaml    # Server hello message
    error-validation.yaml   # Error message with field-level errors
  PROTOCOL.md               # Authoritative protocol manual
```

### Pattern 1: OpenAPI 3.1 as Schema Registry (not REST API)

**What:** Use OpenAPI 3.1 primarily for its `components/schemas` section to define all protocol types. The `paths` section is minimal (or empty) since there are no REST endpoints.

**When to use:** When the protocol is WebSocket-based but you need well-tooled schema definitions.

**Example:**
```yaml
# spec/openapi.yaml
openapi: "3.1.0"
info:
  title: OpenSDUI Protocol
  version: "1.0.0"
  description: |
    Server-Driven UI protocol specification.
    All communication occurs over a single WebSocket connection at /ws.
    See spec/PROTOCOL.md for transport details.
  license:
    name: MIT

# Paths is required by OpenAPI but can be empty for schema-only specs
paths: {}

# Webhooks can document server-to-client messages conceptually
webhooks:
  onMessage:
    post:
      summary: WebSocket message received
      description: |
        All messages (both directions) use the same envelope format.
        This webhook entry documents the message schema; actual transport is WebSocket.
      requestBody:
        content:
          application/json:
            schema:
              $ref: "#/components/schemas/ProtocolMessage"
      responses:
        "200":
          description: Message processed

components:
  schemas:
    ProtocolMessage:
      $ref: "schemas/message.yaml#/ProtocolMessage"
    Component:
      $ref: "schemas/component.yaml#/Component"
    # ... etc
```

### Pattern 2: Tagged Union with Discriminator

**What:** Use `oneOf` + `discriminator` on the `type` field to define the message envelope as a tagged union. Each message variant has `type` as a required string `const`.

**When to use:** Whenever multiple message shapes share a discriminating field.

**Example:**
```yaml
# spec/schemas/message.yaml
ProtocolMessage:
  oneOf:
    - $ref: "#/HelloMessage"
    - $ref: "#/RenderMessage"
    - $ref: "#/PatchMessage"
    - $ref: "#/ActionMessage"
    - $ref: "#/EventMessage"
    - $ref: "#/ErrorMessage"
  discriminator:
    propertyName: type
    mapping:
      hello: "#/HelloMessage"
      render: "#/RenderMessage"
      patch: "#/PatchMessage"
      action: "#/ActionMessage"
      event: "#/EventMessage"
      error: "#/ErrorMessage"

HelloMessage:
  type: object
  required: [type, version]
  properties:
    type:
      type: string
      const: hello
    version:
      type: string
      description: Protocol version (semver)
      example: "1.0.0"
  additionalProperties: false

RenderMessage:
  type: object
  required: [type, surface, root, nodes, data]
  properties:
    type:
      type: string
      const: render
    id:
      $ref: "#/MessageId"
    surface:
      $ref: "common.yaml#/Surface"
    root:
      type: string
      description: ID of the root node in the adjacency list
    nodes:
      type: object
      additionalProperties:
        $ref: "component.yaml#/Component"
      description: Flat map of node ID to component definition
    data:
      type: object
      description: Application state that components bind to
  additionalProperties: false
```

### Pattern 3: Component Adjacency List Schema

**What:** Components are a flat map of string IDs to component objects. Each component references children by ID, not by nesting.

**Example:**
```yaml
# spec/schemas/component.yaml
Component:
  type: object
  required: [type]
  properties:
    type:
      type: string
      description: Component type identifier (open set -- frontend defines vocabulary)
      example: "text-input"
    props:
      type: object
      description: Component-specific properties
      additionalProperties: true
    children:
      type: array
      items:
        type: string
      description: Ordered list of child node IDs
    bind:
      type: string
      format: json-pointer
      description: JSON Pointer (RFC 6901) to the data this component reads/writes
    action:
      $ref: "#/ComponentAction"
    visible:
      type: string
      format: json-pointer
      description: JSON Pointer to a boolean controlling visibility
  additionalProperties: false

ComponentAction:
  type: object
  required: [type]
  properties:
    type:
      type: string
      description: Action type identifier
    name:
      type: string
    target:
      type: string
    idPath:
      type: string
      format: json-pointer
  additionalProperties: true
```

### Pattern 4: JSON Pointer Data Binding

**What:** The `bind` field uses RFC 6901 JSON Pointer syntax to reference data paths. JSON Schema supports `format: "json-pointer"` natively.

**Key rules from RFC 6901:**
- Empty string `""` references the whole document
- `/foo` references the "foo" key
- `/foo/0` references array index 0 in "foo"
- `~0` escapes `~`, `~1` escapes `/` within tokens
- All pointers start with `/` (except empty string for root)

**Example in data binding context:**
```yaml
# Component binds to a specific path in the data object
nodes:
  email-field:
    type: "text-input"
    bind: "/user/email"        # Points to data.user.email
    props:
      label: "Email"

# The data object it references
data:
  user:
    email: "alice@example.com"
```

### Pattern 5: Keyed Collections (not arrays)

**What:** Collections use objects with stable string keys instead of arrays. Display order is maintained in a separate array.

**Example schema:**
```yaml
# spec/schemas/data.yaml
KeyedCollection:
  description: |
    A map of stable string keys to record objects.
    Patches target individual records by key path (e.g., /users/u-123/name).
    Display order is maintained in a separate array at the orderPath.
  type: object
  additionalProperties:
    type: object
  example:
    "u-123": { id: "u-123", name: "Alice" }
    "u-456": { id: "u-456", name: "Bob" }
```

### Anti-Patterns to Avoid

- **Nesting components in the schema:** Components are flat nodes in a map. Never define children inline -- always by ID reference. This is the core adjacency list guarantee.
- **Array-indexed collections:** Never use arrays for data collections. Patches to `/users/0/name` break when items are inserted or deleted. Always use keyed objects.
- **Defining specific component types in the protocol:** The protocol specifies the `Component` structure, not what types exist. Component types are an open string set -- the frontend library defines the vocabulary.
- **REST endpoint schemas:** Per user decision, there are no REST endpoints. Do not define `paths` entries for protocol messages.
- **Generating Rust types from the spec:** Per user decision, Rust types are hand-written and validated against the spec, not code-generated.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| OpenAPI validation | Custom YAML parser/checker | Redocly CLI `lint` + Spectral | Catches structural errors, $ref resolution failures, schema inconsistencies |
| JSON Schema from OpenAPI | Manual JSON Schema extraction | Redocly CLI `bundle` + schema extraction | Ensures JSON Schema files stay in sync with OpenAPI definitions |
| Spec bundling | Manual copy-paste into single file | Redocly CLI `bundle` | Resolves all $ref into a single distributable file |
| Discriminator logic | Manual if/else type dispatch | OpenAPI discriminator + mapping | Standard pattern all tools understand; generates correct code |
| JSON Pointer validation | Regex for path matching | `format: "json-pointer"` in schema | RFC 6901 compliant; validators handle edge cases (escaping, empty pointer) |

**Key insight:** The spec itself is the product of this phase. Tooling validates the spec -- do not substitute manual review for automated validation.

## Common Pitfalls

### Pitfall 1: OpenAPI 3.1 $ref Resolution Across Files
**What goes wrong:** $ref paths resolve relative to the file containing the reference, not the root openapi.yaml. Cross-file references break when paths are wrong.
**Why it happens:** Confusion between `$ref: "schemas/message.yaml#/RenderMessage"` (from openapi.yaml) vs `$ref: "#/RenderMessage"` (within message.yaml) vs `$ref: "common.yaml#/Surface"` (from message.yaml to common.yaml).
**How to avoid:** Use consistent relative paths. Within a schema file, use `#/TypeName` for local refs. Between schema files in the same directory, use `filename.yaml#/TypeName`. From openapi.yaml to schemas/, use `schemas/filename.yaml#/TypeName`.
**Warning signs:** Redocly/Spectral report "unresolved $ref" errors.

### Pitfall 2: Discriminator Requires Consistent Type Property
**What goes wrong:** The discriminator on `type` field requires each variant to declare `type` as a required string property with a `const` value. Missing this causes validation tools to ignore the discriminator.
**Why it happens:** OpenAPI discriminator is a "hint" -- if the schemas don't structurally support it, tools fall back to brute-force oneOf validation.
**How to avoid:** Every message variant must have: `required: [type]` and `properties.type.const: "message_type_string"`.
**Warning signs:** Swagger UI shows "one of" instead of the specific variant; validation accepts invalid message types.

### Pitfall 3: additionalProperties and Schema Evolution
**What goes wrong:** Setting `additionalProperties: false` on all schemas prevents future extension without breaking changes.
**Why it happens:** Desire for strict validation conflicts with forward compatibility.
**How to avoid:** Use `additionalProperties: false` on message envelopes (strict contract) but `additionalProperties: true` on component `props` (open by design -- components define their own props). Document the extension policy in PROTOCOL.md.
**Warning signs:** Frontend rejects valid messages from a newer backend because they contain unknown fields.

### Pitfall 4: PROT-10 vs PROT-13 Contradiction
**What goes wrong:** REQUIREMENTS.md defines PROT-10 as "errors as data" and defines a dedicated error message type. The CONTEXT.md resolves this: error is a dedicated message type for transport/protocol errors, while field validation errors live in the data model (bound to `/ui/errors` or similar).
**Why it happens:** Two valid error patterns (errors-as-messages vs errors-as-data) coexist.
**How to avoid:** Clearly distinguish in the spec: `ErrorMessage` (type="error") is for protocol-level errors (malformed action, unknown surface, server error). Field validation errors are data patches to error paths. Document both patterns in PROTOCOL.md.
**Warning signs:** Implementors confused about when to use which error mechanism.

### Pitfall 5: PROT-13 (REST endpoints) is Superseded
**What goes wrong:** REQUIREMENTS.md says PROT-13 is "REST endpoint definitions" but CONTEXT.md locked the decision to WebSocket-only with no REST endpoints.
**Why it happens:** Requirements were written before the transport decision was finalized.
**How to avoid:** Mark PROT-13 as fulfilled by documenting the WebSocket-only decision. The transport definition in PROT-14 covers what PROT-13 would have addressed.
**Warning signs:** Planner tries to create REST endpoint schemas.

### Pitfall 6: Hello Message Not in the Five-Type List
**What goes wrong:** CONTEXT.md says "five message types: render, patch, action, event, error" but also says server sends `type: "hello"` on connect. Six types total.
**Why it happens:** Hello is a handshake message, not a regular protocol message. The "five types" refers to the ongoing protocol flow.
**How to avoid:** Define `HelloMessage` as part of the `ProtocolMessage` union but document it as connection-lifecycle-only. It is sent exactly once per connection, before any other messages.
**Warning signs:** Schema shows only five types but hello is expected.

## Code Examples

### Complete Message Schema (message.yaml)

```yaml
# spec/schemas/message.yaml
# All protocol message types defined as a tagged union

MessageId:
  type: string
  description: Optional correlation ID set by client, echoed by server
  example: "msg-a1b2c3"

ProtocolMessage:
  description: Tagged union of all protocol message types
  oneOf:
    - $ref: "#/HelloMessage"
    - $ref: "#/RenderMessage"
    - $ref: "#/PatchMessage"
    - $ref: "#/ActionMessage"
    - $ref: "#/EventMessage"
    - $ref: "#/ErrorMessage"
  discriminator:
    propertyName: type
    mapping:
      hello: "#/HelloMessage"
      render: "#/RenderMessage"
      patch: "#/PatchMessage"
      action: "#/ActionMessage"
      event: "#/EventMessage"
      error: "#/ErrorMessage"

HelloMessage:
  type: object
  required: [type, version]
  properties:
    type:
      type: string
      const: hello
    version:
      type: string
      description: Protocol version (semver)
  additionalProperties: false

ActionMessage:
  type: object
  required: [type, name]
  properties:
    type:
      type: string
      const: action
    id:
      $ref: "#/MessageId"
    name:
      type: string
      description: Action identifier
    source:
      type: string
      description: Component ID that triggered the action
    payload:
      type: object
      description: Action-specific data
      additionalProperties: true
    optimistic:
      type: object
      properties:
        patch:
          type: array
          items:
            $ref: "data.yaml#/PatchOperation"
      required: [patch]
      description: Patches to apply immediately (rolled back on error)
  additionalProperties: false

ErrorMessage:
  type: object
  required: [type, errors]
  properties:
    type:
      type: string
      const: error
    id:
      $ref: "#/MessageId"
    errors:
      type: array
      items:
        type: object
        required: [message]
        properties:
          path:
            type: string
            format: json-pointer
            description: Data path the error relates to (optional for global errors)
          message:
            type: string
            description: Human-readable error message
  additionalProperties: false
```

### Patch Operation Schema (data.yaml)

```yaml
# spec/schemas/data.yaml

PatchOperation:
  type: object
  required: [path, value]
  properties:
    path:
      type: string
      format: json-pointer
      description: JSON Pointer to the data location to update
    value:
      description: New value to set at the path (any JSON value)
  additionalProperties: false
```

### Common Types (common.yaml)

```yaml
# spec/schemas/common.yaml

Surface:
  type: string
  description: Named render target in the frontend layout
  examples:
    - main
    - sidebar
    - modal
    - toast

JsonPointer:
  type: string
  format: json-pointer
  description: RFC 6901 JSON Pointer path
  pattern: "^(/[^/]*)*$"
  examples:
    - "/user/name"
    - "/users/u-123/email"
    - "/ui/loading"
```

### Redocly Configuration

```yaml
# spec/redocly.yaml (or .redocly.yaml in project root)
extends:
  - recommended

rules:
  no-unused-components: error
  spec-components-invalid-map-name: error

apis:
  main:
    root: spec/openapi.yaml
```

### Spectral Configuration

```yaml
# spec/.spectral.yaml
extends:
  - "spectral:oas"

rules:
  # OpenAPI 3.1 specific
  oas3-schema: error
  # Ensure all schemas have descriptions
  oas3-valid-schema-example: warn
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| OpenAPI 3.0 + custom JSON Schema | OpenAPI 3.1 with native JSON Schema 2020-12 | Feb 2021 (OAS 3.1 release) | No more `nullable` keyword -- use `type: ["string", "null"]`; full `const`, `if/then/else` support |
| Separate JSON Schema + OpenAPI | Unified in OpenAPI 3.1 | OAS 3.1 | Schema Object IS a JSON Schema; no translation needed |
| swagger-cli for validation | Redocly CLI / Spectral | 2023-2024 | Better OAS 3.1 support, multi-file handling, bundling |
| REST + WebSocket hybrid | WebSocket-only (project decision) | This project | Simpler protocol, single transport, all messages use same envelope |

**Deprecated/outdated:**
- `nullable: true` in schemas: Replaced by type arrays `type: ["string", "null"]` in OAS 3.1
- `swagger-cli validate`: Limited OAS 3.1 support; use Redocly CLI instead
- `$ref` with sibling properties (OAS 3.0): Now officially supported in OAS 3.1

## Open Questions

1. **Ping/Pong Keepalive**
   - What we know: WebSocket protocol has built-in ping/pong frames at the transport layer. Application-level keepalive is optional.
   - What's unclear: Whether to define `type: "ping"` / `type: "pong"` messages in the protocol spec, or rely on WebSocket transport-level ping/pong.
   - Recommendation: **Do NOT add application-level ping/pong.** WebSocket transport handles this. Adding protocol-level keepalive adds complexity with no benefit. Document in PROTOCOL.md that implementations should use WebSocket ping/pong frames for connection liveness. This keeps the message type count clean (6 types: hello, render, patch, action, event, error).

2. **Reconnection Backoff Parameters**
   - What we know: Spec should define reconnection strategy with exponential backoff.
   - Recommendation: Initial delay 1s, max delay 30s, jitter +/- 500ms. These are sensible defaults. Document as SHOULD (not MUST) to allow implementation flexibility.

3. **Component Props Schema Openness**
   - What we know: Component types are open (string, not enum). Props are component-specific.
   - Recommendation: `props` should be `type: object, additionalProperties: true` -- fully open. The protocol does not constrain what props a component accepts. The frontend library's component catalog defines that. This preserves the extensibility model described in CONCEPT.md.

4. **Where to Install Spec Tooling**
   - What we know: Project has `frontend/package.json` with npm. Spec tooling is also npm-based.
   - Recommendation: Add a minimal `spec/package.json` (or add to root-level package.json if one exists). Spec tooling is independent of frontend build. A `spec/package.json` keeps concerns separated and allows `make lint-spec` to work independently.

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Redocly CLI 2.24.0 + Spectral 6.15.0 |
| Config file | `spec/redocly.yaml` + `spec/.spectral.yaml` |
| Quick run command | `cd spec && npx @redocly/cli lint openapi.yaml` |
| Full suite command | `cd spec && npx @redocly/cli lint openapi.yaml && npx @stoplight/spectral-cli lint openapi.yaml` |

### Phase Requirements to Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| PROT-01 | Message envelope with type discriminator | schema validation | `npx @redocly/cli lint openapi.yaml` | Wave 0 |
| PROT-02 | Component structure schema | schema validation | `npx @redocly/cli lint openapi.yaml` | Wave 0 |
| PROT-03 | Adjacency list (root + nodes map) | schema validation | `npx @redocly/cli lint openapi.yaml` | Wave 0 |
| PROT-04 | JSON Pointer bind field | schema validation | `npx @redocly/cli lint openapi.yaml` | Wave 0 |
| PROT-05 | Keyed collections documented | manual-only | Review PROTOCOL.md keyed collection section | Manual |
| PROT-06 | Render message schema | schema validation + example | Validate example against schema | Wave 0 |
| PROT-07 | Patch message schema | schema validation + example | Validate example against schema | Wave 0 |
| PROT-08 | Action message schema | schema validation + example | Validate example against schema | Wave 0 |
| PROT-09 | Event message schema | schema validation + example | Validate example against schema | Wave 0 |
| PROT-10 | Error message schema | schema validation + example | Validate example against schema | Wave 0 |
| PROT-11 | Surface concept | schema validation | Surface type referenced in render/event schemas | Wave 0 |
| PROT-12 | Optimistic update field | schema validation | optimistic field in ActionMessage schema | Wave 0 |
| PROT-13 | REST endpoints (superseded) | N/A | Documented as WebSocket-only decision | N/A |
| PROT-14 | WebSocket transport | manual-only | Review PROTOCOL.md transport section | Manual |
| DOC-01 | OpenAPI 3.1 validates | automated | `npx @redocly/cli lint openapi.yaml` | Wave 0 |
| DOC-02 | Protocol manual | manual-only | Review spec/PROTOCOL.md for completeness | Manual |

### Sampling Rate
- **Per task commit:** `cd spec && npx @redocly/cli lint openapi.yaml`
- **Per wave merge:** Full lint with both Redocly and Spectral
- **Phase gate:** Both linters pass; all example files validate; PROTOCOL.md reviewed

### Wave 0 Gaps
- [ ] `spec/package.json` -- needs creation with @redocly/cli and @stoplight/spectral-cli devDependencies
- [ ] `spec/redocly.yaml` -- Redocly CLI configuration
- [ ] `spec/.spectral.yaml` -- Spectral linting rules
- [ ] `spec/openapi.yaml` -- entry point (does not exist yet)
- [ ] `spec/schemas/*.yaml` -- all schema files
- [ ] `spec/examples/*.yaml` -- all example files
- [ ] `spec/PROTOCOL.md` -- protocol manual
- [ ] Makefile target `lint-spec` or integration into existing `make lint`

## Sources

### Primary (HIGH confidence)
- [OpenAPI 3.1.0 Specification](https://spec.openapis.org/oas/v3.1.0) -- discriminator, oneOf, JSON Schema alignment
- [RFC 6901 - JSON Pointer](https://tools.ietf.org/html/rfc6901) -- pointer syntax, escaping rules
- npm registry -- verified @redocly/cli 2.24.0, @stoplight/spectral-cli 6.15.0, swagger-ui-dist 5.32.1 on 2026-03-18
- [Redocly multi-file definitions](https://redocly.com/learn/openapi/multi-file-definitions) -- $ref patterns, file organization
- [Redocly CLI commands](https://redocly.com/docs/cli/commands) -- lint, bundle, split commands

### Secondary (MEDIUM confidence)
- [Speakeasy $ref best practices](https://www.speakeasy.com/openapi/references) -- when to split files, naming conventions
- [Redocly discriminator guide](https://redocly.com/learn/openapi/discriminator) -- discriminator with oneOf pattern
- [OpenAPI 3.1 JSON Schema alignment](https://learn.openapis.org/upgrading/v3.0-to-v3.1.html) -- draft 2020-12 features available

### Tertiary (LOW confidence)
- None -- all findings verified against official sources

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- OpenAPI 3.1 is well-documented; tooling versions verified against npm registry
- Architecture: HIGH -- patterns (discriminator, $ref splitting, adjacency list) are well-established
- Pitfalls: HIGH -- $ref resolution and discriminator gotchas are well-documented in community
- Protocol design: HIGH -- decisions are locked by CONTEXT.md; design follows CONCEPT.md closely

**Research date:** 2026-03-18
**Valid until:** 2026-04-18 (stable specifications, 30-day window)
