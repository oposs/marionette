# Technology Stack

**Analysis Date:** 2026-04-08

## Languages

**Primary:**
- Rust 1.93.1 - Backend (all crates in `backend/`)
- TypeScript 5.9 - Frontend (`frontend/src/`)
- Svelte 5.53 - Frontend UI components

**Secondary:**
- YAML - Protocol specification (`spec/openapi.yaml`, `spec/schemas/`)

## Runtime

**Backend:**
- Tokio async runtime (full features) via `tokio = "1"`
- Listens on `0.0.0.0:3001`

**Frontend:**
- Node.js 25.4.0

**Package Manager:**
- Backend: Cargo 1.93.1, lockfile `backend/Cargo.lock` present
- Frontend: npm, lockfile `frontend/package-lock.json` present

## Frameworks

**Backend Core:**
- `axum 0.8` - HTTP server and WebSocket upgrade handler
- `axum-extra 0.12` (cookie features) - HTTP-only session cookie management
- `tower-http 0.6` (fs, cors) - Static file serving with SPA fallback
- `tokio 1` (full) - Async runtime

**Backend ORM:**
- `sea-orm 1.1` (sqlx-sqlite, runtime-tokio-rustls, macros) - ORM and query builder
- `sea-orm-migration 1.1` - Schema migrations

**Backend Macro Crate:**
- `marionette-macros` (internal) - Procedural macros using `syn 2`, `quote 1`, `darling 0.23`

**Frontend Framework:**
- `@sveltejs/kit 2.55` with `@sveltejs/adapter-static 3.0` - SPA mode, outputs to `frontend/build/`
- `vite 7.0` with `@tailwindcss/vite 4.2` plugin
- `tailwindcss 4.2` - Utility CSS
- `flowbite-svelte 1.31` + `flowbite-svelte-icons 3.1` - UI component library

**Frontend Routing:**
- Custom Marionette router in `frontend/src/lib/routing/router.svelte.ts` (server-driven)
- Dev proxy: Vite proxies `/api` and `/ws` to `http://localhost:3001`

**Testing:**
- Backend: `cargo test`, `wiremock 0.6` for HTTP mocking
- Frontend unit: `vitest 4.1` with node environment
- Frontend browser component: `vitest-browser-svelte 2.1` with Playwright/Chromium
- Frontend E2E: `@playwright/test 1.58` against `http://localhost:5173`

**Spec/Documentation:**
- `@redocly/cli 2.24` - OpenAPI spec linting and bundling
- `@stoplight/spectral-cli 6.15` - Additional spec linting

**Build/Dev:**
- `make dev` - runs both backend and frontend concurrently
- `make build` - builds frontend static assets then compiles backend binary
- `mise.toml` - tool version management (node latest, rust latest, rust-analyzer latest)

## Key Dependencies

**Critical:**
- `serde 1` + `serde_json 1` - All protocol serialization/deserialization
- `uuid 1` (v4, serde) - Session tokens and component IDs
- `bcrypt 0.19` - Password hashing (spawned in `tokio::task::spawn_blocking`)
- `chrono 0.4` (serde) - Timestamp handling throughout
- `futures 0.3` - Stream utilities for WebSocket split sink/stream
- `reqwest 0.12` (rustls-tls, json, no default-features) - HTTP client for Listmonk integration
- `json-ptr 3.1` (frontend) - JSON Pointer (`/path/to/value`) navigation for data store
- `ajv 8.18` + `ajv-formats 3.0` (frontend devDep) - JSON Schema validation
- `js-yaml 4.1` (frontend devDep) - YAML parsing

**Infrastructure:**
- `tracing 0.1` + `tracing-subscriber 0.3` (env-filter) - Structured logging
- `time 0.3` - Cookie `max_age` (axum-extra cookie builder requires this type)
- `tokio-tungstenite 0.26` (devDep) - WebSocket client used in integration tests

## Workspace Structure (Rust)

Four crates in `backend/crates/`:
- `marionette-protocol` - Protocol message types (no async, no axum; pure serde structs)
- `marionette-macros` - Proc-macro crate; generates handler boilerplate
- `marionette` - Core library: WebSocket runtime, session management, ORM helpers, builders
- `crm-demo` - Demo application binary; depends on all three crates above

Workspace resolver: `"3"` (Cargo 2024 edition)

## Configuration

**Environment (backend):**
- `LISTMONK_URL` - Base URL for Listmonk instance (optional; disables sync features when absent)
- `LISTMONK_USER` - Listmonk basic auth username
- `LISTMONK_PASSWORD` - Listmonk basic auth password
- No `.env` files committed; variables set externally

**Logging:**
- Controlled via `RUST_LOG` environment variable (tracing-subscriber env-filter)

**Build:**
- Backend: `backend/rustfmt.toml` — `edition = "2024"`
- Frontend: `frontend/tsconfig.json` — strict mode, bundler resolution
- Frontend: `frontend/svelte.config.js` — static adapter, SPA fallback on `index.html`
- Spec: `spec/.redocly.yaml` (exists)

## Platform Requirements

**Development:**
- Rust 1.93+ with Cargo
- Node.js 25+, npm
- `mise` recommended for reproducible tool versions

**Production:**
- Single binary: `target/release/crm-demo`
- Serves frontend static files from `../frontend/build/` relative to working directory
- SQLite database file `crm.db` in working directory
- Listens on port 3001

---

*Stack analysis: 2026-04-08*
