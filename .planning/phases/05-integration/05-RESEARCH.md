# Phase 5: Integration - Research

**Researched:** 2026-03-23
**Domain:** Axum static file serving, WebSocket E2E testing, OpenAPI schema validation
**Confidence:** HIGH

## Summary

Phase 5 wires together the existing frontend (Phase 3) and backend (Phase 4) into a working end-to-end system. The codebase is well-prepared: the frontend already has a WebSocket transport connecting to `/ws` with hello handshake, a message dispatcher, and Surface rendering. The backend has `ws_handler`, `ActionRouter`, and component builders. The main integration work is: (1) configuring Axum to serve the built SvelteKit static files, (2) registering a demo action handler that returns a render message with components, (3) writing Playwright E2E tests that capture WebSocket frames and validate them against the OpenAPI schemas.

All required libraries are already in the workspace (`tower-http` with `fs` feature, `axum` with `ws` feature, `@playwright/test`). The frontend `adapter-static` with `fallback: 'index.html'` is already configured. The Vite proxy for `/ws` is already configured for dev mode. The crm-demo binary is a skeleton ready for Axum router setup.

**Primary recommendation:** Use tower-http `ServeDir` with `fallback(ServeFile)` for SPA serving, Playwright `page.on('websocket')` with `framereceived`/`framesent` events for WebSocket capture, and AJV with the extracted JSON schemas from `spec/schemas/` for protocol conformance validation.

<user_constraints>

## User Constraints (from CONTEXT.md)

### Locked Decisions
- Axum serves the built SvelteKit app (`frontend/build/`) as static files using tower-http ServeDir
- SPA fallback: all non-file routes serve `index.html` (adapter-static with `fallback: 'index.html'` already configured in Phase 1)
- `make build` produces both `frontend/build/` and the Rust binary
- Single WebSocket at `/ws` -- frontend connects on page load via the transport module (Phase 3)
- Backend ws_handler (Phase 4) upgrades the connection, sends hello, dispatches actions via ActionRouter
- Vite proxy handles `/ws` in dev mode; production serves everything from Axum directly
- Minimal but realistic demo: a "hello" screen that backend renders via the protocol
- Backend registers a `navigate` action handler that returns a render message with a few components (heading, text, button)
- Button click sends an action, backend responds with a patch -- proving the full round-trip
- No database required for the demo -- pure in-memory protocol exercise
- Validate WebSocket messages against the OpenAPI schemas at test time
- Use the bundled `spec/openapi.yaml` as the schema source
- Playwright E2E tests capture WebSocket frames and validate structure

### Claude's Discretion
- Exact demo screen content and component tree
- How to capture and validate WebSocket frames in Playwright tests
- Whether to add a `/api/health` REST endpoint for basic liveness checking
- Build script orchestration details in Makefile
- Error handling for missing `frontend/build/` directory

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope

</user_constraints>

<phase_requirements>

## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| INTEG-01 | Axum serves built Svelte app as static files | tower-http ServeDir with SPA fallback pattern, crm-demo main.rs Axum router setup |
| INTEG-02 | End-to-end message flow (action -> backend -> render -> frontend) | Demo action handler using existing ActionRouter + component builders, Playwright WebSocket frame capture |
| INTEG-03 | Protocol conformance validation against OpenAPI schemas | AJV JSON Schema validation in Playwright tests, schema extraction from spec/schemas/*.yaml |

</phase_requirements>

## Standard Stack

### Core (Already in Workspace)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| tower-http | 0.6.x | Static file serving via ServeDir + ServeFile | Already in workspace deps with `fs` feature |
| axum | 0.8.x | HTTP/WebSocket server | Already in workspace deps with `ws` feature |
| @playwright/test | ^1.58.2 | E2E testing with WebSocket frame capture | Already in frontend devDeps |

### Supporting (New for This Phase)
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| ajv | 8.x | JSON Schema 2020-12 validation | Validate WebSocket frames against OpenAPI schemas in Playwright tests |
| js-yaml | 4.x | Parse YAML schema files | Load spec/schemas/*.yaml in test helpers |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| AJV | zod (manual schema) | AJV validates directly against the OpenAPI YAML schemas we already have; zod would require duplicating schemas |
| js-yaml | bundled JSON copies | YAML parsing keeps schemas as single source of truth; JSON copies would drift |

**Installation:**
```bash
cd frontend && npm install --save-dev ajv ajv-formats js-yaml @types/js-yaml
```

## Architecture Patterns

### Axum Router Structure (crm-demo/src/main.rs)

The crm-demo binary needs an Axum router that combines WebSocket handling and static file serving:

```rust
// Source: tower-http docs + existing ws.rs pattern
use std::sync::Arc;
use axum::Router;
use axum::routing::any;
use tower_http::services::{ServeDir, ServeFile};
use marionette::ws::{ws_handler, AppState};
use marionette::router::{ActionRouter, box_handler};
use marionette_protocol::common::AuthRequirement;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let router = ActionRouter::new()
        .action("navigate", box_handler(handle_navigate), AuthRequirement::None);

    let state = Arc::new(AppState {
        router,
        db: /* mock or in-memory db */,
    });

    // Static files with SPA fallback
    let serve_dir = ServeDir::new("../frontend/build")
        .fallback(ServeFile::new("../frontend/build/index.html"));

    let app = Router::new()
        .route("/ws", any(ws_handler))
        .fallback_service(serve_dir)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
