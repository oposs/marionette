# Phase 4: Backend Toolkit - Research

**Researched:** 2026-03-20
**Domain:** Rust backend toolkit -- protocol types, derive macros, Axum WebSocket, action routing, SeaORM persistence, authorization
**Confidence:** HIGH

## Summary

Phase 4 builds the complete Marionette Rust backend toolkit across three crates: `marionette-protocol` (hand-written types matching the OpenAPI spec), `marionette-macros` (proc macros for `#[derive(ComponentBuilder)]` and `#[action]`), and `marionette` (Axum WebSocket integration, action routing, extractors, SeaORM patterns, authorization). All three crates are stubs today with only clippy configuration.

The protocol types are straightforward serde-based structs matching the 6 message types in `spec/schemas/message.yaml`. The derive macros use `syn`/`quote`/`proc-macro2` (already resolved in workspace). The Axum WebSocket integration uses axum's built-in `extract::ws` module. SeaORM 1.1.x with SQLite provides the persistence layer. The action routing system mirrors Axum's own router pattern with name-based dispatch and typed extractors.

**Primary recommendation:** Build bottom-up: protocol types first (foundation for everything), then derive macros (depend on protocol types), then the framework crate (WebSocket, routing, auth, SeaORM patterns), with tests at each layer.

<user_constraints>

## User Constraints (from CONTEXT.md)

### Locked Decisions
- **Component builder ergonomics:** Builder pattern (fluent method chain), NOT proc macro DSL. `#[derive(ComponentBuilder)]` generates typed builder methods from struct fields. Both `.child()` and `.children(vec![...])` for nesting.
- **Action routing:** Name-based dispatch with `#[action(name = "save-contact")]` derive macro. Generates action name constants and auto-registration. Axum-style typed extractors for handler parameters.
- **Database & persistence:** SQLite everywhere. SeaORM for ORM with migrations. Handlers ARE the business logic layer -- no separate entity-to-SDUI mapping framework.
- **Authorization:** Two-layer: `#[requires(authenticated)]` / `#[requires(role = "admin")]` declarative attributes + manual row-level checks inside handlers. Handlers without `#[requires]` are public.

### Claude's Discretion
- Exact WebSocket session management implementation (ping/pong, session state)
- SeaORM migration file structure and naming
- Error response format for unauthorized actions
- How the `#[derive(ComponentBuilder)]` macro generates the builder methods internally
- Integration test harness design (test database setup/teardown)

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope

</user_constraints>

<phase_requirements>

## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| BACK-01 | Axum handlers for serving SDUI responses | Axum WebSocket upgrade pattern, `extract::ws` module, State extractor |
| BACK-02 | Rust macros for ergonomic component construction | `#[derive(ComponentBuilder)]` using syn/quote, builder pattern generation |
| BACK-03 | Protocol message encoding/decoding | serde Serialize/Deserialize with `#[serde(tag = "type")]` tagged union |
| BACK-04 | Action routing and handler dispatch | Name-based router, typed extractors (Payload, Db, Session), `#[action]` macro |
| BACK-05 | SeaORM entity patterns for persistence | SeaORM 1.1.x with SQLite, DeriveEntityModel, migration patterns |
| BACK-06 | WebSocket session management | Axum WS upgrade, split sender/receiver, ping/pong, session state |
| BACK-07 | Permission/authorization utilities | `#[requires]` attribute macro, AuthInfo extractor, role checking |
| BACK-10 | Unit tests for component builders and macros | `cargo test` in each crate, compile-pass tests for macros |
| BACK-11 | Unit tests for message encoding/decoding | Round-trip serde tests, JSON schema conformance against spec/ |
| BACK-12 | Unit tests for action routing and dispatch | Router dispatch tests, extractor tests |
| BACK-13 | Integration tests for Axum handlers | `axum::test::TestClient` or tower::ServiceExt for in-process testing |
| BACK-14 | Integration tests for WebSocket session management | tokio-tungstenite client for WS integration tests |
| BACK-15 | SeaORM entity tests with test database | In-memory SQLite, migration runner, CRUD test patterns |

</phase_requirements>

## Standard Stack

