# Architecture Research: OpenSDUI + Marionette

**Domain:** Server-Driven UI Protocol and Reference Implementation
**Researched:** 2026-01-23
**Confidence:** MEDIUM-HIGH

## System Overview

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                           OPENSDUI PROTOCOL                                   │
│         (Message Format, Component Structure, Data Binding Spec)              │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                               │
│  ┌─────────────────────────────┐      ┌─────────────────────────────┐        │
│  │     MARIONETTE BACKEND      │      │    MARIONETTE FRONTEND      │        │
│  │        (Rust + Axum)        │      │   (Svelte 5 + Flowbite)     │        │
│  │                             │      │                             │        │
│  │  ┌───────────────────────┐  │      │  ┌───────────────────────┐  │        │
│  │  │   Protocol Layer      │  │ HTTP │  │   Protocol Layer      │  │        │
│  │  │   (Message Encode/    │<─┼──────┼─>│   (Message Decode/    │  │        │
│  │  │    Decode, Validate)  │  │  WS  │  │    Dispatch)          │  │        │
│  │  └───────────────────────┘  │      │  └───────────────────────┘  │        │
│  │            │                │      │            │                │        │
│  │  ┌───────────────────────┐  │      │  ┌───────────────────────┐  │        │
│  │  │   Component Builder   │  │      │  │   Component Registry  │  │        │
│  │  │   (Macros, DSL,       │  │      │  │   (Type -> Renderer,  │  │        │
│  │  │    Type-Safe API)     │  │      │  │    Dynamic Dispatch)  │  │        │
│  │  └───────────────────────┘  │      │  └───────────────────────┘  │        │
│  │            │                │      │            │                │        │
│  │  ┌───────────────────────┐  │      │  ┌───────────────────────┐  │        │
│  │  │   Action Handlers     │  │      │  │   Data Store          │  │        │
│  │  │   (Route Actions to   │  │      │  │   (Reactive State,    │  │        │
│  │  │    Business Logic)    │  │      │  │    JSON Pointer Bind) │  │        │
│  │  └───────────────────────┘  │      │  └───────────────────────┘  │        │
│  │            │                │      │            │                │        │
│  │  ┌───────────────────────┐  │      │  ┌───────────────────────┐  │        │
│  │  │   Business Logic      │  │      │  │   Surface Manager     │  │        │
│  │  │   (CRM Domain,        │  │      │  │   (main, modal,       │  │        │
│  │  │    Listmonk Client)   │  │      │  │    sidebar, toast)    │  │        │
│  │  └───────────────────────┘  │      │  └───────────────────────┘  │        │
│  │            │                │      │                             │        │
│  │  ┌───────────────────────┐  │      └─────────────────────────────┘        │
│  │  │   Data Layer          │  │                                             │
│  │  │   (SeaORM, SQLite/    │  │                                             │
│  │  │    PostgreSQL)        │  │                                             │
│  │  └───────────────────────┘  │                                             │
│  │                             │                                             │
│  └─────────────────────────────┘                                             │
│                                                                               │
└──────────────────────────────────────────────────────────────────────────────┘
```

## Component Responsibilities

| Component | Responsibility | Communicates With |
|-----------|----------------|-------------------|
| **Protocol Layer (Backend)** | Encode messages to JSON, validate incoming actions, manage WebSocket connections | Action Handlers, Axum routes |
| **Component Builder** | Type-safe Rust API for constructing UI component trees (adjacency list format) | Protocol Layer, Business Logic |
| **Action Handlers** | Route incoming actions to appropriate business logic, return render/patch responses | Protocol Layer, Business Logic |
| **Business Logic** | CRM domain operations, Listmonk integration, validation | Action Handlers, Data Layer |
| **Data Layer** | SeaORM entities, queries, migrations, audit trail | Business Logic |
| **Protocol Layer (Frontend)** | Decode messages, dispatch to renderer, send actions upstream | Data Store, Surface Manager |
| **Component Registry** | Map component type strings to Svelte components, handle unknown types | Protocol Layer, Renderers |
| **Data Store** | Svelte 5 reactive state store, JSON Pointer path resolution, dirty field tracking | All bound components |
| **Surface Manager** | Manage render targets (main, modal, sidebar, toast), coordinate multi-surface updates | Component Registry |

## Recommended Project Structure

### Backend (Rust)

```
backend/
├── src/
│   ├── main.rs                    # Axum server setup, routes
│   ├── lib.rs                     # Library exports
│   │
│   ├── protocol/                  # OpenSDUI protocol implementation
│   │   ├── mod.rs
│   │   ├── message.rs             # Message types (render, patch, action, event)
│   │   ├── component.rs           # Component struct (id, type, props, children, bind)
│   │   ├── data.rs                # Data envelope, JSON Pointer utilities
│   │   └── validate.rs            # Message validation
│   │
│   ├── components/                # Component builder DSL
│   │   ├── mod.rs
│   │   ├── macros.rs              # Derive macros for component builders
│   │   ├── primitives.rs          # Base component types (text-input, button, etc.)
│   │   ├── layout.rs              # Container, row, column, form
│   │   └── builders.rs            # Fluent API for building components
│   │
│   ├── handlers/                  # Action handlers
│   │   ├── mod.rs
│   │   ├── navigate.rs            # Navigation actions
│   │   ├── submit.rs              # Form submission
│   │   ├── crud.rs                # Generic CRUD operations
│   │   └── websocket.rs           # WebSocket connection management
│   │
│   ├── domain/                    # CRM business logic
│   │   ├── mod.rs
│   │   ├── contacts.rs
│   │   ├── companies.rs
│   │   ├── deals.rs
│   │   └── listmonk.rs            # Listmonk API client
│   │
│   ├── entities/                  # SeaORM entities
│   │   ├── mod.rs
│   │   ├── contact.rs
│   │   ├── company.rs
│   │   ├── deal.rs
│   │   ├── activity.rs
│   │   └── audit_log.rs
│   │
│   └── screens/                   # Screen builders (assemble components for views)
│       ├── mod.rs
│       ├── dashboard.rs
│       ├── contacts.rs
│       ├── contact_detail.rs
│       └── forms/
│           ├── contact_form.rs
│           └── deal_form.rs
│
├── migration/                     # SeaORM migrations
│   └── src/
│       ├── lib.rs
│       ├── m20260101_000001_create_company.rs
│       ├── m20260101_000002_create_contact.rs
│       └── ...
│
└── Cargo.toml
```

### Frontend (Svelte 5)

```
frontend/
├── src/
│   ├── app.html
│   ├── app.css                    # Tailwind imports
│   │
│   ├── lib/
│   │   ├── protocol/              # OpenSDUI client protocol
│   │   │   ├── index.ts
│   │   │   ├── messages.ts        # Message type definitions
│   │   │   ├── client.ts          # HTTP + WebSocket client
│   │   │   └── decoder.ts         # Message parsing
│   │   │
│   │   ├── store/                 # Reactive data store
│   │   │   ├── index.ts
│   │   │   ├── data.svelte.ts     # Svelte 5 runes-based data store
│   │   │   ├── pointer.ts         # JSON Pointer resolution (RFC 6901)
│   │   │   └── dirty.ts           # Dirty field tracking
│   │   │
│   │   ├── registry/              # Component registry
│   │   │   ├── index.ts
│   │   │   ├── registry.ts        # Type string -> Component mapping
│   │   │   └── fallback.svelte    # Unknown component handler
│   │   │
│   │   ├── renderer/              # Tree renderer
│   │   │   ├── index.ts
│   │   │   ├── Renderer.svelte    # Recursive adjacency list renderer
│   │   │   └── Surface.svelte     # Surface mount point
│   │   │
│   │   └── components/            # SDUI component implementations
│   │       ├── index.ts           # Registry exports
│   │       ├── primitives/
│   │       │   ├── TextInput.svelte
│   │       │   ├── Button.svelte
│   │       │   ├── Select.svelte
│   │       │   └── Checkbox.svelte
│   │       ├── layout/
│   │       │   ├── Container.svelte
│   │       │   ├── Form.svelte
│   │       │   └── Card.svelte
│   │       ├── data/
│   │       │   ├── DataTable.svelte
│   │       │   └── List.svelte
│   │       ├── navigation/
│   │       │   ├── SideNav.svelte
│   │       │   └── NavItem.svelte
│   │       └── feedback/
│   │           ├── Modal.svelte
│   │           ├── Toast.svelte
│   │           └── Alert.svelte
│   │
│   └── routes/
│       ├── +layout.svelte         # App shell with surfaces
│       └── +page.svelte           # Main entry, protocol init
│
├── svelte.config.js
├── tailwind.config.js
└── package.json
```

### Protocol Specification

```
spec/
├── openapi.yaml                   # REST endpoints
├── asyncapi.yaml                  # WebSocket messages (optional)
└── schemas/
    ├── message.yaml               # Render, Patch, Action, Event envelopes
    ├── component.yaml             # Component node structure
    └── data.yaml                  # Data envelope, JSON Pointer
