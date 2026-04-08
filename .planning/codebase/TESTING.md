# Testing Patterns

**Analysis Date:** 2026-04-08

## Test Framework

**Runner (Frontend):**
- Vitest 4.x for unit and browser component tests
- Playwright 1.x for E2E and visual snapshot tests

**Assertion Library:**
- Vitest `expect` for unit tests
- `@testing-library`-style `expect.element(...)` via `vitest-browser-svelte` for browser tests
- Playwright `expect` for E2E tests

**Run Commands:**
```bash
# Frontend unit tests
cd frontend && npm test                  # Run all unit tests (vitest)

# Frontend browser component tests
cd frontend && npx vitest --config vitest-browser.config.ts  # Run browser tests (chromium)

# Frontend E2E tests (requires running dev server on :5173)
cd frontend && npx playwright test       # playwright.config.ts -- tests/e2e + tests/visual

# Backend integration E2E (builds frontend + starts backend on :3001)
cd frontend && npx playwright test --config playwright.e2e.config.ts

# Rust unit + integration tests
cd backend && cargo test
```

## Test File Organization

**Location:**
- Frontend unit tests: co-located with source, same directory as module
  - Pattern: `src/lib/**/<module>.test.ts` or `src/lib/**/<module>.svelte.test.ts`
- Frontend browser component tests: co-located with component
  - Pattern: `src/lib/components/**/<Component>.browser-test.ts`
- Frontend E2E + visual tests: `frontend/tests/` directory (separate from source)
  - `tests/e2e/` — integration specs using real WebSocket server
  - `tests/visual/` — Playwright screenshot regression tests
  - `tests/helpers/` — shared test utilities
  - `tests/__snapshots__/` — Playwright screenshot baseline files

**Naming:**
- Unit (node): `<module>.test.ts` or `<module>.svelte.test.ts`
- Browser component: `<Component>.browser-test.ts`
- E2E Playwright: `*.spec.ts`

**Rust test location:**
- Inline: `#[cfg(test)] mod tests { ... }` at bottom of source file (protocol crates, error, router)
- Integration: `backend/crates/<crate>/tests/integration_test.rs` (separate file, full server spin-up)
- Macro tests: `backend/crates/marionette/tests/macro_tests.rs`

## Test Structure

**Unit suite organization (TypeScript):**
```typescript
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';

describe('Module name', () => {
  beforeEach(() => {
    // Reset shared state via reset* functions
    resetStore('main');
    resetDirty();
    vi.clearAllMocks();
  });

  afterEach(() => {
    // Restore globals (timers, etc.)
    vi.useRealTimers();
  });

  it('action does expected thing', () => {
    // arrange
    // act
    // assert
    expect(result).toBe(expected);
  });
});
```

**Browser component test structure:**
```typescript
import { render } from 'vitest-browser-svelte';
import { expect, test, vi, beforeEach } from 'vitest';
import MyComponent from './MyComponent.svelte';

vi.mock('$lib/transport/dispatcher', () => ({
  sendAction: vi.fn(),
}));
import { sendAction } from '$lib/transport/dispatcher';

beforeEach(() => {
  vi.clearAllMocks();
});

test('renders correctly', async () => {
  const screen = await render(MyComponent, {
    props: { props: { label: 'Save' }, surface: 'test' },
  });
  await expect.element(screen.getByText('Save')).toBeVisible();
});
```

**Rust inline test pattern:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn feature_does_expected_thing() {
        let input = ...;
        let result = function_under_test(input);
        assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn async_feature_works() {
        // async test with tokio runtime
    }
}
```

**Patterns:**
- Each test has a descriptive name stating the expected behavior (not the implementation)
- `beforeEach` always resets module-level state (stores, mocks, timers)
- `afterEach` restores patched globals (timers, history.pushState, etc.)

## Mocking

**Framework:** Vitest `vi.*` APIs

**Module mocking (static import replacement):**
```typescript
// Mock BEFORE importing the module under test
vi.mock('$lib/transport/dispatcher', () => ({
  sendAction: vi.fn(),
}));
// Then import after mock declaration
import { sendAction } from '$lib/transport/dispatcher';
```

**Global stubbing:**
```typescript
// Stub WebSocket globally (must happen before module import)
vi.stubGlobal('WebSocket', MockWebSocket);

// Stub crypto.randomUUID
vi.stubGlobal('crypto', { randomUUID: vi.fn(() => 'test-uuid-1234') });
```

**Browser API mocking:**
```typescript
// Override browser APIs via Object.defineProperty
Object.defineProperty(window, 'location', {
  value: { pathname: '/contacts', href: 'http://localhost/contacts' },
  writable: true,
  configurable: true
});

// Spy on window methods
const addSpy = vi.spyOn(window, 'addEventListener');
```

**Fake timers:**
```typescript
vi.useFakeTimers();
vi.advanceTimersByTime(1500);  // advance past reconnect delay
vi.useRealTimers();  // restore in afterEach
```

**Module reset for fresh state:**
```typescript
beforeEach(async () => {
  vi.resetModules();
  module = await import('./module.svelte');
});
```

**What to Mock:**
- WebSocket (use `MockWebSocket` class with simulate* helpers)
- `$lib/transport/dispatcher` (`sendAction`) in component tests
- `crypto.randomUUID` for deterministic IDs
- Browser globals (`location`, `history.pushState`, `addEventListener`)
- `sea_orm::MockDatabase` for Rust handler unit tests

**What NOT to Mock:**
- Svelte stores (`data.svelte.ts`, `dirty.svelte.ts`) — test them directly using reset functions
- Protocol types — use real structs/messages
- The NodeRenderer + component registry — use real implementations in browser tests

## Fixtures and Factories

**Test data (TypeScript):**
```typescript
// Inline adjacency list for component trees
const nodes: Record<string, ComponentNode> = {
  root: { type: 'container', children: ['child1'] },
  child1: { type: 'text', props: { text: 'Hello' } },
};

