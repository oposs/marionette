# Coding Conventions

**Analysis Date:** 2026-04-08

## Naming Patterns

**Files:**
- Svelte components: `PascalCase.svelte` (e.g., `Button.svelte`, `DataTable.svelte`, `NodeRenderer.svelte`)
- Svelte state files: `camelCase.svelte.ts` (e.g., `data.svelte.ts`, `sidebar.svelte.ts`)
- Plain TypeScript modules: `camelCase.ts` (e.g., `dispatcher.ts`, `registry.ts`)
- Test files (unit): `<subject>.test.ts` or `<subject>.svelte.test.ts` (e.g., `websocket.svelte.test.ts`)
- Test files (browser): `<subject>.browser-test.ts` (e.g., `Button.browser-test.ts`, `DataTable.browser-test.ts`)
- Rust: `snake_case.rs` with module names matching directory (e.g., `contact.rs`, `auth.rs`)

**Functions (TypeScript/Svelte):**
- Exported functions: `camelCase` (e.g., `sendAction`, `registerHandler`, `initMarionette`)
- Event handlers: `handle<Event>` prefix (e.g., `handleClick`)
- Boolean helpers: `is<State>` or `has<State>` (e.g., `isConnected`, `isDirty`)

**Variables:**
- TypeScript: `camelCase` for local variables and module-level state
- Svelte `$state` and `$derived`: `camelCase` (e.g., `let scrollTop = $state(0)`)
- Svelte component `$props()`: destructured inline with types
- Constants: `SCREAMING_SNAKE_CASE` for fixed values (e.g., `ROW_HEIGHT = 48`, `BUFFER = 5`, `FETCH_CHUNK = 100`)

**Types:**
- TypeScript interfaces/types: `PascalCase` (e.g., `ComponentAction`, `PatchOperation`, `CapturedFrame`)
- Rust types: `PascalCase` (e.g., `ActionRouter`, `AppState`, `HandlerEntry`)
- Rust enums: `PascalCase` variants (e.g., `ActionError::NotFound`, `ActionError::Unauthorized`)

## Code Style

**Formatting (Frontend):**
- Tool: Prettier with `prettier-plugin-svelte`
- Tabs (not spaces) for indentation
- Single quotes for strings
- No trailing commas
- Print width: 100 characters
- Svelte files use the `svelte` parser override

**Linting (Frontend):**
- Tool: ESLint with `typescript-eslint` and `eslint-plugin-svelte`
- Config: `eslint.config.js` using flat config format
- Standard recommended rules for JS, TS, and Svelte files
- Globals: browser + node (both available)

**Formatting (Backend):**
- Tool: `rustfmt` with `edition = "2024"` in `backend/rustfmt.toml`
- Clippy: `#![warn(clippy::pedantic)]` enabled on all crates
- Allowed: `#![allow(clippy::module_name_repetitions)]`

**TypeScript:**
- Strict mode enabled (`"strict": true` in `tsconfig.json`)
- `moduleResolution: "bundler"` (SvelteKit resolution)
- Path alias `$lib` maps to `src/lib/`

## Svelte 5 Component Patterns

**Props declaration:**
```svelte
<script lang="ts">
  let {
    props = {},
    bind,
    action,
    surface,
    children,
  }: {
    props: Record<string, unknown>;
    bind?: string;
    action?: ComponentAction;
    surface: string;
    children?: Snippet;
  } = $props();
</script>
```

**Reactive state:**
```svelte
let sortColumn = $state('');
let columns = $derived((props.columns as ColumnDef[]) ?? []);
```

**Component interface contract:** All renderable components accept a `surface: string` prop and optionally `props: Record<string, unknown>`, `bind?: string`, and `action?: ComponentAction`. This is a protocol-driven pattern.

**Children:** Use Svelte 5 `Snippet` type for `children` prop (not `$$slots`).

## Import Organization

**TypeScript/Svelte files order:**
1. External packages (e.g., `flowbite-svelte`, `vitest`)
2. `$lib/...` path alias imports (store, transport, registry)
3. Relative imports for sibling components (`./*.svelte`)
4. Type imports last (`import type { ... }`)

No explicit barrel files except `$lib/index.ts` which re-exports the entire public API.

**Rust import order (conventional):**
1. `std::` imports
2. External crates (alphabetical)
3. Internal crates (`marionette_protocol`, `marionette_macros`)
4. Local module imports (`crate::...`)

## Error Handling

**TypeScript:**
- Errors are not thrown across module boundaries; they flow via the protocol (`ErrorMessage`)
- Unknown message types use `console.warn` (not `throw`) to stay resilient
- WebSocket frame parse errors are silently caught with empty catch blocks: `catch { /* binary frame, skip */ }`
- Missing/fallback component types render `FallbackComponent.svelte` with `console.warn` (dev mode)

**Rust:**
- All action handler errors use the `ActionError` enum: `NotFound`, `Unauthorized`, `BadPayload`, `Internal`
- Return type for all handlers: `ActionResult = Result<Vec<ProtocolMessage>, ActionError>`
- `ActionError` implements `From<ActionError> for Vec<ProtocolMessage>` so errors auto-convert to protocol responses
- Never panic in handlers; use `ActionError::Internal(msg)` instead
- Use `?` operator to propagate `sea_orm` / `serde_json` errors, converting with `.map_err(|e| ActionError::Internal(e.to_string()))`

## Logging

**Frontend:**
- `console.warn` for unhandled protocol message types and unknown components
- `console.debug` for protocol events (prefixed `[marionette]`)
- No structured logging library

**Backend:**
- `tracing` crate with `tracing-subscriber`
- Macros: `info!`, `debug!`, `warn!`, `error!` from `tracing`
- Log levels used in `ws.rs`: `debug`, `info`, `warn`, `error`

## Comments

**When to comment:**
- Module-level: `//!` doc comments for crate/module purpose (e.g., `//! Standard component builders for all 18 protocol component types.`)
- Public items: `///` doc comments on all public functions, structs, and enum variants
- Inline: `//` comments for non-obvious logic or protocol constraints
- `// --` separator comments used in Rust to group related items visually

**JSDoc/TSDoc:**
- TypeScript: JSDoc `/** */` block comments on exported functions (e.g., `initMarionette`, `captureWebSocketFrames`, `sendAction`)
- Parameters documented with `@param` when non-obvious

## Function Design

**TypeScript:**
- Functions are small and single-purpose; handlers delegate to store/transport modules
- Functions return `void` when dispatching side effects (no return value needed)
- Optional parameters use `?: Type` (not default `= undefined`)
- Guard clauses: `if (!x) return;` before main logic

**Rust:**
- Builder pattern via `#[derive(ComponentBuilder)]` macro on component structs
- `#[must_use]` on builder methods that return `Self`
- Async handlers always return `ActionResult`
- Helper functions prefixed with their purpose: `now_sqlite()`, `tag_color()`, `deserialize_int_or_string()`

## Module Design

**Frontend exports:**
- Single public API file: `src/lib/index.ts` re-exports everything consumers need
- Internal modules do NOT re-export each other except through `index.ts`
- No barrel files within sub-directories; imports are always direct paths

**Rust:**
- `lib.rs` uses `pub mod` for all public modules and `pub use` to flatten common items
- All crates have `#![warn(clippy::pedantic)]` and `#![allow(clippy::module_name_repetitions)]`
- Protocol types in `marionette-protocol` crate; framework code in `marionette` crate; app code in `crm-demo` crate

---

*Convention analysis: 2026-04-08*