### Core (Already in Workspace)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| axum | 0.8.8 | Web framework + WebSocket | Built-in WS support via `extract::ws`, Tower ecosystem |
| serde | 1.0.228 | Serialization framework | Industry standard for Rust JSON |
| serde_json | 1.0.149 | JSON encoding/decoding | Pairs with serde |
| tokio | 1.50.0 | Async runtime | Required by axum |
| tower-http | 0.6.8 | HTTP middleware (CORS, static files) | Official companion to axum |
| tracing | 0.1.44 | Structured logging | De facto Rust logging |
| syn | 2.0.117 | Rust source parsing for proc macros | Standard for derive macros |
| quote | 1.0.45 | Code generation for proc macros | Pairs with syn |
| proc-macro2 | 1.0.106 | Token stream utilities | Required by syn/quote |

### New Dependencies to Add
| Library | Version | Purpose | Crate |
|---------|---------|---------|-------|
| sea-orm | 1.1.19 | Async ORM with SQLite support | marionette, crm-demo |
| sea-orm-migration | 1.1.19 | Database schema migrations | crm-demo |
| uuid | 1.22.0 | Generate correlation IDs, entity keys | marionette-protocol |
| darling | 0.23.0 | Parse proc macro attributes into structs | marionette-macros |
| tokio-tungstenite | 0.29.0 | WebSocket client for integration tests | dev-dependency |
| futures | 0.3 | Stream utilities (SplitSink/SplitStream) | marionette |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| sea-orm | sqlx direct | SeaORM provides entity models, query builder -- chosen per TOOLING.md |
| darling | manual syn parsing | darling reduces boilerplate for attribute parsing significantly |
| sea-orm 2.0-rc | sea-orm 1.1.x | 2.0 is RC only, 1.1.19 is stable and production-ready |

**Installation (workspace Cargo.toml additions):**
```toml
[workspace.dependencies]
sea-orm = { version = "1.1", features = ["sqlx-sqlite", "runtime-tokio-rustls", "macros"] }
sea-orm-migration = { version = "1.1", features = ["sqlx-sqlite", "runtime-tokio-rustls"] }
uuid = { version = "1", features = ["v4", "serde"] }
darling = "0.23"
futures = "0.3"
tokio-tungstenite = "0.29"
```

## Architecture Patterns

### Recommended Crate Structure

```
backend/crates/
  marionette-protocol/src/
    lib.rs              # Re-exports
    messages.rs         # 6 message types (HelloMessage, RenderMessage, etc.)
    component.rs        # Component, ComponentAction structs
    data.rs             # PatchOperation, ValidationError, KeyedCollection
    common.rs           # Surface (type alias), MessageId, JsonPointer
  marionette-macros/src/
    lib.rs              # Proc macro entry points
    component_builder.rs # #[derive(ComponentBuilder)] implementation
    action.rs           # #[action] attribute macro implementation
    requires.rs         # #[requires] attribute macro implementation
  marionette/src/
    lib.rs              # Public API re-exports
    router.rs           # ActionRouter -- name-based dispatch
    extractors.rs       # Typed extractors: Payload<T>, Db, Session, AuthInfo
    session.rs          # WebSocket session management
    ws.rs               # WebSocket upgrade handler, message loop
    auth.rs             # Authorization checking, role verification
    builders/
      mod.rs            # Re-exports all standard component builders
      button.rs         # Button builder (example pattern)
      text_input.rs     # TextInput builder
      ...               # One file per component type
    error.rs            # Error types, error response helpers
```

### Pattern 1: Protocol Message Types (Tagged Union)

**What:** Serde-based enum with `#[serde(tag = "type")]` for the 6 message types
**When to use:** All message encoding/decoding

```rust
// marionette-protocol/src/messages.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ProtocolMessage {
    Hello(HelloMessage),
    Render(RenderMessage),
    Patch(PatchMessage),
    Action(ActionMessage),
    Event(EventMessage),
    Error(ErrorMessage),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderMessage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub surface: String,
    pub root: String,
    pub nodes: HashMap<String, Component>,
    pub data: serde_json::Value,
}
```

### Pattern 2: ComponentBuilder Derive Macro

**What:** `#[derive(ComponentBuilder)]` generates a typed builder from a props struct
**When to use:** Every component type definition

