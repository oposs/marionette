# OpenSDUI Protocol Specification

**Version:** 1.1.0

OpenSDUI is a server-driven UI protocol built on three primitives: **components**, **data**, and **messages**. A server describes what to render, a client knows how to render it. All communication flows over a single WebSocket connection using JSON text frames.

> This document is the authoritative protocol reference. See `CONCEPT.md` for design motivation and background.

## Three Primitives

The protocol rests on three building blocks:

1. **Components** -- typed UI nodes arranged in a flat adjacency list. The server declares *what* to show; the client decides *how* to show it.
2. **Data** -- JSON state that components bind to via JSON Pointer paths. Data flows down from the server and can be edited by the user through bound components.
3. **Messages** -- typed envelopes that carry components, data, and interactions between server and client.

Everything in the protocol is a composition of these three primitives.

## Transport

### WebSocket-Only

All protocol communication occurs over a single WebSocket connection at the `/ws` path. There are no REST endpoints for protocol messages. This protocol uses WebSocket exclusively.

The initial HTTP GET request serves the application shell (static files). Once loaded, the client opens a WebSocket connection and all subsequent communication uses that channel. JSON text frames carry every message in both directions.

### Connection Lifecycle

1. Client opens a WebSocket connection to `/ws`.
2. Server sends a `HelloMessage` containing the protocol version.
3. Client sends an initial action (typically a navigation action for the current URL).
4. Server responds with `RenderMessage`(s) for the active surfaces.
5. Ongoing communication: the client sends actions up, the server sends renders, patches, events, and errors down.

```
Client                              Server
  |                                    |
  |  ---- WebSocket CONNECT /ws ---->  |
  |  <--- hello { version: "1.1.0" }  |
  |  ---- action { name: "navigate",  |
  |         payload: { url: "/" } } -> |
  |  <--- render { surface: "main",   |
  |         root: ..., nodes: ...,     |
  |         data: ... }                |
  |                                    |
  |  (ongoing action/render/patch      |
  |   exchange)                        |
  |                                    |
```

### Reconnection

If the WebSocket connection drops, the client SHOULD reconnect using exponential backoff:

- **Initial delay:** 1 second
- **Maximum delay:** 30 seconds
- **Jitter:** +/- 500 milliseconds

These parameters are recommendations (SHOULD), not requirements (MUST). Implementations may adjust them to suit their environment.

On reconnection, the server re-sends the current render state for all active surfaces. The client treats a new connection as a fresh session from the protocol's perspective -- all prior surface state is replaced by the server's render messages.

### Keepalive

Use WebSocket transport-level ping/pong frames for connection liveness detection. The protocol does not define application-level ping or pong messages. Implementations SHOULD configure their WebSocket library's built-in ping interval (a typical value is 30 seconds).

## Messages

The protocol defines six message types, carried as a tagged union discriminated by the `type` field. See `schemas/message.yaml` for the machine-readable schema.

### hello

- **Direction:** Server to client (connection lifecycle)
- **Purpose:** Version handshake on connection establishment
- **Sent:** Exactly once, immediately after WebSocket connection opens

The `hello` message communicates the protocol version to the client. No response is expected -- the client proceeds to send its initial action.

**Fields:**

| Field     | Type   | Required | Description               |
|-----------|--------|----------|---------------------------|
| `type`    | string | yes      | Always `"hello"`          |
| `version` | string | yes      | Protocol version (semver) |

**Example:**

```yaml
type: hello
version: "1.1.0"
```

### render

- **Direction:** Server to client
- **Purpose:** Deliver a complete surface state -- component tree and associated data
- **When to use:** On initial load, after navigation, or when the server needs to replace an entire surface

A `render` message provides the full component tree and data for a single surface. It replaces any previous content for that surface. The `nodes` field is a flat map (adjacency list) of node IDs to component definitions. The `root` field identifies the tree's entry point.

**Fields:**

| Field     | Type   | Required | Description                                      |
|-----------|--------|----------|--------------------------------------------------|
| `type`    | string | yes      | Always `"render"`                                |
| `id`      | string | no       | Correlation ID (echoed from a triggering action) |
| `surface` | string | yes      | Target surface name (e.g., `"main"`, `"modal"`)  |
| `root`    | string | yes      | ID of the root node in `nodes`                   |
| `nodes`   | object | yes      | Flat map of node ID to Component                 |
| `data`    | object | yes      | Application state that components bind to        |

