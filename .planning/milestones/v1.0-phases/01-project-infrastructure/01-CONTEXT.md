# Phase 1: Project Infrastructure - Context

**Gathered:** 2026-03-17
**Status:** Ready for planning

<domain>
## Phase Boundary

Set up the complete development environment: Makefile with standard targets, project directory structure for frontend + backend + spec, GitHub Actions CI/CD, and code formatting/linting configuration. No application code — just the skeleton and tooling.

</domain>

<decisions>
## Implementation Decisions

### Directory structure
- Three top-level directories: `frontend/`, `backend/`, `spec/`
- `backend/` contains a Cargo workspace with `backend/Cargo.toml` as workspace root
- Cargo workspace has `backend/crates/` with sub-crates: `marionette-protocol`, `marionette-macros`, `marionette`, `crm-demo`
- `frontend/` is a Svelte library package (publishable as @marionette/svelte or similar)
- Frontend uses SvelteKit with `src/lib/` for library components and `src/routes/` for CRM demo app
- `spec/` holds OpenAPI YAML and JSON schemas

### Dev server orchestration
- `make dev` uses simple background processes with trap cleanup (no external process manager)
- Vite dev server proxies `/api/*` to backend on localhost:3001
- WebSocket connections proxied too (`/ws` -> `ws://localhost:3001`)
- Single origin in dev matches production behavior (Axum serves everything)

### CI/CD workflow
- Single workflow file `.github/workflows/ci.yml`
- Parallel jobs: `frontend` (lint + test) and `backend` (fmt + clippy + test + build)
- Triggers on both push to main and pull requests
- Cache both Cargo registry/target and npm node_modules

### Linting & formatting
- **Frontend:** ESLint + eslint-plugin-svelte for linting, Prettier + prettier-plugin-svelte for formatting
- **Backend:** rustfmt for formatting, clippy with pedantic level (`#![warn(clippy::pedantic)]`, `-D warnings` in CI)
- No pre-commit hooks — CI enforces formatting and lint rules
- `make lint` runs both frontend and backend linters
- `make format` runs both formatters (convenience, not required before commit)

### Claude's Discretion
- Exact port numbers for dev servers (suggested: Vite :5173, Axum :3001)
- SvelteKit adapter choice (static vs node)
- Specific clippy allows beyond `module_name_repetitions`
- Cargo workspace resolver version
- GitHub Actions runner OS and tool version pinning strategy

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project definition
- `CONCEPT.md` — Full protocol vision, three primitives, component catalog examples, implementation plan outline
- `TOOLING.md` — Tech stack decisions, SQL conventions, build/dev/test approach
- `.planning/PROJECT.md` — Project scope, constraints, key decisions, design principles
- `.planning/REQUIREMENTS.md` — INFRA-01 through INFRA-05 requirements for this phase

### Toolchain
- `mise.toml` — Existing toolchain config (node latest, rust latest, rust-analyzer latest)
- `.mcp.json` — MCP server config (Svelte, Flowbite Svelte)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `mise.toml` already configures node + rust + rust-analyzer toolchains — build on this, don't replace it

### Established Patterns
- No code exists yet — this phase establishes all patterns
- TOOLING.md defines the conventions to follow (Makefile targets, testing frameworks, SQL conventions)

### Integration Points
- `mise.toml` at repo root — toolchain management
- `.mcp.json` at repo root — MCP servers for Svelte and Flowbite (development tooling, not part of build)

</code_context>

<specifics>
## Specific Ideas

- Makefile targets match TOOLING.md exactly: dev, build, test, lint, clean
- Cargo workspace inside backend/ keeps Rust self-contained while frontend/ stays independent
- Frontend as publishable Svelte library enables reuse beyond the CRM demo

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 01-project-infrastructure*
*Context gathered: 2026-03-17*