```rust
// User writes:
#[derive(ComponentBuilder)]
#[component(type = "text-input")]
pub struct TextInput {
    pub label: String,
    #[builder(optional)]
    pub placeholder: Option<String>,
    #[builder(optional)]
    pub required: Option<bool>,
    #[builder(optional)]
    pub input_type: Option<String>,
}

// Macro generates:
impl TextInput {
    pub fn new(label: impl Into<String>) -> TextInputBuilder { ... }
}
pub struct TextInputBuilder { /* fields */ }
impl TextInputBuilder {
    pub fn placeholder(mut self, v: impl Into<String>) -> Self { ... }
    pub fn required(mut self, v: bool) -> Self { ... }
    pub fn bind(mut self, path: impl Into<String>) -> Self { ... }
    pub fn action(mut self, action: ComponentAction) -> Self { ... }
    pub fn child(mut self, child: impl Into<Node>) -> Self { ... }
    pub fn children(mut self, children: Vec<Node>) -> Self { ... }
    pub fn build(self) -> Node { ... }
}
```

The macro uses `darling` to parse the `#[component]` and `#[builder]` attributes, `syn` to inspect the struct fields, and `quote` to generate the builder struct and impl block.

### Pattern 3: Action Router (Name-Based Dispatch)

**What:** Router that maps action names to handler functions with typed extractors
**When to use:** Central action dispatch in the WebSocket message loop

```rust
// User writes:
#[action(name = "save-contact")]
async fn save_contact(
    payload: Payload<SaveContactPayload>,
    db: Db,
    session: Session,
) -> ActionResult {
    // business logic
    Ok(vec![patch(...), event(...)])
}

// Registration:
let router = ActionRouter::new()
    .action(actions::SAVE_CONTACT, save_contact)
    .action(actions::NAVIGATE, navigate);

// Dispatch (inside WS loop):
let responses = router.dispatch(&action_msg, &app_state).await?;
```

### Pattern 4: WebSocket Session Management

**What:** Axum WS upgrade -> split into reader/writer tasks -> dispatch actions -> send responses
**When to use:** The `/ws` endpoint handler

```rust
async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_session(socket, state))
}

async fn handle_session(socket: WebSocket, state: AppState) {
    let (sender, receiver) = socket.split();
    let sender = Arc::new(Mutex::new(sender));

    // Send hello message
    send_message(&sender, &ProtocolMessage::Hello(HelloMessage {
        version: "1.0.0".into(),
    })).await;

    // Process incoming messages
    while let Some(Ok(msg)) = receiver.next().await {
        if let Message::Text(text) = msg {
            let action: ActionMessage = serde_json::from_str(&text)?;
            let responses = state.router.dispatch(&action, &state).await;
            for response in responses {
                send_message(&sender, &response).await;
            }
        }
    }
}
```

### Pattern 5: SeaORM Entity with SQLite

**What:** Entity definition with DeriveEntityModel following project SQL conventions
**When to use:** All database entities

```rust
// Following TOOLING.md conventions: singular table names, <table>_<field> columns
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "contact")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub contact_id: i32,
    pub contact_name: String,
    pub contact_email: String,
    pub contact_company: Option<i32>, // FK to company.company_id
}
```

### Anti-Patterns to Avoid
- **String-typed action names everywhere:** Use the generated constants from `#[action]` macro. Never duplicate action name strings between builders and handlers.
- **Nested component trees in Rust code:** Always produce flat adjacency list (HashMap of node ID -> Component), never nested structs. The builder's `.build()` method flattens.
- **Blocking in async handlers:** All DB access through SeaORM is already async. Never use `std::fs` or blocking I/O in handlers.
- **Global mutable state for sessions:** Use Axum's State extractor with Arc, not global variables.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Attribute parsing in macros | Manual TokenStream parsing | darling 0.23 | Handles optional fields, defaults, validation, error spans |
| WebSocket transport | Raw TCP/TLS | axum `extract::ws` | Handles upgrade, framing, ping/pong, close frames |
| Database migrations | Raw SQL files | sea-orm-migration | Tracks applied migrations, up/down, ordering |
| JSON serialization | Manual JSON building | serde + serde_json | Handles all edge cases, derives work with enums |
| UUID generation | Random strings | uuid crate | RFC 4122 compliant, multiple versions, serde support |
| Async streams | Manual poll loops | futures::StreamExt | `.split()`, `.next()`, combinators |