**Example -- a settings page:**

```yaml
type: render
surface: main
root: settings-page
nodes:
  settings-page:
    type: page
    props:
      title: Account Settings
    children:
      - timezone-field
      - language-field
      - save-btn
  timezone-field:
    type: select
    props:
      label: Timezone
      options:
        - { value: "UTC", label: "UTC" }
        - { value: "US/Eastern", label: "US Eastern" }
        - { value: "Europe/Zurich", label: "Europe/Zurich" }
    bind: "/settings/timezone"
  language-field:
    type: select
    props:
      label: Language
      options:
        - { value: "en", label: "English" }
        - { value: "de", label: "Deutsch" }
        - { value: "fr", label: "French" }
    bind: "/settings/language"
  save-btn:
    type: button
    props:
      label: Save Settings
      variant: primary
    action:
      type: submit
      name: save-settings
data:
  settings:
    timezone: "Europe/Zurich"
    language: "en"
```

### patch

- **Direction:** Server to client
- **Purpose:** Incrementally update a surface's data and/or component tree without a full re-render
- **When to use:** When part of a surface changes -- a data field, an added form row, a swapped-in sub-component -- and you want to preserve focus, cursor position, and unrelated component state

A `patch` message targets exactly one surface (via the required `surface` field) and carries a batch of `PatchOperation` entries applied in declared order, all-or-nothing. Data ops and node-tree ops can be mixed freely in one batch.

**Fields:**

| Field     | Type              | Required | Description                                       |
|-----------|-------------------|----------|---------------------------------------------------|
| `type`    | string            | yes      | Always `"patch"`                                  |
| `id`      | string            | no       | Correlation ID                                    |
| `surface` | string            | yes      | Target surface name (e.g., `"main"`, `"content"`) |
| `patch`   | PatchOperation[]  | yes      | Operations applied in declared order              |

Each `PatchOperation` is a tagged object discriminated by `op`:

#### Data operations

| `op`  | Payload fields     | Effect                                               |
|-------|--------------------|------------------------------------------------------|
| `set` | `path`, `value`    | Set `value` at JSON Pointer `path` in surface data   |

#### Node tree operations

| `op`            | Payload fields                | Effect                                                                            |
|-----------------|-------------------------------|-----------------------------------------------------------------------------------|
| `set-node`      | `id`, `component`             | Replace (or create) the component at node `id` in the surface's adjacency list    |
| `delete-node`   | `id`                          | Remove the node with `id` from the adjacency list                                 |
| `set-children`  | `id`, `children: string[]`    | Replace `id`'s children array with the given ordered list of child IDs            |
| `insert-child`  | `parent`, `index`, `childId`  | Insert `childId` into `parent`'s children array at zero-based `index`             |
| `remove-child`  | `parent`, `childId`           | Remove `childId` from `parent`'s children array                                   |

**Root immutability:** A `root` pointer is immutable for the lifetime of a `Render`. Node patches can replace the component AT the root ID (via `set-node`) or mutate its children, but cannot re-point `root` to a different ID. Top-level transitions (login → shell, error → recovery) use a full `render` message instead.

**Unknown ops:** A `PatchMessage` containing an `op` not in this table is an error. Clients surface this as a stale-client prompt and should reload. Protocol version negotiation via `HelloMessage.version` is the mechanism for forward compatibility -- see §Protocol Versioning and §Stale Client Handling.

**Focus preservation:** Frontends applying node patches must mutate node map entries in place rather than replacing parent tree objects wholesale. This guarantees that a focused input field retains its focus and cursor position across arbitrary patches to sibling nodes in the same surface.

**Example -- data + tree ops mixed, swap a form field on country select:**

```yaml
type: patch
surface: content
patch:
  - op: set
    path: "/contact/country"
    value: "CH"
  - op: insert-child
    parent: contact-form
    index: 4
    childId: contact-canton
  - op: set-node
    id: contact-canton
    component:
      type: select
      bind: "/contact/canton"
      props:
        label: "Canton"
        options:
          - { value: "ZH", label: "Zürich" }
          - { value: "BE", label: "Bern" }
  - op: delete-node
    id: contact-us-state
```