```

**Key insight:** The `/ws` route MUST be registered before the `fallback_service`. Axum checks named routes first, then falls through to the fallback service for everything else. This means `/ws` goes to the WebSocket handler and all other paths try static files, falling back to `index.html` for SPA routing.

### Demo Action Handler Pattern

Use existing component builders from Phase 4 to construct the render response:

```rust
use std::collections::HashMap;
use marionette::builders::standard::*;
use marionette::builders::node::Node;
use marionette_protocol::{Component, ProtocolMessage, RenderMessage};
use marionette::extractors::HandlerContext;
use marionette::error::ActionResult;

async fn handle_navigate(ctx: HandlerContext) -> ActionResult {
    let heading = Heading::new("Welcome to Marionette").id("heading-1").build();
    let text = Text::new("This demo proves the full protocol round-trip.").id("text-1").build();
    let button = Button::new("Click Me")
        .id("btn-1")
        .action(marionette_protocol::ComponentAction::click("demo_click"))
        .build();

    let nodes_vec = Container::new()
        .id("root")
        .children(vec![heading, text, button])
        .build_with_children();

    let mut nodes = HashMap::new();
    for (id, component) in nodes_vec {
        nodes.insert(id, component);
    }

    Ok(vec![ProtocolMessage::Render(RenderMessage {
        id: ctx.action.id.clone(),
        surface: "main".into(),
        root: "root".into(),
        nodes,
        data: serde_json::json!({ "greeting": "Hello from the backend!" }),
    })])
}
```

### Playwright WebSocket Frame Capture Pattern

```typescript
// Source: Playwright docs - WebSocket class API
import { test, expect } from '@playwright/test';

test('full round-trip: connect -> hello -> navigate -> render', async ({ page }) => {
    const messages: { direction: 'sent' | 'received'; data: unknown }[] = [];

    // Set up WebSocket listener BEFORE navigating
    page.on('websocket', (ws) => {
        ws.on('framesent', (frame) => {
            messages.push({ direction: 'sent', data: JSON.parse(frame.payload as string) });
        });
        ws.on('framereceived', (frame) => {
            messages.push({ direction: 'received', data: JSON.parse(frame.payload as string) });
        });
    });

    await page.goto('/');

    // Wait for the render to arrive and be visible
    await page.waitForSelector('[data-surface="main"]', { timeout: 5000 });

    // Validate captured messages
    const received = messages.filter(m => m.direction === 'received');
    expect(received.length).toBeGreaterThanOrEqual(1);

    // First received should be hello
    expect((received[0].data as Record<string, unknown>).type).toBe('hello');
});
```

### Schema Validation Helper Pattern

Since OpenAPI 3.1 uses JSON Schema 2020-12, and AJV supports 2020-12, we can extract individual message schemas from the YAML files and validate against them:

```typescript
import Ajv from 'ajv';
import addFormats from 'ajv-formats';
import yaml from 'js-yaml';
import fs from 'fs';
import path from 'path';

function loadSchemas(): Record<string, unknown> {
    const specDir = path.resolve(__dirname, '../../../spec/schemas');
    const messageSchema = yaml.load(fs.readFileSync(path.join(specDir, 'message.yaml'), 'utf8'));
    const componentSchema = yaml.load(fs.readFileSync(path.join(specDir, 'component.yaml'), 'utf8'));
    const dataSchema = yaml.load(fs.readFileSync(path.join(specDir, 'data.yaml'), 'utf8'));
    const commonSchema = yaml.load(fs.readFileSync(path.join(specDir, 'common.yaml'), 'utf8'));
    return { messageSchema, componentSchema, dataSchema, commonSchema };
}