**Key insight:** The Rust ecosystem has mature solutions for every infrastructure problem in this phase. The novel work is the domain-specific design: how protocol types map to builders, how action routing works, how authorization integrates.

## Common Pitfalls

### Pitfall 1: Serde Tag Representation Mismatch
**What goes wrong:** Protocol messages don't round-trip correctly because serde's internally-tagged representation differs from the spec's expected JSON shape.
**Why it happens:** The spec uses `{"type": "render", "surface": "main", ...}` (internally tagged). Using `#[serde(tag = "type")]` on the enum is correct, but the variant data structs must NOT include a `type` field themselves -- serde adds it from the enum.
**How to avoid:** Use `#[serde(tag = "type", rename_all = "lowercase")]` on the enum. Each variant struct omits the `type` field. Write round-trip tests against the spec's example JSON.
**Warning signs:** Serialized JSON has `type` field nested or duplicated.

### Pitfall 2: Proc Macro Error Spans
**What goes wrong:** Compile errors from the derive macro point to the macro invocation site, not the actual problem field.
**Why it happens:** Using `proc_macro2::Span::call_site()` instead of preserving the span from the original tokens.
**How to avoid:** Use darling (which preserves spans) and always attach spans from the parsed input to generated tokens.
**Warning signs:** Error messages say "error at #[derive(ComponentBuilder)]" instead of pointing to the problematic field.

### Pitfall 3: SeaORM SQLite Feature Flags
**What goes wrong:** Compile errors about missing database driver or runtime.
**Why it happens:** SeaORM requires explicit feature flags for the database backend AND the async runtime.
**How to avoid:** Always specify both: `features = ["sqlx-sqlite", "runtime-tokio-rustls", "macros"]`. The `macros` feature enables `DeriveEntityModel`.
**Warning signs:** "no implementation for DatabaseConnection" or missing trait errors.

### Pitfall 4: WebSocket Message Ordering
**What goes wrong:** Multiple response messages (e.g., close-modal event + data patch) arrive out of order at the client.
**Why it happens:** If using separate tasks for sending different response types.
**How to avoid:** Send all response messages from a single handler dispatch sequentially through the same sender. Don't spawn separate tasks for individual response messages.
**Warning signs:** Modal closes before data updates, or data appears before navigation completes.

### Pitfall 5: Edition 2024 + Proc Macros
**What goes wrong:** Proc macro crate fails to compile or produces unexpected results with edition 2024.
**Why it happens:** Edition 2024 has some changes to how items are resolved. The proc macro crate itself compiles with its own edition, but the generated code executes in the caller's edition context.
**How to avoid:** Test macro output in a crate using edition 2024. Use fully qualified paths in generated code (e.g., `::std::string::String` not `String`).
**Warning signs:** Name resolution errors in code that looks correct.

### Pitfall 6: HashMap Serialization Order
**What goes wrong:** Snapshot tests fail intermittently because HashMap iteration order varies.
**Why it happens:** Rust's HashMap uses random hashing by default.
**How to avoid:** For tests: compare parsed Value objects, not JSON strings. For deterministic output in production: use `IndexMap` or `BTreeMap` for `nodes` if ordering matters.
**Warning signs:** Tests pass locally but fail in CI, or pass on retry.

## Code Examples

### Complete Message Type Definition
```rust
// marionette-protocol/src/messages.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::component::Component;
use crate::data::{PatchOperation, ValidationError};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ProtocolMessage {
    Hello(HelloMessage),
    Render(RenderMessage),
    Patch(PatchMessage),
    Action(ActionMessage),
    Event(EventMessage),
    Error(ErrorMessage),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HelloMessage {
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RenderMessage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub surface: String,
    pub root: String,
    pub nodes: HashMap<String, Component>,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PatchMessage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub patch: Vec<PatchOperation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ActionMessage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optimistic: Option<OptimisticUpdate>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OptimisticUpdate {
    pub patch: Vec<PatchOperation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventMessage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub surface: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ErrorMessage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub errors: Vec<ValidationError>,
}
```