// State setup via store functions
setFullState('test', {
  contacts: {
    row1: { id: '1', name: 'Alice', email: 'alice@example.com' },
  },
});
```

**Test data (Rust):**
```rust
fn mock_db() -> Arc<sea_orm::DatabaseConnection> {
    Arc::new(MockDatabase::new(DatabaseBackend::Sqlite).into_connection())
}

fn make_ctx(name: &str, session: Session) -> HandlerContext {
    HandlerContext {
        action: ActionMessage { name: name.into(), id: None, ... },
        db: mock_db(),
        session,
    }
}
```

**Location:**
- No dedicated fixtures directory; test data is declared inline within each test or test file
- Shared Playwright helpers in `frontend/tests/helpers/` (e.g., `ws-capture.ts`)
- Rust helper functions are plain `fn` inside `#[cfg(test)] mod tests` block

## Coverage

**Requirements:** None enforced (no coverage threshold configured)

**View Coverage:**
```bash
cd frontend && npx vitest --coverage   # generates coverage report
cd backend && cargo tarpaulin           # if tarpaulin installed
```

## Test Types

**Unit Tests (Frontend):**
- Scope: Individual modules — stores, transport, routing
- Environment: `node` (default in `vite.config.ts`)
- Exception: Router tests use `// @vitest-environment jsdom` directive for browser APIs
- Files: `src/lib/**/*.test.ts`, `src/lib/**/*.svelte.test.ts`

**Browser Component Tests (Frontend):**
- Scope: Single Svelte components rendered in real Chromium browser
- Framework: `vitest-browser-svelte` + `@vitest/browser-playwright`
- Config: `vitest-browser.config.ts`
- Files: `src/lib/components/**/*.browser-test.ts`
- Use `render(Component, { props: { ... } })` and `expect.element(...)` assertions

**E2E Tests (Frontend/Backend):**
- Playwright `*.spec.ts` files in `frontend/tests/e2e/`
- Two configs: `playwright.config.ts` (dev server on `:5173`) and `playwright.e2e.config.ts` (backend on `:3001`)
- Tests verify real WebSocket round-trips including frame capture via `captureWebSocketFrames(page)`

**Visual Snapshot Tests:**
- Playwright screenshot regression in `frontend/tests/visual/`
- Tolerance: `maxDiffPixels: 100` per screenshot
- Baselines stored in `frontend/tests/__snapshots__/`
- Tests: sidebar, form, and data table visual comparisons

**Rust Unit Tests:**
- Inline `#[cfg(test)] mod tests` for protocol serialization round-trips, error conversion, router dispatch
- `#[tokio::test]` for async handler tests
- `sea_orm::MockDatabase` used for DB-dependent tests (no real SQLite in unit tests)

**Rust Integration Tests:**
- File: `backend/crates/crm-demo/tests/integration_test.rs`
- Spins up a real `axum` server on a random port via `start_server()`
- Uses `tokio_tungstenite::connect_async` to send real WebSocket frames
- Verifies protocol JSON shape with `serde_json::Value` assertions
- Tests: hello exchange, navigate round-trip, demo_click patch, SPA fallback, health endpoint

## Common Patterns

**Async Testing (Rust):**
```rust
#[tokio::test]
async fn navigate_round_trip() {
    let (url, _) = start_server().await;
    let (mut ws, _) = connect_async(&url).await.unwrap();
    // skip hello
    let _ = ws.next().await.unwrap().unwrap();
    // send action and assert response
}
```

**Async Testing (TypeScript):**
```typescript
it('does something async', async () => {
  const onMsg = vi.fn();
  ws.connect('ws://test/ws', onMsg);
  MockWebSocket.latest().simulateOpen();
  // assertions
});
```

**Error Testing (TypeScript):**
```typescript
it('does not throw on unknown message type', () => {
  const warnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});
  expect(() => { handleMessage({ type: 'unknown-type' }); }).not.toThrow();
  expect(warnSpy).toHaveBeenCalledWith('Unhandled message type:', 'unknown-type');
  warnSpy.mockRestore();
});
```

**Error Testing (Rust):**
```rust
#[test]
fn error_converts_to_protocol_unauthorized() {
    let err = ActionError::Unauthorized("no access".into());
    let msgs: Vec<ProtocolMessage> = err.into();
    match &msgs[0] {
        ProtocolMessage::Error(ErrorMessage { errors, .. }) => {
            assert!(errors[0].message.contains("Unauthorized"));
        }
        other => panic!("Expected Error, got {other:?}"),
    }
}
```

**Polling for async E2E conditions (Playwright):**
```typescript
await expect.poll(() => frames.filter(f => f.direction === 'received').length, {
  timeout: 10000,
}).toBeGreaterThan(0);
```

---

*Testing analysis: 2026-04-08*