function createValidator() {
    const ajv = new Ajv({ allErrors: true, strict: false });
    addFormats(ajv);
    // Register schemas and build validators for each message type
    // ... (resolve $ref cross-references between schema files)
    return ajv;
}
```

**Important caveat:** The OpenAPI schema files use cross-file `$ref` references (e.g., `"data.yaml#/PatchOperation"`). AJV does not natively resolve cross-file refs in this format. The pragmatic approach is to either:
1. Inline/flatten the schemas into a single JSON Schema document at test setup time
2. Register each schema file as a named schema and rewrite refs to use AJV's `$id` resolution

Recommendation: Write a small test helper that loads all four YAML files, merges them into a flat definitions map, and rewrites `$ref` values to use JSON Pointer format (`#/$defs/TypeName`). This is a one-time setup cost.

### Anti-Patterns to Avoid
- **Nesting WS route inside fallback_service:** The `/ws` route must be a named route on the Axum Router, not handled by ServeDir. ServeDir only serves files.
- **Hardcoded paths to frontend/build:** Use a configurable path or relative path from the binary's working directory. The Makefile should set the working directory correctly.
- **Blocking on WebSocket in E2E tests:** Always use event-based capture (`page.on('websocket')`) before navigation, never try to synchronously read frames after page load.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Static file serving | Custom file reader handler | tower-http ServeDir + ServeFile | Content-type detection, caching headers, directory listing safety, pre-compression support |
| SPA fallback routing | Path-matching middleware | ServeDir::fallback(ServeFile) | Handles all edge cases (encoded paths, directory traversal protection) |
| WebSocket frame capture | Custom WebSocket client in tests | Playwright `page.on('websocket')` events | Captures real browser WebSocket frames, no mock needed |
| JSON Schema validation | Manual field checking | AJV with spec YAML schemas | Validates against the actual protocol specification, catches drift |

## Common Pitfalls

### Pitfall 1: Frontend Build Not Available
**What goes wrong:** `make dev` starts the backend but `frontend/build/` does not exist (never ran `make build` or `npm run build`).
**Why it happens:** Dev mode uses Vite dev server (not built files), but integration tests may run against the Axum-served static files.
**How to avoid:** In dev mode, the backend does not need to serve static files (Vite proxy handles it). For integration testing and production, `make build` must run first. Add a startup check that logs a warning if the static directory is missing.
**Warning signs:** 404 errors for all frontend routes when hitting the Axum server directly.

### Pitfall 2: WebSocket Route vs Fallback Priority
**What goes wrong:** The `/ws` endpoint returns `index.html` instead of upgrading to WebSocket.
**Why it happens:** If `/ws` is not registered as a named route and only ServeDir is used, it tries to find a file named `ws` and falls back to `index.html`.
**How to avoid:** Register `/ws` as an explicit route on the Router BEFORE the fallback_service.

### Pitfall 3: Frontend Sends Hello But Backend Expects Action
**What goes wrong:** The frontend WebSocket transport (websocket.svelte.ts line 25) sends a `hello` message on connect. The backend `read_loop` tries to parse it as `ActionMessage` and returns an error.
**Why it happens:** The backend `handle_text_message` function only parses `ActionMessage`. The frontend sends `{ type: 'hello', version: '1.0.0' }` which is not an ActionMessage.
**How to avoid:** The backend read_loop must either: (a) parse as `ProtocolMessage` first and handle hello separately, or (b) silently ignore non-action messages, or (c) parse as generic JSON and check the `type` field before parsing as ActionMessage.
**Warning signs:** Error messages in WebSocket frames right after connection.

### Pitfall 4: Frontend Expects Navigate Action on Connect
**What goes wrong:** The frontend connects, receives hello, but no UI appears because no render message follows.
**Why it happens:** The frontend init module does not automatically send a `navigate` action -- it waits for the router module to trigger one based on the current URL.
**How to avoid:** Check how `initRouter(sendAction)` works. The router module (router.svelte.ts) likely sends a navigate action for the current URL path on init. Ensure the backend has a `navigate` handler registered that responds with a render message.

### Pitfall 5: AJV Cross-File $ref Resolution
**What goes wrong:** AJV throws "can't resolve reference" errors when validating against schemas that use cross-file refs like `"data.yaml#/PatchOperation"`.
**Why it happens:** AJV expects `$ref` to use JSON Pointer format with registered schema `$id` values, not file-path references.
**How to avoid:** Build a schema resolver helper that loads all YAML files, assigns `$id` values, and rewrites refs to internal format before registering with AJV.