### Round-Trip Test Pattern
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_message_round_trip() {
        let msg = ProtocolMessage::Render(RenderMessage {
            id: None,
            surface: "main".into(),
            root: "page-1".into(),
            nodes: HashMap::from([
                ("page-1".into(), Component {
                    r#type: "container".into(),
                    props: None,
                    children: Some(vec!["btn-1".into()]),
                    bind: None,
                    action: None,
                    visible: None,
                }),
            ]),
            data: serde_json::json!({"title": "Hello"}),
        });

        let json = serde_json::to_string(&msg).unwrap();
        let parsed: ProtocolMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, parsed);

        // Verify shape matches spec
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value["type"], "render");
        assert_eq!(value["surface"], "main");
    }
}
```

### Axum WebSocket Integration Test Pattern
```rust
// tests/ws_integration.rs
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures::{SinkExt, StreamExt};

#[tokio::test]
async fn test_websocket_hello() {
    // Start test server
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app()).await.unwrap();
    });

    // Connect WebSocket client
    let (mut ws, _) = connect_async(format!("ws://{addr}/ws"))
        .await.unwrap();

    // First message should be hello
    let msg = ws.next().await.unwrap().unwrap();
    let hello: ProtocolMessage = serde_json::from_str(
        msg.to_text().unwrap()
    ).unwrap();

    assert!(matches!(hello, ProtocolMessage::Hello(_)));
}
```

### SeaORM Test Database Pattern
```rust
#[cfg(test)]
mod tests {
    use sea_orm::{Database, DatabaseConnection, Schema};
    use sea_orm_migration::MigratorTrait;

    async fn setup_test_db() -> DatabaseConnection {
        let db = Database::connect("sqlite::memory:").await.unwrap();
        Migrator::up(&db, None).await.unwrap();
        db
    }

