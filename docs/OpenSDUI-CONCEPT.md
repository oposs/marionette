# OpenSDUI Protocol + Marionette Implementation

> **This is an exposé, not a finished product.** It describes the vision, architecture, and key design decisions for **OpenSDUI** — an open server-driven UI protocol — and **Marionette**, its reference implementation. The detailed specification, frontend library, and backend are yet to be built. This document is the starting point.

## The Goal

**Build applications where the backend drives the frontend through a standardized protocol.**

Write a new backend, get a working UI. No frontend changes needed. The protocol is the contract between any backend and any frontend that implements it.

## Why This Matters

Traditional approaches couple backend and frontend tightly:
- Backend returns data, frontend hardcodes how to display it
- New screens require frontend deployments
- Different backends need different frontends

With a proper SDUI protocol:
- Backend describes *what* to render, frontend knows *how*
- New applications = new backend, same frontend
- Protocol is the contract, implementations are interchangeable

## Why Open Source

Companies like Airbnb, Lyft, Netflix, and Zalando have built internal SDUI systems. They've proven the pattern works at scale. But their implementations are proprietary — internal tools solving internal problems.

**Nothing comparable exists as open source.** No public protocol spec. No reusable frontend library. No reference implementation you can build on.

**OpenSDUI** aims to fill that gap: an open protocol specification. **Marionette** provides the reference implementation — a frontend library and backend toolkit. The pattern shouldn't require a large engineering team to adopt.

### Origins

This isn't my first SDUI. I created [CallBackery](https://github.com/oetiker/callbackery) — a Perl/Mojolicious framework with a Qooxdoo frontend — over a decade ago. It pioneered the "backend configures frontend" pattern before SDUI had a name, and it's still running in production today.

CallBackery works. But it accumulated inconsistencies over years of organic growth, and the concepts were never formally documented. OpenSDUI + Marionette is a clean-room redesign: same core insight, modern stack, proper protocol specification.

## Three Primitives

The entire protocol rests on three concepts. Everything else is a pattern within these.

### 1. Components (What to Render)