### action

- **Direction:** Client to server
- **Purpose:** Report a user interaction or navigation event
- **When to use:** When the user clicks a button, submits a form, navigates, or any other interaction that requires server processing

The `action` message carries user intent to the server. The `name` field identifies what happened. The optional `source` field indicates which component triggered it. The optional `optimistic` field contains patches to apply immediately on the client for responsive UI (see [Optimistic Updates](#optimistic-updates)).

**Fields:**

| Field        | Type   | Required | Description                                          |
|--------------|--------|----------|------------------------------------------------------|
| `type`       | string | yes      | Always `"action"`                                    |
| `id`         | string | no       | Correlation ID (set by client, echoed by server)     |
| `name`       | string | yes      | Action identifier                                    |
| `source`     | string | no       | Component ID that triggered the action               |
| `payload`    | object | no       | Action-specific data                                 |
| `optimistic` | object | no       | Contains `patch` array for immediate client-side application |

**Example -- navigate to a contact detail page:**

```yaml
type: action
id: "msg-9f3a"
name: navigate
payload:
  url: "/contacts/c-007"
```

**Example -- submit an edited contact:**

```yaml
type: action
id: "msg-b2e1"
name: save-contact
source: save-btn
payload:
  phone: "+41 44 632 1111"
  notes: "Updated phone number"
optimistic:
  patch:
    - path: "/ui/saving"
      value: true
```

### event

- **Direction:** Server to client
- **Purpose:** Signal something happened without delivering a full re-render
- **When to use:** Closing a modal, triggering a toast notification, hinting that data should be refreshed

Events are lightweight signals. They carry a `name` and optional metadata (`hint`). They can target a specific surface.

**Fields:**

| Field     | Type   | Required | Description                                |
|-----------|--------|----------|--------------------------------------------|
| `type`    | string | yes      | Always `"event"`                           |
| `id`      | string | no       | Correlation ID                             |
| `name`    | string | yes      | Event identifier                           |
| `surface` | string | no       | Target surface (if event is surface-specific) |
| `hint`    | object | no       | Event-specific metadata                    |

**Example -- close a modal after successful save:**

```yaml
type: event
name: close
surface: modal
```

**Example -- hint that contact data has changed:**

```yaml
type: event
name: data-changed
hint:
  paths:
    - "/contacts"
```

### error

- **Direction:** Server to client
- **Purpose:** Report protocol-level errors
- **When to use:** Malformed action, unknown surface, internal server error, or any condition where normal message processing failed

The `error` message carries an array of error objects, each with a required `message` and optional `path`. This message type is for **protocol-level errors** -- transport failures, malformed messages, unknown references. Field validation errors use a different mechanism (see [Error Handling](#error-handling)).

**Fields:**

| Field    | Type              | Required | Description                    |
|----------|-------------------|----------|--------------------------------|
| `type`   | string            | yes      | Always `"error"`               |
| `id`     | string            | no       | Correlation ID (echoed from the action that caused the error) |
| `errors` | ValidationError[] | yes      | Array of error objects          |

Each error object has:

| Field     | Type   | Required | Description                                       |
|-----------|--------|----------|---------------------------------------------------|
| `path`    | string | no       | JSON Pointer to the relevant data location         |
| `message` | string | yes      | Human-readable error description                   |

**Example -- action references an unknown surface:**

```yaml
type: error
id: "msg-9f3a"
errors:
  - message: "Unknown surface: 'drawer'"
```

**Example -- server encountered an internal error:**

```yaml
type: error
errors:
  - message: "Internal server error while processing action 'delete-contact'"
  - path: "/contacts/c-007"
    message: "Record is locked by another operation"
```

## Components

### Adjacency List

Components are represented as a flat map of string IDs to `Component` objects -- the adjacency list pattern. A `root` pointer identifies the entry node. Children are referenced by their IDs, not nested inline.

```yaml
root: "list-page"
nodes:
  list-page:
    type: page
    props:
      title: Contacts
    children:
      - search-bar
      - contact-list
  search-bar:
    type: text-input
    props:
      label: Search
      placeholder: "Filter by name..."
    bind: "/ui/searchTerm"
  contact-list:
    type: data-table
    props:
      columns:
        - { key: "name", label: "Name", sortable: true }
        - { key: "email", label: "Email" }
        - { key: "phone", label: "Phone" }
      keyField: "id"
    bind: "/contacts"
    action:
      type: navigate
      idPath: "/id"
```

This flat structure is easy to patch (update one node by ID), streaming-friendly (send nodes as they become ready), and straightforward for tooling to process.

### Component Structure

Each component has the following fields (see `schemas/component.yaml`):

| Field      | Type     | Required | Description                                                  |
|------------|----------|----------|--------------------------------------------------------------|
| `type`     | string   | yes      | Component type identifier (open set)                         |
| `props`    | object   | no       | Component-specific properties (`additionalProperties: true`) |
| `children` | string[] | no       | Ordered list of child node IDs                               |
| `bind`     | string   | no       | JSON Pointer (RFC 6901) to the data this component reads/writes |
| `action`   | object   | no       | ComponentAction triggered by the primary interaction         |
| `visible`  | string   | no       | JSON Pointer to a boolean controlling visibility             |

**Component types are an open set.** The protocol does not enumerate valid component types -- that vocabulary is defined by the frontend library. The `type` field is a plain string: `"text-input"`, `"data-table"`, `"chart"`, or any string the frontend recognizes.

**Strict envelope, open props.** The `Component` schema uses `additionalProperties: false` to enforce a strict envelope -- only the six fields above are allowed at the top level. However, `props` uses `additionalProperties: true`, allowing each component type to define its own properties freely.

### ComponentAction

The `action` field on a component defines what happens when the user interacts with it (click, submit, etc.). See `schemas/component.yaml#/ComponentAction`.

| Field    | Type   | Required | Description                            |
|----------|--------|----------|----------------------------------------|
| `type`   | string | yes      | Action type identifier                 |
| `name`   | string | no       | Action name                            |
| `target` | string | no       | Target identifier (e.g., route path)   |
| `idPath` | string | no       | JSON Pointer to an ID within row data  |

`ComponentAction` uses `additionalProperties: true` for extensibility.

## Data Binding

Components bind to application data using JSON Pointer paths ([RFC 6901](https://datatracker.ietf.org/doc/html/rfc6901)).

### Path Syntax

JSON Pointers use forward slashes to navigate into nested objects:

| Path                        | Resolves to                      |
|-----------------------------|----------------------------------|
| `/settings/timezone`        | `data.settings.timezone`         |
| `/contacts/c-007/name`      | `data.contacts["c-007"].name`    |
| `/ui/loading`               | `data.ui.loading`                |
| `/notifications/n-1/read`   | `data.notifications["n-1"].read` |

### Two-Way Binding

When a component declares `bind: "/settings/timezone"`:

- **Read:** The component displays the current value of `data.settings.timezone`.
- **Write:** When the user changes the value (e.g., selects a different timezone), the client updates `data.settings.timezone` locally.

Data is delivered to the client in `RenderMessage` and updated incrementally via `PatchMessage`. Components reactively reflect the current data state.

### Example

```yaml
# Component declaration
nodes:
  email-field:
    type: text-input
    props:
      label: "Email Address"
      required: true
    bind: "/contact/email"

# Data the component binds to
data:
  contact:
    email: "info@example.ch"
```

The `email-field` component reads its value from `data.contact.email` and writes user edits back to the same path.

## Keyed Collections

Collections in data use objects with stable string keys, NOT arrays.

### Why Not Arrays?

Array indices are unstable. When an item is inserted or deleted, all subsequent indices shift. A patch targeting `/contacts/2/name` may hit the wrong record after a deletion. This creates race conditions and data corruption in real-time UIs.

### The Pattern

Collections are keyed objects where each key is a stable identifier:

```yaml
data:
  contacts:
    "c-001":
      id: "c-001"
      name: "Maria Bernasconi"
      email: "maria@example.ch"
    "c-002":
      id: "c-002"
      name: "Hans Meier"
      email: "hans@example.ch"
    "c-003":
      id: "c-003"
      name: "Sophie Duval"
      email: "sophie@example.fr"
  contactOrder:
    - "c-001"
    - "c-002"
    - "c-003"
```

Display order is maintained in a separate array (e.g., `contactOrder`). This separates identity (stable keys) from presentation (display order).

### Patching by Key

Patches target specific records by their key path:

```yaml
type: patch
patch:
  - path: "/contacts/c-002/email"
    value: "hans.meier@example.ch"
```

This always updates Hans Meier's email, regardless of how the collection is sorted or whether other items have been added or removed.

### Adding and Removing Items

To add an item, patch the new key and update the order array:

```yaml
type: patch
patch:
  - path: "/contacts/c-004"
    value:
      id: "c-004"
      name: "Luca Rossi"
      email: "luca@example.it"
  - path: "/contactOrder"
    value: ["c-001", "c-002", "c-003", "c-004"]
```

To remove an item, set its value to `null` and update the order array:

```yaml
type: patch
patch:
  - path: "/contacts/c-003"
    value: null
  - path: "/contactOrder"
    value: ["c-001", "c-002", "c-004"]
```

## Optimistic Updates

Optimistic updates are a core protocol feature, not an optional extension.

### Mechanism

When a user performs an action that has a predictable immediate effect, the client can apply data patches instantly without waiting for the server. The `optimistic` field on an `ActionMessage` contains the patches to apply:

```yaml
type: action
id: "msg-7c4d"
name: save-contact
source: save-btn
payload:
  name: "Maria Bernasconi-Fischer"
optimistic:
  patch:
    - path: "/ui/saving"
      value: true
    - path: "/contacts/c-001/name"
      value: "Maria Bernasconi-Fischer"
```

### Lifecycle

1. **Client sends action** with `optimistic.patch` array.
2. **Client applies patches immediately** -- the UI reflects the changes without network delay.
3. **Server processes the action** and responds:
   - **Success:** Server sends an authoritative `PatchMessage` confirming the state. The optimistic patches are superseded by the server's authoritative data.
   - **Failure:** Server sends an `ErrorMessage`. The client rolls back the optimistic patches to restore the previous data state.

### Example: Toggle a Notification as Read

```yaml
# Client sends:
type: action
id: "msg-e8f2"
name: mark-read
source: "notif-n-42"
payload:
  notificationId: "n-42"
optimistic:
  patch:
    - path: "/notifications/n-42/read"
      value: true
    - path: "/ui/unreadCount"
      value: 3

# On success, server confirms with authoritative state:
type: patch
id: "msg-e8f2"
patch:
  - path: "/notifications/n-42/read"
    value: true
  - path: "/notifications/n-42/readAt"
    value: "2026-03-18T10:45:00Z"
  - path: "/ui/unreadCount"
    value: 3

# On failure, server sends error and client rolls back:
type: error
id: "msg-e8f2"
errors:
  - message: "Notification n-42 not found"
```

## Error Handling

The protocol distinguishes two error mechanisms. This distinction is important -- conflating them leads to incorrect implementations.

### ErrorMessage (Protocol-Level Errors)

The `error` message type (see [Messages > error](#error)) is for **protocol-level errors**: malformed actions, unknown surfaces, server crashes, authorization failures. These are exceptional conditions that disrupt normal message processing.

```yaml
type: error
errors:
  - message: "Action 'delete-all' requires admin authorization"
```

An `ErrorMessage` contains an `errors` array. Each entry has a required `message` (human-readable) and an optional `path` (JSON Pointer to the relevant data location).

### Validation Errors as Data

**Field-level validation errors are data patches**, not error messages. The server patches error information into the data model, and components bind to it. This keeps validation state in the reactive data flow.

```yaml
# Server patches validation errors into data:
type: patch
patch:
  - path: "/contactForm/errors"
    value:
      - path: "/contactForm/data/email"
        message: "Invalid email address"
      - path: "/contactForm/data/phone"
        message: "Phone number must include country code"
```

A component bound to `/contactForm/errors` displays these errors inline. When the user corrects the fields and resubmits, the server either patches the errors to an empty array (valid) or patches new errors (still invalid).

### When to Use Which

| Situation                          | Mechanism                    |
|------------------------------------|------------------------------|
| Malformed action payload           | `ErrorMessage`               |
| Unknown surface reference          | `ErrorMessage`               |
| Server internal error              | `ErrorMessage`               |
| Authorization failure              | `ErrorMessage`               |
| Invalid email format in a form     | Validation error as data patch |
| Required field left blank          | Validation error as data patch |
| Duplicate entry in a collection    | Validation error as data patch |

**Rule of thumb:** If the error relates to a user-visible form field, it belongs in data. If it relates to protocol mechanics or server health, it belongs in an `ErrorMessage`.

## Surfaces

Surfaces are named render targets in the frontend layout. Each surface maintains an independent component tree and data store.

### Concept

A surface is simply a string identifier that tells the frontend *where* to render a component tree. Common surface names include:

- `main` -- the primary content area
- `sidebar` -- a navigation or secondary panel
- `modal` -- an overlay dialog
- `toast` -- transient notification area

These names are examples, not a fixed set. The frontend defines which surfaces it supports and where they appear in the layout.

### Independent State

Each surface has its own component tree and data. A `RenderMessage` targeting `"modal"` does not affect the `"main"` surface. This allows the server to update a modal independently of the page content behind it.

### Surface-Targeted Events

Events can target a specific surface:

```yaml
type: event
name: close
surface: modal
```

This tells the frontend to close the modal surface. The main surface remains unaffected.

### Multi-Surface Rendering

A single user action may trigger multiple render messages targeting different surfaces:

```yaml
# Action: user clicks "New Contact"
type: action
name: open-form
payload:
  entity: contact

# Server responds with two messages:

# 1. Render form in modal
type: render
surface: modal
root: contact-form
nodes:
  contact-form:
    type: modal
    props:
      title: "New Contact"
    children:
      - name-field
      - email-field
      - form-actions
  name-field:
    type: text-input
    props:
      label: Name
      required: true
    bind: "/contactForm/data/name"
  email-field:
    type: text-input
    props:
      label: Email
    bind: "/contactForm/data/email"
  form-actions:
    type: button-group
    children:
      - cancel-btn
      - save-btn
  cancel-btn:
    type: button
    props:
      label: Cancel
      variant: secondary
    action:
      type: close-modal
  save-btn:
    type: button
    props:
      label: Save
      variant: primary
    action:
      type: submit
      name: save-contact
data:
  contactForm:
    data:
      name: ""
      email: ""
    errors: []

# 2. Patch the main surface to show a visual cue
type: patch
patch:
  - path: "/ui/modalOpen"
    value: true
```

## Protocol Versioning

The protocol version is communicated via the `HelloMessage` sent by the server on every new WebSocket connection.

```yaml
type: hello
version: "1.1.0"
```

### Deployment Model

The backend and frontend are deployed together as a single bundle. Under normal operation, there is no version mismatch -- both sides speak the same protocol version.

### Stale Client Handling

The `version` field enables graceful handling of cached or stale frontends. If a client connects with a cached frontend that expects a different protocol version, it can detect the mismatch from the `hello` message and prompt the user to reload.

There is no in-band version negotiation. A version mismatch means the client should reconnect after updating (typically a page reload).

## Schema Reference

The machine-readable protocol specification is defined as an OpenAPI 3.1 document with JSON Schema definitions (draft 2020-12).

### Entry Point

- **`spec/openapi.yaml`** -- OpenAPI 3.1 entry point. References all schema files. Defines the `ProtocolMessage` tagged union via webhooks (since there are no REST paths).

### Schema Files

| File                       | Contents                                                      |
|----------------------------|---------------------------------------------------------------|
| `schemas/common.yaml`      | `Surface`, `JsonPointer`, `MessageId` -- shared primitive types |
| `schemas/component.yaml`   | `Component`, `ComponentAction` -- UI node structure            |
| `schemas/data.yaml`        | `PatchOperation`, `KeyedCollection`, `ValidationError` -- data patterns |
| `schemas/message.yaml`     | `ProtocolMessage` (tagged union), `HelloMessage`, `RenderMessage`, `PatchMessage`, `ActionMessage`, `EventMessage`, `ErrorMessage` |

### Schema Conventions

- All schemas use JSON Schema draft 2020-12 (aligned with OpenAPI 3.1).
- Cross-file references use relative paths: `common.yaml#/Surface` (from within `schemas/`), `schemas/common.yaml#/Surface` (from `openapi.yaml`).
- Within-file references use fragment-only syntax: `#/ComponentAction`.
- Message types use `const` on the `type` field for discriminator support.