### Pitfall 6: Playwright WebSocket Timing
**What goes wrong:** Test captures zero WebSocket frames.
**Why it happens:** `page.on('websocket')` listener was registered AFTER `page.goto()` completed, missing the initial connection.
**How to avoid:** Always register the WebSocket listener before calling `page.goto()`.

## Code Examples

### Axum Router with Static Files and WebSocket (INTEG-01)
```rust
// Source: tower-http ServeDir docs + axum routing
use tower_http::services::{ServeDir, ServeFile};

let spa = ServeDir::new("../frontend/build")
    .fallback(ServeFile::new("../frontend/build/index.html"));

let app = Router::new()
    .route("/ws", any(ws_handler))
    .fallback_service(spa)
    .with_state(state);
```

Note: `fallback()` preserves the original status (200 OK for index.html). Use `not_found_service()` instead if you want 404 status for non-file routes. For an SPA, `fallback()` returning 200 is correct since the client-side router handles the path.

### Demo Navigate Handler (INTEG-02)
```rust
// Use Phase 4 component builders
async fn handle_navigate(ctx: HandlerContext) -> ActionResult {
    let nodes_vec = Container::new()
        .id("root")
        .children(vec![
            Heading::new("Welcome").id("h1").build(),
            Text::new("Protocol demo").id("t1").build(),
            Button::new("Click")
                .id("btn1")
                .action(ComponentAction::click("demo_click"))
                .build(),
        ])
        .build_with_children();

    let mut nodes = HashMap::new();
    for (id, comp) in nodes_vec {
        nodes.insert(id, comp);
    }

    Ok(vec![ProtocolMessage::Render(RenderMessage {
        id: ctx.action.id.clone(),
        surface: "main".into(),
        root: "root".into(),
        nodes,
        data: serde_json::json!({}),
    })])
}
```

### Demo Click Handler Returning Patch (INTEG-02)
```rust
async fn handle_demo_click(_ctx: HandlerContext) -> ActionResult {
    Ok(vec![ProtocolMessage::Patch(PatchMessage {
        id: None,
        patch: vec![PatchOperation {
            path: "/message".into(),
            value: serde_json::json!("Button was clicked!"),
        }],
    })])
}
```

### Playwright WebSocket Frame Capture (INTEG-03)
```typescript
// Source: Playwright WebSocket API docs
import { test, expect, Page } from '@playwright/test';

interface CapturedFrame {
    direction: 'sent' | 'received';
    data: Record<string, unknown>;
    timestamp: number;
}

async function captureWebSocketFrames(page: Page): Promise<CapturedFrame[]> {
    const frames: CapturedFrame[] = [];
    page.on('websocket', (ws) => {
        ws.on('framesent', (frame) => {
            try {
                frames.push({
                    direction: 'sent',
                    data: JSON.parse(frame.payload as string),
                    timestamp: Date.now(),
                });
            } catch { /* binary frame, ignore */ }
        });
        ws.on('framereceived', (frame) => {
            try {
                frames.push({
                    direction: 'received',
                    data: JSON.parse(frame.payload as string),
                    timestamp: Date.now(),
                });
            } catch { /* binary frame, ignore */ }
        });
    });
    return frames;
}
```

### Health Endpoint (Optional, Claude's Discretion)
```rust
// Simple liveness check -- recommend adding for integration test readiness
async fn health() -> &'static str {
    "ok"
}

// Add to router:
let app = Router::new()
    .route("/ws", any(ws_handler))
    .route("/api/health", axum::routing::get(health))
    .fallback_service(spa)
    .with_state(state);
```

Recommendation: Add `/api/health` -- it is useful for Playwright `webServer` readiness checking and for the Makefile `dev` target to know when the backend is ready.

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Custom file serving handler | tower-http ServeDir | tower-http 0.4+ | No need for custom static file middleware |
| WebSocket mock servers in tests | Playwright native WebSocket capture | Playwright 1.12+ | Real browser WS frames, no mock needed |
| Playwright WebSocket mocking | page.routeWebSocket() | Playwright 1.48 | Can intercept/modify WS frames (not needed here -- we want real frames) |

## Open Questions

1. **Frontend hello message handling in backend**
   - What we know: Frontend sends `{ type: 'hello', version: '1.0.0' }` on connect (websocket.svelte.ts:25). Backend `handle_text_message` parses all text as `ActionMessage` which will fail for hello.
   - What's unclear: Should backend parse as ProtocolMessage and filter, or should frontend stop sending hello (since backend already sends its own)?
   - Recommendation: Modify `handle_text_message` in ws.rs to first try parsing as `ProtocolMessage`, then only dispatch if it is an Action. Hello from client can be silently acknowledged or ignored. Alternatively, remove the client-side hello send since the protocol spec has the server sending hello, not the client. **Check `spec/PROTOCOL.md` for the canonical handshake direction.**