A flat list of typed nodes with ID references — the **adjacency list** pattern from [A2UI](https://a2ui.org/):

```yaml
root: "form-1"
nodes:
  form-1:
    type: "form"
    children: ["name-field", "email-field", "save-btn"]
  name-field:
    type: "text-input"
    bind: "/user/name"
    props: { label: "Name" }
  email-field:
    type: "text-input"
    bind: "/user/email"
    props: { label: "Email", required: true }
  save-btn:
    type: "button"
    props: { label: "Save" }
    action: { type: "submit", target: "/user" }
```

**Why flat, not nested?**
- Easy to patch (update one node by ID)
- Streaming-friendly (send nodes as ready)
- LLM-friendly (generate incrementally)

**Component type is a string** (open set). Frontend declares what types it supports. Protocol doesn't enumerate them — that's what makes it extensible.

**Where to render?** The message includes a surface: `"surface": "main"` or `"surface": "modal"`. Surfaces are just named locations the frontend defines (main area, modal overlay, toast region, etc.).

### 2. Data (Application State)

JSON that components bind to via [RFC 6901 JSON Pointer](https://datatracker.ietf.org/doc/html/rfc6901) paths:

```yaml
data:
  user:
    name: "Alice"
    email: "alice@example.com"
  ui:
    loading: false
    errors: []
```

A component with `bind: "/user/email"` displays and edits that value. Change the data, UI updates. User edits the field, data updates.

**Collections use keys, not array indices.** Array indices are unstable — if a row is deleted or reordered, indices shift and patches hit the wrong item. Instead, collections are keyed objects:

```yaml
data:
  users:
    "u-123": { id: "u-123", name: "Alice", email: "alice@example.com" }
    "u-456": { id: "u-456", name: "Bob", email: "bob@example.com" }
  userOrder: ["u-123", "u-456"]  # Display order if needed
```

Binding: `bind: "/users"` with `keyPath: "id"`. Patches target by key:

```yaml
type: "patch"
surface: "main"
patch:
  - op: "set"
    path: "/users/u-123/name"
    value: "Alicia"
```

No race conditions. No index confusion. Keys are stable.

**Validation state lives in data too.** Errors aren't a special channel — they're data the UI binds to:

```yaml
data:
  ui:
    errors:
      - path: "/user/email"
        message: "Invalid format"
```

An error-display component binds to `/ui/errors` and shows them.

**Translated strings are just strings.** Backend sends them already localized based on the session's locale. No translation keys in the protocol — that's an implementation detail.

### 3. Messages (Communication)

Everything flows through messages. Backend and frontend exchange typed payloads:

**Backend → Frontend:**
```yaml
type: "render"
surface: "main"
components: { ... }
data: { ... }
```

```yaml
type: "patch"
surface: "main"
patch:
  - op: "set"
    path: "/user/name"
    value: "Bob"
  - op: "set-node"
    id: "contact-canton"
    component:
      type: "select"
      bind: "/contact/canton"
```

**patch** — incrementally update a surface's data and/or component tree. A patch message targets one surface and carries a batch of ops: `set` (data), `set-node` / `delete-node` / `set-children` / `insert-child` / `remove-child` (tree). Ops apply in declared order, all-or-nothing. Mix freely. The tree-mutation ops are how the "easy to patch — update one node by ID" claim above is implemented. See `spec/PROTOCOL.md §patch` for the full op reference and examples.

```yaml
type: "event"
name: "data-changed"
hint: { paths: ["/orders"] }
```

**Frontend → Backend:**
```yaml
type: "action"
name: "submit"
source: "save-btn"
payload: { ... }
```

```yaml
type: "action"
name: "navigate"
target: "/orders/ord-472"
```

That's it. Actions go up, renders and events come down. The message `type` and `name` determine what happens. Error responses are just messages with error payloads.

**Action response patterns:**
- **Synchronous** — Action → immediate render/patch response
- **Fire-and-forget** — Action acknowledged, no state change (e.g., analytics event)
- **Async operation** — Action → ack + patch `/ui/saving: true` → event when done → patch with result

## What the Protocol Defines

- **Message envelope** (type, payload, correlation ID)
- **Component structure** (id, type, props, children, bind, action)
- **Data binding** (JSON Pointer paths)
- **Standard message types** (render, patch, action, event)
- **Error format** (path + message, generic)

## Stateless by Design

The protocol is inherently stateless. Each message is self-contained:

- **Render** includes all nodes and data — frontend doesn't need prior context
- **Action** includes full payload — backend doesn't need to remember what it rendered
- **Patch** includes path and value — no dependency on previous patches

The backend doesn't track "what UI is currently displayed." It receives an action, processes it, responds. The frontend manages its own UI state.

```
Frontend state:              Backend:
┌─────────────────┐          ┌─────────────────┐
│ nodes: {...}    │  action  │                 │
│ data: {...}     │ ───────> │ process action  │
│                 │  render  │ (stateless)     │
│ (manages UI)    │ <─────── │                 │
└─────────────────┘          └─────────────────┘
```

**What about sessions?** Authentication and user context are orthogonal — handled via tokens/cookies as in any web app. The *protocol* doesn't require session state; *applications* layer it on top.

**What about WebSocket?** The connection is stateful (it stays open), but the *messages* are still self-contained. WebSocket is just transport — each message stands alone.

**What about multi-step wizards?** Progress can be:
- Sent with each action (frontend tracks step)
- Stored server-side keyed by user/session (application state, not protocol state)
- Encoded in the data model itself

The protocol is the transport. Applications add state as needed.

## Where the Client Is Smart

The protocol is stateless for *application* state — but every real UI has
*presentation* state that lives in the client by nature. A text input buffers
keystrokes locally before the committed value reaches the server. A toast
counts down and fades. A debounced search waits until the user stops typing.
A spinner animates between render frames. Round-tripping any of these to the
server every tick would be absurd.

The boundary: **the protocol owns what the application means; the client
owns how it's presented and when it's collected.**

| Protocol owns | Client owns |
|---|---|
| Committed values (`/form/data/email`) | In-progress typing, cursor, selection |
| Validation results (`/form/errors`) | When to fire the validate action (blur vs change) |
| Toast message + severity + duration | Countdown, fade, stacking, position |
| Search results | Debounce window before firing the search action |
| Authoritative row state | Optimistic overlay + rollback |
| Which field is under active edit | Skip/queue patches to the dirty field |

Presentation state is **allowed to be smart**, but it MUST respect three
invariants so the application can't diverge from the server:

1. **Server is authoritative for anything anyone else would care about.**
   If two users look at the same row, the value they each see comes from
   the protocol, not from one user's local edit buffer.
2. **Client timers and animations MUST NOT mutate data bound via `bind`.**
   They can fade, position, count down, debounce — they can't decide what
   the user's email address is.
3. **During active edit, client wins for the dirty path until commit.**
   After commit, server wins (dirty-field handling — see Production
   Considerations).

Rule of thumb: if it involves **time** or **pixels**, the client handles
it. If it involves **meaning**, the protocol does.

### Toasts as the worked example

A toast is not a persistent node — it's an event carrying presentation
hints the client knows how to render:

```yaml
type: "event"
name: "toast"
hint:
  message: "User deleted"
  severity: "success"
  duration: 6000
  action:
    label: "Undo"
    action: { name: "undo-user-delete", payload: { userId: "usr-3" } }
```

The client library owns the queue, render, countdown, fade, stacking, and
the "Undo" button's onClick-dispatch. The protocol owns the message
content, the severity, the duration, and the meaning of the Undo action.
The `action.action` nested shape is just a normal SDUI action dispatched
when the user clicks — same as any Button's `action`.

For richer toasts, the hint can carry an embedded SDUI tree instead of
plain text — same pattern as modals:

```yaml
type: "event"
name: "toast"
hint:
  duration: 8000
  component: { type: "container", children: [...] }
```

The client renders the tree inside the toast chrome; the protocol owns
the content, the client owns the overlay mechanics.

The reference frontend ships `svelte-sonner` for the toast chrome. Other
frontend libraries targeting other platforms may render the same event
differently — a native snackbar on mobile, a bottom banner on TV — which
is exactly why the event is named for the *concern* (`toast`) and not
for the library.

## What the Protocol Does NOT Define

- **Component schemas** (what props a "text-input" accepts)
- **Specific component types** (that's the frontend's vocabulary)
- **Validation codes** (REQUIRED, TOO_SHORT — implementation details)
- **Business logic** (what actions do — backend's job)

## Implementation Notes

- **Edit sync timing** — Component events specify when actions fire (blur, change). Frontend can debounce.
- **Loading states** — Just data. Bind a spinner to `/users/loading`.
- **Client-side validation** — Component props: `{ required: true, pattern: "..." }`. Server validates authoritatively.
- **Conditional visibility** — Bind to a boolean data path: `visible: "/form/showAdvanced"`. Backend controls visibility by patching the data. Keeps logic server-side where it belongs.
- **Pagination** — Action fetches page, backend patches the keyed collection. Frontend manages display order.
- **File uploads** — Action requests upload URL, browser handles binary separately.
- **Unknown components** — Frontend library decides: render fallback or error. (But you control the library, so this shouldn't happen.)
- **Error recovery** — Stateless helps. Retry action, reconnect, re-fetch state.
- **Optimistic updates** — Action can include `optimistic: { patch: [...] }`. Frontend applies immediately, server reverts on failure.
- **Focus preservation** — When patches arrive while user is typing, frontend must not steal focus, not reset cursor position, and intelligently skip or queue patches to fields being actively edited.
- **Component granularity** — 60fps interactions (drag-drop, charts, canvas, animation) should be single leaf components with internal state, not orchestrated node-by-node. Send `type: "chart", props: { data: [...] }`, not individual SVG elements.

## Production Considerations

**Dirty field handling** — When user is actively editing a field, frontend marks it "dirty" and skips/queues incoming patches to that path. Prevents server updates from clobbering user input mid-keystroke. Dirty flag clears on blur or submit.

**Security** — Frontend must sanitize string props before rendering (no raw HTML injection). Define max tree depth and node count to prevent DoS via massive payloads. Backend validates all action payloads.

**URL routing** — Navigation actions should include route information. Frontend reflects current route in URL. Page refresh sends current URL to backend, which responds with appropriate render. Browser back/forward trigger navigation actions.

**Accessibility** — Frontend library infers ARIA attributes from semantic component types (`button` gets `role="button"`, etc.). Components can include explicit `aria` props when needed. Focus management is frontend responsibility.

**Protocol versioning** — Include version in initial message exchange. Backend and frontend are deployed together (no version mismatch in normal operation), but version field enables graceful handling of cached/stale frontends.

**Error boundaries** — A single component failure must not crash the entire surface. Frontend renders fallback for errored components and reports the failure.

## Two Levels: Protocol and Frontend Library

**The protocol** is universal — it defines *how* backends and frontends communicate (Components, Data, Messages). Anyone can build a frontend library following this methodology.

**A frontend library** implements the protocol with a specific component vocabulary. Think of it like a UI toolkit: base components (text-input, data-table, modal) that your application uses. Different libraries exist for different platforms — web with shadcn-svelte, mobile with native widgets, TV with remote-friendly controls.

```
┌─────────────────────────────────────────────────────────┐
│                    THE PROTOCOL                         │
│         (Components, Data, Messages patterns)           │
├─────────────────┬─────────────────┬─────────────────────┤
│  Web Library    │ Mobile Library  │   TV Library        │
│ (shadcn-svelte) │ (Native)        │   (Remote-friendly) │
│                 │                 │                     │
│  text-input     │  text-field     │   focusable-input   │
│  data-table     │  list-view      │   scrollable-list   │
│  modal          │  bottom-sheet   │   overlay-panel     │
└─────────────────┴─────────────────┴─────────────────────┘
```

## The Frontend Library Model

**There is no capability negotiation.** The frontend library is not a separate service that your backend discovers at runtime. It's a dependency your application includes — like any other library.

```
┌─────────────────────────────────────────────────────────┐
│                  YOUR APPLICATION                        │
│                                                          │
│   ┌─────────────────┐    ┌────────────────────────────┐ │
│   │  Your Backend   │    │  Frontend Library          │ │
│   │  (Rust/Axum)    │    │  ┌──────────────────────┐  │ │
│   │                 │    │  │ Base Components      │  │ │
│   │  - serves the   │───>│  │ text-input, table,   │  │ │
│   │    frontend     │    │  │ modal, nav...        │  │ │
│   │  - sends        │    │  ├──────────────────────┤  │ │
│   │    components   │    │  │ Your Extensions      │  │ │
│   │  - handles      │    │  │ project-specific     │  │ │
│   │    actions      │    │  │ components           │  │ │
│   └─────────────────┘    │  └──────────────────────┘  │ │
│                          └────────────────────────────┘ │
└─────────────────────────────────────────────────────────┘
                              │
                              ▼
                         [ Browser ]
```

**The backend serves the frontend library to the browser.** They're bundled and deployed together. When you update the library, you redeploy. No version mismatch possible.

**Think OO:** The base library provides core components. Your project can:
- **Use** the base components as-is
- **Extend** components with project-specific behavior
- **Add** entirely new component types for your domain

```yaml
# Base library provides:
text-input, select, checkbox, button, data-table, modal, ...

# Your project extends with:
customer-picker    # extends select with customer search
order-timeline     # domain-specific visualization
approval-workflow  # custom multi-step component
```

Your backend knows exactly what components exist because you control both sides. The component catalog is documentation for your own team, not a runtime contract with a stranger.

**This is why there's no handshake.** Capability negotiation solves a problem we don't have. The frontend isn't a separate product with its own release cycle — it's part of your application.

## Worked Example: A Business App Frontend

A frontend with four component families: **navigation**, **forms**, **tables**, and **popups**.

### The Layout (Mount Points)

The frontend defines where things can render:

```
┌──────────────────────────────────────────────────┐
│  header                                          │
├────────────┬─────────────────────────────────────┤
│            │                                     │
│  sidebar   │              main                   │
│            │                                     │
│            │                                     │
├────────────┴─────────────────────────────────────┤
│  toast (overlay)           modal (overlay)       │
└──────────────────────────────────────────────────┘
```

Backend messages include `surface: "main"` or `surface: "modal"` — the frontend knows where that is.

### The Component Catalog

```yaml
# Navigation
side-nav:
  children: true

nav-item:
  props:
    label: { type: "string", required: true }
    icon: { type: "string" }
  bind: "boolean"  # active state
  events:
    click: true

nav-group:
  props:
    label: { type: "string", required: true }
    collapsed: { type: "boolean" }
  children: true   # contains nav-items

# Forms
form:
  props:
    title: { type: "string" }
  children: true

text-input:
  props:
    label: { type: "string", required: true }
    placeholder: { type: "string" }
    required: { type: "boolean" }
    inputType: { enum: ["text", "email", "password", "number"] }
  bind: "string"
  events:
    blur: true      # Validate on blur
    change: true    # Search-as-you-type

select:
  props:
    label: { type: "string", required: true }
    options: { type: "array" }  # [{value, label}]
  bind: "string"
  events:
    change: true

button:
  props:
    label: { type: "string", required: true }
    variant: { enum: ["primary", "secondary", "danger"] }
  events:
    click: true

# Tables
data-table:
  props:
    columns: { type: "array" }  # [{key, label, sortable}]
    selectable: { type: "boolean" }
    pageSize: { type: "integer" }
    keyField: { type: "string" }  # Which field is the stable ID
  bind: "object"      # Keyed object of rows
  orderPath: "string" # Path to display order array
  events:
    rowClick: true
    selectionChange: true

# Popups
modal:
  props:
    title: { type: "string" }
    size: { enum: ["sm", "md", "lg"] }
  children: true

confirm-dialog:
  props:
    title: { type: "string", required: true }
    message: { type: "string", required: true }
    confirmLabel: { type: "string" }
    cancelLabel: { type: "string" }
  events:
    confirm: true
    cancel: true
```

### Example Flow: User Management

**1. App loads — backend renders navigation and welcome screen:**

```yaml
# Message 1: Render sidebar
type: "render"
surface: "sidebar"
root: "nav"
nodes:
  nav:
    type: "side-nav"
    children: ["nav-users", "nav-settings"]
  nav-users:
    type: "nav-item"
    props: { label: "Users", icon: "users" }
    bind: "/nav/active/users"
    action: { type: "navigate", target: "users" }
  nav-settings:
    type: "nav-item"
    props: { label: "Settings", icon: "cog" }
    bind: "/nav/active/settings"
    action: { type: "navigate", target: "settings" }
data:
  nav: { active: { users: true, settings: false } }
```

**2. User clicks "Users" — frontend sends action:**

```yaml
type: "action"
name: "navigate"
target: "users"
```

**3. Backend renders user table:**

```yaml
type: "render"
surface: "main"
root: "page"
nodes:
  page:
    type: "container"
    children: ["header", "table"]
  header:
    type: "container"
    props: { class: "flex justify-between" }
    children: ["title", "add-btn"]
  title:
    type: "heading"
    props: { text: "Users", level: 1 }
  add-btn:
    type: "button"
    props: { label: "Add User", variant: "primary" }
    action: { type: "open-form", name: "user" }
  table:
    type: "data-table"
    props:
      columns:
        - { key: "name", label: "Name", sortable: true }
        - { key: "email", label: "Email" }
        - { key: "role", label: "Role" }
      selectable: true
      keyField: "id"
    bind: "/users/items"
    orderPath: "/users/order"
    action: { type: "open-detail", idPath: "/id" }
data:
  users:
    items:
      "usr-1": { id: "usr-1", name: "Alice", email: "alice@example.com", role: "Admin" }
      "usr-2": { id: "usr-2", name: "Bob", email: "bob@example.com", role: "User" }
    order: ["usr-1", "usr-2"]
```

**4. User clicks "Add User" — frontend sends action:**

```yaml
type: "action"
name: "open-form"
form: "user"
```

**5. Backend renders form in modal:**

```yaml
type: "render"
surface: "modal"
root: "modal"
nodes:
  modal:
    type: "modal"
    props: { title: "Add User", size: "md" }
    children: ["form"]
  form:
    type: "form"
    children: ["name", "email", "role", "actions"]
  name:
    type: "text-input"
    props: { label: "Name", required: true }
    bind: "/form/data/name"
  email:
    type: "text-input"
    props: { label: "Email", inputType: "email", required: true }
    bind: "/form/data/email"
  role:
    type: "select"
    props:
      label: "Role"
      options: [{ value: "user", label: "User" }, { value: "admin", label: "Admin" }]
    bind: "/form/data/role"
  actions:
    type: "container"
    children: ["cancel-btn", "save-btn"]
  cancel-btn:
    type: "button"
    props: { label: "Cancel", variant: "secondary" }
    action: { type: "close-modal" }
  save-btn:
    type: "button"
    props: { label: "Save", variant: "primary" }
    action: { type: "submit", name: "save-user" }
data:
  form:
    data: { name: "", email: "", role: "user" }
    errors: []
```

**6. User fills form, clicks Save — frontend sends action with form data:**

```yaml
type: "action"
name: "submit"
form: "save-user"
payload:
  name: "Charlie"
  email: "charlie@example.com"
  role: "user"
```

**7a. Validation fails — backend patches error state:**

```yaml
type: "patch"
data:
  - path: "/form/errors"
    value:
      - { path: "/form/data/email", message: "Email already exists" }
```

The form component displays errors because it binds to `/form/errors`.

**7b. Success — backend closes modal, refreshes table:**

```yaml
type: "event"
name: "close"
surface: "modal"
---
type: "patch"
data:
  - path: "/users/items/usr-3"  # Add by key
    value: { id: "usr-3", name: "Charlie", email: "charlie@example.com", role: "User" }
  - path: "/users/order"        # Update display order
    value: ["usr-1", "usr-2", "usr-3"]
```

### What the Frontend Provides

1. **The components** — side-nav, data-table, form fields, modal, etc.
2. **The mount points** — sidebar, main, modal, toast
3. **Data binding** — components react to data changes
4. **Action dispatch** — user interactions become messages
5. **API spec** — what endpoints the backend must implement

### What the Backend Provides

1. **Business logic** — what happens when actions arrive
2. **Data** — the actual content to display
3. **UI decisions** — what to render where, based on state
4. **Validation** — checking data, returning errors

The protocol connects them. The frontend doesn't know it's a "user management app" — it just renders what it's told.

## Implementation Plan

### Phase 1: OpenSDUI Protocol Specification

OpenAPI 3.1 defining the message formats, REST endpoints, and WebSocket events.

```
spec/
  openapi.yaml
  schemas/
    component.yaml
    data.yaml
    message.yaml
```

### Phase 2: Marionette Frontend (Svelte 5 + shadcn-svelte)

The "smart puppet" — renders whatever the backend sends, binds data reactively, sends actions on interaction.

### Phase 3: Marionette Backend (Rust + Axum)

The "puppet master" — a reference implementation proving the spec works.

### Phase 4: Working Application

Authentication, navigation, forms, tables. Proof that real apps can be built with OpenSDUI + Marionette.

## Key Decisions

| Decision | Rationale |
|----------|-----------|
| Adjacency list | Flat structure, easy patching, streaming-compatible |
| JSON Pointer binding | RFC standard, simple paths, scoped contexts |
| Open component types | String not enum — frontend defines vocabulary |
| Errors as data | No special error channel — bind to error state |
| Server-side i18n | Backend has context, simpler protocol |
| REST + WebSocket | REST for requests, WebSocket for push events |

## The Mental Model

**Backend is the puppet master.** It decides what UI to show, what data to display, what happens when users act.

**Frontend is a smart puppet.** It knows *how* to render components, *how* to bind data, *how* to capture actions. It doesn't know *what* the application does.

**The protocol is the strings.** It defines how puppet master and puppet communicate — not what stories they tell.

## LLM Considerations

The protocol and stack choices are informed by a world where LLMs increasingly write code.

### LLM as Code Author

| Layer | Why it works |
|-------|--------------|
| **Protocol (YAML/JSON)** | LLMs generate structured data more reliably than complex UI code. Flat adjacency lists are easy to produce incrementally. |
| **Rust backend** | Strong type system provides immediate compiler feedback. LLMs can self-correct against type errors. Minimal runtime magic. |
| **Svelte frontend** | Explicit reactivity, minimal boilerplate. What you write is what runs — no hidden framework behavior to hallucinate about. |

The component catalog acts as a constraint: LLMs don't need to generate accessible, production-quality UI components — they just reference them by type and props. The hard problems are solved once in the library.

### LLM as Backend (Speculative)

The protocol's structure makes it theoretically possible for an LLM to handle actions and emit renders directly, no traditional backend. Unproven economics and reliability, but the protocol doesn't prevent it.

---

*OpenSDUI is the protocol. Marionette proves it works.*