    #[tokio::test]
    async fn test_create_contact() {
        let db = setup_test_db().await;
        let contact = contact::ActiveModel {
            contact_name: Set("Alice".into()),
            contact_email: Set("alice@example.com".into()),
            ..Default::default()
        };
        let result = contact.insert(&db).await.unwrap();
        assert_eq!(result.contact_name, "Alice");
    }
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| sea-orm 0.x | sea-orm 1.1.x | 2024 | Stable API, better SQLite support |
| axum 0.7 | axum 0.8 | 2025 | API changes in extractors, routing |
| Manual attribute parsing | darling 0.23 | Ongoing | Much cleaner proc macro code |
| syn 1.x | syn 2.x | 2023 | Better error messages, edition 2021+ support |

**Deprecated/outdated:**
- `sea-orm 0.12`: Major API differences from 1.x. Use 1.1.x.
- `axum 0.7`: Extractor API changed in 0.8. Use current patterns.
- `warp` or `actix-web` for WS: Project uses axum per TOOLING.md.

## Open Questions

1. **SeaORM entity generation approach**
   - What we know: SeaORM supports both "entity first" (write entities, generate migrations) and "migration first" (write migrations, generate entities via CLI)
   - What's unclear: Which approach fits better when entities are simple and we want minimal tooling
   - Recommendation: Use "entity first" approach -- write entity structs by hand, write migrations by hand. Simpler than adding sea-orm-cli dependency. Entity structs double as documentation.

2. **Action handler return type design**
   - What we know: Handlers need to return one or more ProtocolMessages (render, patch, event, error)
   - What's unclear: Exact ergonomics of the return type
   - Recommendation: `ActionResult = Result<Vec<ProtocolMessage>, ActionError>` where ActionError auto-converts to ErrorMessage. Keeps it simple and composable.

3. **WebSocket sender sharing pattern**
   - What we know: Need to send responses back through the WS connection, possibly from multiple points
   - What's unclear: Whether Arc<Mutex<SplitSink>> or a channel (mpsc) is cleaner
   - Recommendation: Use `tokio::sync::mpsc` channel. Reader task dispatches actions and sends responses through the channel. Writer task drains the channel and sends to WS. Cleaner separation, no lock contention.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | cargo test (built-in) |
| Config file | None needed -- Cargo.toml per crate |
| Quick run command | `cd backend && cargo test --workspace` |
| Full suite command | `cd backend && cargo test --workspace` |

### Phase Requirements to Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| BACK-01 | Axum serves SDUI responses via WS | integration | `cargo test -p marionette --test ws_integration` | Wave 0 |
| BACK-02 | Component builder macros produce valid nodes | unit | `cargo test -p marionette --lib builders` | Wave 0 |
| BACK-03 | Message encoding/decoding matches spec | unit | `cargo test -p marionette-protocol` | Wave 0 |
| BACK-04 | Action routing dispatches to correct handler | unit | `cargo test -p marionette --lib router` | Wave 0 |
| BACK-05 | SeaORM entities CRUD with SQLite | integration | `cargo test -p marionette --test db_integration` | Wave 0 |
| BACK-06 | WebSocket session lifecycle (connect, hello, close) | integration | `cargo test -p marionette --test ws_integration` | Wave 0 |
| BACK-07 | Authorization blocks/allows based on role | unit | `cargo test -p marionette --lib auth` | Wave 0 |
| BACK-10 | Component builder unit tests | unit | `cargo test -p marionette --lib builders` | Wave 0 |
| BACK-11 | Message encoding unit tests | unit | `cargo test -p marionette-protocol` | Wave 0 |
| BACK-12 | Action routing unit tests | unit | `cargo test -p marionette --lib router` | Wave 0 |
| BACK-13 | Axum handler integration tests | integration | `cargo test -p marionette --test handler_integration` | Wave 0 |
| BACK-14 | WebSocket integration tests | integration | `cargo test -p marionette --test ws_integration` | Wave 0 |
| BACK-15 | SeaORM entity tests with test DB | integration | `cargo test -p marionette --test db_integration` | Wave 0 |

### Sampling Rate
- **Per task commit:** `cd backend && cargo test --workspace`
- **Per wave merge:** `cd backend && cargo test --workspace && cargo clippy --workspace -- -D warnings`
- **Phase gate:** Full suite green + clippy clean before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `backend/crates/marionette-protocol/src/messages.rs` -- protocol type definitions (BACK-03)
- [ ] Test infrastructure for all crates (currently stubs only)
- [ ] sea-orm and related dependencies in workspace Cargo.toml
- [ ] darling dependency in marionette-macros/Cargo.toml
- [ ] tokio-tungstenite as dev-dependency for WS integration tests

## Sources

### Primary (HIGH confidence)
- `spec/schemas/message.yaml` -- Authoritative message type schemas (6 types)
- `spec/schemas/component.yaml` -- Component and ComponentAction structure
- `spec/schemas/data.yaml` -- PatchOperation, ValidationError, KeyedCollection
- `spec/schemas/common.yaml` -- Surface, JsonPointer, MessageId
- `spec/PROTOCOL.md` -- Protocol manual with examples and patterns
- `backend/Cargo.toml` -- Workspace dependencies (axum 0.8, serde, tokio, etc.)
- Cargo metadata -- Resolved versions: axum 0.8.8, syn 2.0.117, quote 1.0.45
- [axum extract::ws docs](https://docs.rs/axum/latest/axum/extract/ws/index.html) -- WebSocket API
- [axum WebSocket example](https://github.com/tokio-rs/axum/blob/main/examples/websockets/src/main.rs) -- Official example

### Secondary (MEDIUM confidence)
- [sea-orm crates.io](https://crates.io/crates/sea-orm) -- Version 1.1.19 confirmed as latest stable
- [SeaORM migration docs](https://www.sea-ql.org/SeaORM/docs/migration/writing-migration/) -- Migration patterns
- [SeaORM entity docs](https://www.sea-ql.org/SeaORM/docs/generate-entity/entity-first/) -- Entity-first workflow

### Tertiary (LOW confidence)
- None -- all findings verified against official sources

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- all versions verified against cargo metadata and crates.io
- Architecture: HIGH -- patterns derived from official axum/serde docs and protocol spec
- Pitfalls: HIGH -- based on known Rust ecosystem gotchas with serde tags, proc macros, and SeaORM features
- Proc macro design: MEDIUM -- darling + syn/quote is well-established, but exact ComponentBuilder output shape needs iteration during implementation

**Research date:** 2026-03-20
**Valid until:** 2026-04-20 (stable ecosystem, slow-moving dependencies)
