# External Integrations

**Analysis Date:** 2026-04-08

## APIs & External Services

**Newsletter / Mailing List:**
- Listmonk (self-hosted open-source newsletter tool)
  - Client: `backend/crates/crm-demo/src/listmonk.rs` — `ListmonkClient` struct wrapping `reqwest::Client`
  - Auth: HTTP Basic Auth
  - Env vars: `LISTMONK_URL`, `LISTMONK_USER`, `LISTMONK_PASSWORD`
  - Optional: entire integration is disabled gracefully when `LISTMONK_URL` is not set
  - Operations implemented:
    - `validate_connection` — GET `/api/lists` health check on startup
    - `find_subscriber_by_email` — GET `/api/subscribers` with query filter
    - `create_subscriber` — POST `/api/subscribers`
    - `update_subscriber` — PUT `/api/subscribers/{id}`
    - `blocklist_subscriber` — PUT `/api/subscribers/{id}/blocklist`
    - `get_or_create_list` — GET + POST `/api/lists`
    - `set_subscriber_lists` — PUT `/api/subscribers/lists`
    - `get_subscriber_export` — GET `/api/subscribers/{id}/export`
  - Handler actions registered on `ActionRouter`:
    - `listmonk_sync` — sync single contact
    - `listmonk_sync_all` — sync all contacts
    - `listmonk_history_refresh` — refresh campaign history for a contact

## Data Storage

**Databases:**
- SQLite (embedded, via SeaORM + sqlx-sqlite)
  - Connection string: `sqlite://crm.db?mode=rwc` (relative to binary working directory)
  - Client: `sea-orm 1.1` with `sea-orm-migration 1.1`
  - Migration runner: `backend/crates/crm-demo/src/migration/mod.rs` — `Migrator` with 10 migrations
  - Schema includes: users, audit_log, companies, contacts, notes, tags, contact_tags, interactions, listmonk_sync, listmonk_cache
  - Migrations run automatically at startup via `Migrator::up(&db, None)`

**File Storage:**
- Not applicable; no file upload or cloud storage

**Caching:**
- `listmonk_cache` table in SQLite (migration 10) — stores campaign history locally to reduce Listmonk API calls

## Authentication & Identity

**Auth Provider:**
- Custom implementation; no third-party identity provider

**HTTP Auth flow:**
- Endpoint: POST `/api/login` — `backend/crates/crm-demo/src/handlers/auth.rs`
- Validates email + bcrypt password hash
- Creates session row in SQLite `session` table with 24h expiry
- Returns HTTP-only `marionette_session` cookie (`SameSite=Lax`, `path=/`, `max_age=24h`)
- Endpoint: POST `/api/logout` — deletes session row, removes cookie

**WebSocket Auth flow:**
- Cookie is extracted at HTTP upgrade time in `backend/crates/marionette/src/ws.rs`
- Session token looked up in DB; expiry checked; user_id and roles loaded into `WsSession`
- Unauthenticated WS connections receive a login form as the initial render message
- After successful `login` action over WS, `WsSession` is updated in-memory for subsequent actions

**Authorization:**
- Role-based via `AuthRequirement` enum in `marionette-protocol`:
  - `AuthRequirement::None` — public (login action)
  - `AuthRequirement::Authenticated` — any logged-in user
  - `AuthRequirement::Role("admin")` — admin-only actions (user management, audit log)

**Password Hashing:**
- `bcrypt 0.19`; verification is CPU-bound and runs in `tokio::task::spawn_blocking`

## Monitoring & Observability

**Error Tracking:**
- None; no external error tracking service integrated

**Logs:**
- `tracing 0.1` + `tracing-subscriber 0.3` with `env-filter` feature
- Initialized via `tracing_subscriber::fmt::init()` at startup
- Level controlled by `RUST_LOG` environment variable
- Structured fields on log statements (session_id, user_id, action, error)

## CI/CD & Deployment

**Hosting:**
- Not configured; no CI pipeline or deployment manifests present

**CI Pipeline:**
- None detected (no `.github/`, `.gitlab-ci.yml`, etc.)

**Build commands (manual):**
- `make build` — `cd frontend && npm run build` then `cd backend && cargo build --release`
- `make test` — `cargo test` + `npm test -- --run`
- `make lint` — cargo fmt check, cargo clippy, eslint, svelte-check, redocly spec lint
- `make e2e` — Playwright E2E suite in `frontend/tests/e2e/`

## WebSocket Transport

**Protocol:**
- Custom JSON-over-WebSocket protocol (OpenSDUI/Marionette protocol)
- Endpoint: `/ws` (proxied by Vite dev server from port 5173 → 3001)
- Frontend client: `frontend/src/lib/transport/websocket.svelte.ts`
- Message dispatcher: `frontend/src/lib/transport/dispatcher.ts`
- Protocol spec: `spec/openapi.yaml` + `spec/PROTOCOL.md`

**Message types (server → client):**
- `hello` — connection established
- `render` — full surface tree replacement
- `patch` — JSON Patch operations on surface data
- `event` — named event (event bus TBD)
- `error` — validation/action errors with optional correlation ID

**Message types (client → server):**
- `action` — named action with optional payload (`ActionMessage`)

## HTTP Endpoints

**REST-style (non-WS):**
- `GET /api/health` — returns `"ok"` plain text
- `POST /api/login` — JSON body `{username, password}`, sets session cookie
- `POST /api/logout` — clears session cookie and deletes session row

**Static Files:**
- All other paths → `ServeDir` serving `../frontend/build/` with SPA fallback to `index.html`

## Environment Configuration

**Required env vars (for full functionality):**
- `LISTMONK_URL` — Listmonk base URL (e.g. `https://listmonk.example.com`)
- `LISTMONK_USER` — Listmonk API username
- `LISTMONK_PASSWORD` — Listmonk API password

**Optional env vars:**
- `RUST_LOG` — log level filter (e.g. `debug`, `info`, `crm_demo=debug`)

**Secrets location:**
- No `.env` files present; environment variables expected to be injected externally at runtime

## Webhooks & Callbacks

**Incoming:**
- None; no webhook endpoints registered

**Outgoing:**
- None; integration with Listmonk is polling/push-on-demand only (no webhooks sent)

---

*Integration audit: 2026-04-08*