```

### Structure Rationale

- **protocol/:** Isolates OpenSDUI spec implementation from application logic. Both backend and frontend have parallel protocol layers for consistency.
- **components/ (backend):** Builder DSL separate from business logic. Macros in dedicated module for maintainability.
- **screens/ (backend):** High-level view assembly. Maps 1:1 with user-visible screens, makes navigation clear.
- **domain/:** CRM-specific logic isolated from protocol concerns. Easy to test independently.
- **registry/ (frontend):** Central component mapping enables dynamic rendering without switch statements.
- **store/ (frontend):** Centralized reactive state with JSON Pointer resolution. Single source of truth.

## Architectural Patterns

### Pattern 1: Adjacency List Component Tree

**What:** Represent UI as a flat map of nodes with ID references instead of nested objects.

**When to use:** Always for SDUI. This is core to the protocol.

**Trade-offs:**
- (+) O(1) node lookup by ID
- (+) Easy to patch individual nodes
- (+) Streaming-friendly (nodes can arrive in any order)
- (+) LLM-friendly (easier to generate than nested trees)
- (-) Requires renderer to build tree at render time
- (-) Slightly more complex mental model

**Example (Backend - Rust):**
```rust
use std::collections::HashMap;

#[derive(Serialize)]
struct Component {
    id: String,
    #[serde(rename = "type")]
    component_type: String,
    props: serde_json::Value,
    children: Option<Vec<String>>,  // IDs, not nested components
    bind: Option<String>,           // JSON Pointer path
    action: Option<Action>,
}