2. **navigate action trigger on page load**
   - What we know: `initRouter(sendAction)` initializes the router with the ability to send actions. The router likely sends a navigate action for the current URL.
   - What's unclear: Exact mechanism -- need to verify router.svelte.ts sends navigate on init.
   - Recommendation: Planner should read `frontend/src/lib/routing/router.svelte.ts` to confirm the navigate action is sent automatically, or plan to add an explicit initial navigate action after hello is received.

3. **AppState.db for demo (no database)**
   - What we know: AppState requires `Arc<sea_orm::DatabaseConnection>`. Demo does not need a database.
   - What's unclear: Can we pass a mock/in-memory database, or should AppState.db become optional?
   - Recommendation: Use sea-orm MockDatabase or an in-memory SQLite connection. The Phase 4 tests already show the MockDatabase pattern (`MockDatabase::new(DatabaseBackend::Sqlite).into_connection()`).

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Playwright ^1.58.2 (E2E) + cargo test (backend unit) |
| Config file | `frontend/playwright.config.ts` |
| Quick run command | `cd frontend && npx playwright test tests/e2e/` |
| Full suite command | `cd backend && cargo test && cd ../frontend && npm test -- --run && npx playwright test` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| INTEG-01 | Axum serves static files and SPA fallback | E2E | `cd frontend && npx playwright test tests/e2e/static-serving.spec.ts -x` | Wave 0 |
| INTEG-02 | Action -> render -> patch round-trip | E2E | `cd frontend && npx playwright test tests/e2e/round-trip.spec.ts -x` | Wave 0 |
| INTEG-03 | Protocol messages match OpenAPI schemas | E2E | `cd frontend && npx playwright test tests/e2e/protocol-conformance.spec.ts -x` | Wave 0 |
| INTEG-02 | Backend navigate handler returns render | unit | `cd backend && cargo test -p crm-demo` | Wave 0 |

### Sampling Rate
- **Per task commit:** `cd frontend && npx playwright test tests/e2e/ --reporter=list`
- **Per wave merge:** Full suite (cargo test + vitest + playwright)
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `frontend/tests/e2e/static-serving.spec.ts` -- covers INTEG-01
- [ ] `frontend/tests/e2e/round-trip.spec.ts` -- covers INTEG-02
- [ ] `frontend/tests/e2e/protocol-conformance.spec.ts` -- covers INTEG-03
- [ ] `frontend/tests/helpers/schema-validator.ts` -- AJV schema loading/validation helper
- [ ] `frontend/tests/helpers/ws-capture.ts` -- WebSocket frame capture helper
- [ ] Install AJV + js-yaml: `cd frontend && npm install --save-dev ajv ajv-formats js-yaml @types/js-yaml`
- [ ] Update `frontend/playwright.config.ts` webServer command to start the built backend+frontend (not just Vite dev)
- [ ] Existing `tests/e2e/smoke.spec.ts` needs updating -- currently expects demo mode, should work with real backend

## Sources

### Primary (HIGH confidence)
- [tower-http ServeDir docs](https://docs.rs/tower-http/latest/tower_http/services/struct.ServeDir.html) -- fallback(), not_found_service(), SPA pattern
- [Playwright WebSocket API](https://playwright.dev/docs/api/class-websocket) -- framesent, framereceived events
- Existing codebase: `backend/crates/marionette/src/ws.rs`, `frontend/src/lib/init.ts`, `frontend/src/lib/transport/websocket.svelte.ts`

### Secondary (MEDIUM confidence)
- [Axum SPA discussion #867](https://github.com/tokio-rs/axum/discussions/867) -- confirmed ServeDir + fallback pattern
- [AJV JSON Schema validator](https://ajv.js.org/) -- 2020-12 support for OpenAPI 3.1 schemas

### Tertiary (LOW confidence)
- AJV cross-file $ref resolution -- needs validation during implementation; may require custom resolver

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- all core libraries already in workspace, patterns well documented
- Architecture: HIGH -- both frontend and backend are ready with clear integration points
- Pitfalls: HIGH -- identified through direct code inspection (hello message parsing, route priority)
- Schema validation: MEDIUM -- AJV cross-file ref handling needs validation during implementation

**Research date:** 2026-03-23
**Valid until:** 2026-04-23 (stable libraries, unlikely to change)