#[derive(Serialize)]
struct RenderMessage {
    #[serde(rename = "type")]
    msg_type: String,  // "render"
    surface: String,
    root: String,      // ID of root component
    nodes: HashMap<String, Component>,
    data: serde_json::Value,
}
```

**Example (Frontend - Svelte 5):**
```svelte
<!-- Renderer.svelte -->
<script lang="ts">
  import { getContext } from 'svelte';
  import { registry } from '$lib/registry';

  let { nodeId, nodes } = $props();

  const node = $derived(nodes[nodeId]);
  const Component = $derived(registry.get(node.type));
</script>

{#if Component}
  <Component {...node.props} bind={node.bind}>
    {#if node.children}
      {#each node.children as childId}
        <svelte:self nodeId={childId} {nodes} />
      {/each}
    {/if}
  </Component>
{:else}
  <Fallback type={node.type} />
{/if}
```

### Pattern 2: JSON Pointer Data Binding (RFC 6901)

**What:** Components reference data via path strings like `/user/name` or `/contacts/c-123/email`. Changes to data automatically update bound components.

**When to use:** For any data that components display or edit.

**Trade-offs:**
- (+) Declarative binding, no imperative wiring
- (+) Standard RFC, well-understood
- (+) Easy patching (just path + value)
- (+) Supports nested and keyed collections
- (-) Requires path resolution at runtime
- (-) Invalid paths need graceful handling

**Example (Data Store - TypeScript):**
```typescript
// pointer.ts
export function resolve(data: unknown, pointer: string): unknown {
  if (pointer === '' || pointer === '/') return data;

  const tokens = pointer.slice(1).split('/').map(token =>
    token.replace(/~1/g, '/').replace(/~0/g, '~')
  );

  let current = data;
  for (const token of tokens) {
    if (current == null) return undefined;
    current = (current as Record<string, unknown>)[token];
  }
  return current;
}

export function set(data: unknown, pointer: string, value: unknown): void {
  const tokens = pointer.slice(1).split('/').map(token =>
    token.replace(/~1/g, '/').replace(/~0/g, '~')
  );

  let current = data as Record<string, unknown>;
  for (let i = 0; i < tokens.length - 1; i++) {
    current = current[tokens[i]] as Record<string, unknown>;
  }
  current[tokens[tokens.length - 1]] = value;
}
```

### Pattern 3: Type-Safe Component Builders (Rust Macros)

**What:** Use derive macros to generate builder APIs that enforce required props at compile time.

**When to use:** Building component trees in Rust. Prevents typos and missing required fields.

**Trade-offs:**
- (+) Compile-time validation of component structure
- (+) IDE autocompletion for props
- (+) Impossible to forget required fields
- (-) Macro complexity
- (-) Longer compile times
- (-) Learning curve

**Example:**
```rust
use derive_builder::Builder;

#[derive(Builder, Serialize)]
#[builder(setter(into))]
pub struct TextInput {
    pub id: String,
    pub label: String,
    #[builder(default)]
    pub placeholder: Option<String>,
    #[builder(default)]
    pub required: bool,
    #[builder(default)]
    pub input_type: InputType,
    pub bind: String,  // Required: must bind to data path
}

// Usage
let name_field = TextInputBuilder::default()
    .id("name-field")
    .label("Name")
    .placeholder("Enter your name")
    .required(true)
    .bind("/user/name")
    .build()?;
```

### Pattern 4: Surface-Based Rendering

**What:** Define named render targets (surfaces) where UI can be mounted. Backend specifies surface in render message.

**When to use:** Multi-pane UIs with modals, sidebars, toasts, etc.

**Trade-offs:**
- (+) Clear separation of UI regions
- (+) Backend controls what renders where
- (+) Supports overlays without complex state
- (-) Fixed set of surfaces (add new ones requires frontend change)

**Example (Frontend Layout):**
```svelte
<!-- +layout.svelte -->
<script lang="ts">
  import { Surface } from '$lib/renderer';
</script>

<div class="app-layout">
  <aside class="sidebar">
    <Surface name="sidebar" />
  </aside>

  <main class="main-content">
    <Surface name="main" />
  </main>
</div>

<Surface name="modal" overlay />
<Surface name="toast" overlay position="bottom-right" />
```

### Pattern 5: Keyed Collections (Not Array Indices)

**What:** Use stable string keys for collection items instead of array indices. Separate display order into a parallel array if needed.

**When to use:** Any list/table data that can be modified (add, delete, reorder).

**Trade-offs:**
- (+) Stable references for patches
- (+) No index confusion when items change
- (+) Parallel updates from different sources work correctly
- (-) Slightly more complex data structure
- (-) Requires key management

**Example:**
```json
{
  "contacts": {
    "items": {
      "c-001": { "id": "c-001", "name": "Alice", "email": "alice@example.com" },
      "c-002": { "id": "c-002", "name": "Bob", "email": "bob@example.com" }
    },
    "order": ["c-001", "c-002"]
  }
}
```

Patch to update Alice's email:
```json
{
  "type": "patch",
  "data": [
    { "path": "/contacts/items/c-001/email", "value": "alice.new@example.com" }
  ]
}
```

## Data Flow

### Request/Response Flow (REST)

```
┌──────────────┐         ┌──────────────┐         ┌──────────────┐
│   Browser    │         │    Axum      │         │  Business    │
│   (Svelte)   │         │   Handler    │         │   Logic      │
└──────┬───────┘         └──────┬───────┘         └──────┬───────┘
       │                        │                        │
       │  POST /api/action      │                        │
       │  { type: "navigate",   │                        │
       │    target: "contacts"} │                        │
       │───────────────────────>│                        │
       │                        │                        │
       │                        │  handle_navigate()     │
       │                        │───────────────────────>│
       │                        │                        │
       │                        │                        │ fetch contacts
       │                        │                        │ build components
       │                        │<───────────────────────│
       │                        │   RenderMessage        │
       │                        │                        │
       │  { type: "render",     │                        │
       │    surface: "main",    │                        │
       │    root: "page",       │                        │
       │    nodes: {...},       │                        │
       │    data: {...} }       │                        │
       │<───────────────────────│                        │
       │                        │                        │
       │  Render to DOM         │                        │
       │                        │                        │
```

### Real-Time Flow (WebSocket)

```
┌──────────────┐         ┌──────────────┐         ┌──────────────┐
│   Browser    │         │    Axum      │         │  Business    │
│   (Svelte)   │         │  WebSocket   │         │   Logic      │
└──────┬───────┘         └──────┬───────┘         └──────┬───────┘
       │                        │                        │
       │  WS Connect            │                        │
       │───────────────────────>│                        │
       │<───────────────────────│ Connection established │
       │                        │                        │
       │  { type: "action",     │                        │
       │    name: "submit",     │                        │
       │    payload: {...} }    │                        │
       │───────────────────────>│                        │
       │                        │  handle_submit()       │
       │                        │───────────────────────>│
       │                        │                        │
       │                        │                        │ process
       │                        │<───────────────────────│
       │  { type: "patch",      │                        │
       │    data: [...] }       │                        │
       │<───────────────────────│                        │
       │                        │                        │
       │  Apply patches         │                        │
       │  (reactive update)     │                        │
       │                        │                        │
       ~                        ~                        ~
       │                        │                        │
       │                        │  Server-side event     │
       │                        │  (e.g., data changed   │
       │                        │   by another user)     │
       │                        │<───────────────────────│
       │  { type: "event",      │                        │
       │    name: "data-changed"│                        │
       │    hint: {...} }       │                        │
       │<───────────────────────│                        │
       │                        │                        │
       │  Refetch if needed     │                        │
       │                        │                        │
```

### Data Binding Flow (Frontend)

```
┌─────────────────────────────────────────────────────────────────┐
│                        DATA STORE                                │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  $state = {                                              │    │
│  │    user: { name: "Alice", email: "alice@example.com" },  │    │
│  │    form: { data: {...}, errors: [] }                     │    │
│  │  }                                                       │    │
│  └───────────────────────────┬──────────────────────────────┘    │
│                              │                                   │
│              ┌───────────────┴───────────────┐                   │
│              ▼                               ▼                   │
│  ┌───────────────────────┐       ┌───────────────────────┐      │
│  │   resolve("/user/name")│       │resolve("/form/errors")│      │
│  │   → "Alice"            │       │   → []                │      │
│  └───────────┬────────────┘       └───────────┬───────────┘      │
│              │                                │                  │
└──────────────┼────────────────────────────────┼──────────────────┘
               │                                │
               ▼                                ▼
       ┌───────────────┐                ┌───────────────┐
       │  TextInput    │                │  ErrorList    │
       │  bind="/user/ │                │  bind="/form/ │
       │       name"   │                │       errors" │
       │               │                │               │
       │  displays:    │                │  displays:    │
       │  "Alice"      │                │  (empty)      │
       └───────────────┘                └───────────────┘
               │
               │ User types "Bob"
               ▼
       ┌───────────────┐
       │  Action sent: │
       │  { type:      │
       │    "change",  │
       │    path:      │
       │    "/user/    │
       │     name",    │
       │    value:     │
       │    "Bob" }    │
       └───────────────┘
```

## Scaling Considerations

| Scale | Architecture Adjustments |
|-------|--------------------------|
| 0-1k users | Monolith is fine. Single SQLite database. REST-only (no WebSocket). |
| 1k-10k users | Add WebSocket for real-time. Consider PostgreSQL. Add connection pooling. |
| 10k-100k users | Horizontal scaling with load balancer. Redis for WebSocket pub/sub across instances. Database read replicas. |
| 100k+ users | Consider splitting read-heavy screens to dedicated services. Event sourcing for audit trail. CDN for static assets. |

### Scaling Priorities

1. **First bottleneck: Database queries** - Add indexes on frequently queried fields. Use pagination for lists. Optimize N+1 queries with SeaORM's eager loading.

2. **Second bottleneck: WebSocket connections** - Each server has a connection limit. Use Redis pub/sub to coordinate messages across server instances. Consider sticky sessions.

3. **Third bottleneck: Component rendering** - Large component trees can be slow to serialize. Use patches instead of full re-renders. Consider server-side caching of rendered screens.

## Anti-Patterns to Avoid

### Anti-Pattern 1: Nested Component Trees in Protocol

**What people do:** Send components as deeply nested JSON objects like traditional HTML/React trees.

**Why it's wrong:**
- Hard to patch individual nodes (need to find path through tree)
- Not streaming-friendly (must wait for complete tree)
- LLMs struggle to generate deeply nested structures correctly
- Merge conflicts when multiple sources update

**Do this instead:** Use adjacency list with string ID references. Flat map of nodes.

### Anti-Pattern 2: Array Indices for Collection Items

**What people do:** Bind table rows to `/users/0`, `/users/1`, etc.

**Why it's wrong:**
- Delete row 0, now row 1 is row 0 - indices shift
- Concurrent updates hit wrong items
- Patches race and corrupt data

**Do this instead:** Use stable keys: `/users/u-123`, `/users/u-456`. Separate order array if needed.

### Anti-Pattern 3: Business Logic in Frontend

**What people do:** Put validation, conditional rendering logic, data transformation in Svelte components.

**Why it's wrong:**
- Defeats the purpose of SDUI (backend controls UI)
- Logic must be duplicated across platforms
- Harder to change behavior without frontend deploy

**Do this instead:** Backend sends fully-resolved data. Conditional visibility via `visible` prop bound to boolean data path. Validation errors as data, not component logic.

### Anti-Pattern 4: Giant Monolithic Handlers

**What people do:** Single `handle_action` function with massive match statement.

**Why it's wrong:**
- Becomes unmaintainable quickly
- Hard to test individual actions
- Poor code organization

**Do this instead:** Route actions to dedicated handler modules. Each handler is a separate function/module. Use Axum's layered routing.

### Anti-Pattern 5: Exposing Raw Database Entities in Protocol

**What people do:** Serialize SeaORM entities directly into component data.

**Why it's wrong:**
- Couples database schema to protocol
- May expose sensitive fields
- Database changes break frontend

**Do this instead:** Map entities to DTOs/view models. Explicit serialization of only needed fields.

## Integration Points

### External Services

| Service | Integration Pattern | Notes |
|---------|---------------------|-------|
| Listmonk | REST API client | Async HTTP calls. Handle rate limits. Cache subscriber lists. |
| PostgreSQL/SQLite | SeaORM connection pool | Use `sea-orm` async connections. Configure pool size per environment. |
| Future: Auth provider | OAuth2/OIDC | Token validation middleware. User context in request extensions. |

### Internal Boundaries

| Boundary | Communication | Notes |
|----------|---------------|-------|
| Protocol <-> Handlers | Typed Rust structs | Handlers receive validated `ActionMessage`, return `RenderMessage` or `PatchMessage`. |
| Handlers <-> Domain | Service layer | Domain functions don't know about protocol. Return domain types, handlers convert to messages. |
| Domain <-> Data | SeaORM entities | Repository pattern optional. Domain can use entities directly for simple cases. |
| Frontend Protocol <-> Store | TypeScript interfaces | Messages decoded to typed objects. Store updates trigger reactivity. |
| Store <-> Components | Svelte 5 runes | `$derived` for reads, actions for writes. JSON Pointer resolution. |

## Build Order Implications

Based on component dependencies, the recommended build order is:

### Phase 1: Protocol Foundation
1. **OpenAPI/AsyncAPI Spec** - Define message formats first (source of truth)
2. **Protocol Layer (Backend)** - Message types, serialization
3. **Protocol Layer (Frontend)** - Message types, client

**Rationale:** Everything depends on the protocol. Get it right first.

### Phase 2: Core Rendering Pipeline
1. **Data Store (Frontend)** - JSON Pointer resolution, reactive state
2. **Component Registry (Frontend)** - Type-to-component mapping
3. **Renderer (Frontend)** - Adjacency list to DOM
4. **Surface Manager (Frontend)** - Multiple render targets

**Rationale:** Frontend needs to render before backend can send meaningful content.

### Phase 3: Component Library
1. **Base Components (Frontend)** - TextInput, Button, Select (Flowbite-based)
2. **Component Builders (Backend)** - Rust macros/builders for same components
3. **Layout Components** - Container, Form, Card
4. **Data Components** - DataTable, List

**Rationale:** Build components in parallel front and back. Test rendering as you go.

### Phase 4: Application Layer
1. **Axum Handlers** - Action routing infrastructure
2. **Screen Builders (Backend)** - High-level view composition
3. **Navigation** - SideNav, routing
4. **CRM Entities** - SeaORM models, migrations

**Rationale:** Application logic depends on all lower layers.

### Phase 5: Domain Features
1. **Contacts CRUD**
2. **Companies CRUD**
3. **Deals/Opportunities**
4. **Activities/Audit Trail**
5. **Listmonk Integration**

**Rationale:** Features can be built incrementally once infrastructure is solid.

## CRM Data Model

### Core Entities

```
┌─────────────────┐       ┌─────────────────┐
│    Company      │       │    Contact      │
├─────────────────┤       ├─────────────────┤
│ company_id (PK) │       │ contact_id (PK) │
│ company_name    │◄──────│ contact_company │
│ company_industry│  1:N  │ contact_name    │
│ company_size    │       │ contact_email   │
│ company_website │       │ contact_phone   │
│ created_at      │       │ contact_title   │
│ updated_at      │       │ created_at      │
│ created_by      │       │ updated_at      │
└─────────────────┘       │ created_by      │
                          └─────────────────┘
                                  │
                                  │ 1:N
                                  ▼
┌─────────────────┐       ┌─────────────────┐
│     Deal        │       │   Activity      │
├─────────────────┤       ├─────────────────┤
│ deal_id (PK)    │       │ activity_id(PK) │
│ deal_contact    │───────│ activity_type   │
│ deal_company    │ M:1   │ activity_subject│
│ deal_title      │       │ activity_body   │
│ deal_value      │       │ activity_contact│
│ deal_stage      │       │ activity_deal   │
│ deal_close_date │       │ activity_date   │
│ created_at      │       │ created_at      │
│ updated_at      │       │ created_by      │
│ created_by      │       └─────────────────┘
└─────────────────┘
        │
        │ M:N (via deal_tag junction)
        ▼
┌─────────────────┐       ┌─────────────────┐
│      Tag        │       │   Audit Log     │
├─────────────────┤       ├─────────────────┤
│ tag_id (PK)     │       │ audit_id (PK)   │
│ tag_name        │       │ audit_entity    │
│ tag_color       │       │ audit_entity_id │
└─────────────────┘       │ audit_action    │
                          │ audit_old_value │
                          │ audit_new_value │
                          │ audit_user      │
                          │ audit_timestamp │
                          └─────────────────┘
```

### Audit Trail Strategy

Use **application-level audit logging** via SeaORM's `ActiveModelBehavior` trait:

```rust
impl ActiveModelBehavior for ActiveModel {
    async fn after_save<C>(model: Model, db: &C, insert: bool) -> Result<Model, DbErr>
    where
        C: ConnectionTrait,
    {
        let action = if insert { "INSERT" } else { "UPDATE" };
        AuditLog::log(db, "contact", model.contact_id, action, &model).await?;
        Ok(model)
    }

    async fn before_delete<C>(self, db: &C) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        AuditLog::log(db, "contact", self.contact_id.clone().unwrap(), "DELETE", &self).await?;
        Ok(self)
    }
}
```

## Sources

### HIGH Confidence (Official Documentation)
- [Axum WebSocket Extract Documentation](https://docs.rs/axum/latest/axum/extract/ws/index.html)
- [RFC 6901 - JSON Pointer](https://tools.ietf.org/html/rfc6901)
- [A2UI Specification v0.8](https://a2ui.org/specification/v0.8-a2ui/) - Adjacency list pattern, data binding
- [A2UI Data Binding Concepts](https://a2ui.org/concepts/data-binding/)
- [SeaORM Documentation](https://www.sea-ql.org/SeaORM/)

### MEDIUM Confidence (Verified Community Sources)
- [Airbnb's Server-Driven UI System - InfoQ](https://www.infoq.com/news/2021/07/airbnb-server-driven-ui/) - Ghost Platform architecture
- [Apollo GraphQL SDUI Schema Design](https://www.apollographql.com/docs/graphos/schema-design/guides/sdui/schema-design)
- [Flowbite Svelte Documentation](https://flowbite-svelte.com/)
- [derive_builder crate](https://docs.rs/derive_builder) - Rust builder pattern

### LOW Confidence (WebSearch Only - Needs Validation)
- [Server-Driven UI Design Patterns Medium Article](https://devcookies.medium.com/server-driven-ui-design-patterns-a-professional-guide-with-examples-a536c8f9965f) - Template/component/schema patterns
- [CRM Database Schema Guide - DragonflyDB](https://www.dragonflydb.io/databases/schema/crm) - Entity relationships

---
*Architecture research for: OpenSDUI Protocol + Marionette Reference Implementation*
*Researched: 2026-01-23*
