# Phase 1: Project Infrastructure - Research

**Researched:** 2026-03-18
**Domain:** Build system, CI/CD, project scaffolding (Rust + SvelteKit)
**Confidence:** HIGH

## Summary

Phase 1 establishes the development skeleton: directory structure, Makefile with standard targets, dev server orchestration, CI/CD pipeline, and code quality tooling. No application code is written -- only the infrastructure that all future phases build on.

The project uses a Cargo workspace inside `backend/` with four crates, a SvelteKit library project in `frontend/` (dual-purpose: publishable component library via `src/lib/` and CRM demo app via `src/routes/`), and an OpenAPI spec directory at `spec/`. Toolchain is already pinned via `mise.toml` (Node latest = v25.4.0, Rust latest = 1.93.1). Tailwind CSS v4 uses the Vite plugin approach (no PostCSS config needed). Flowbite Svelte v1.31.0 supports Svelte 5 + Tailwind v4 natively.

**Primary recommendation:** Scaffold all directories and config files first, then wire up the Makefile targets, then CI. Keep everything minimal but functional -- empty test suites that pass, placeholder routes, stub crate `lib.rs` files.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Three top-level directories: `frontend/`, `backend/`, `spec/`
- `backend/` contains a Cargo workspace with `backend/Cargo.toml` as workspace root
- Cargo workspace has `backend/crates/` with sub-crates: `marionette-protocol`, `marionette-macros`, `marionette`, `crm-demo`
- `frontend/` is a Svelte library package (publishable as @marionette/svelte or similar)
- Frontend uses SvelteKit with `src/lib/` for library components and `src/routes/` for CRM demo app
- `spec/` holds OpenAPI YAML and JSON schemas
- `make dev` uses simple background processes with trap cleanup (no external process manager)
- Vite dev server proxies `/api/*` to backend on localhost:3001
- WebSocket connections proxied too (`/ws` -> `ws://localhost:3001`)
- Single CI workflow `.github/workflows/ci.yml`
- Parallel jobs: `frontend` (lint + test) and `backend` (fmt + clippy + test + build)
- Triggers on push to main and pull requests
- Cache both Cargo registry/target and npm node_modules
- Frontend: ESLint + eslint-plugin-svelte for linting, Prettier + prettier-plugin-svelte for formatting
- Backend: rustfmt for formatting, clippy with pedantic level (`#![warn(clippy::pedantic)]`, `-D warnings` in CI)
- No pre-commit hooks -- CI enforces formatting and lint rules
- `make lint` runs both frontend and backend linters
- `make format` runs both formatters

### Claude's Discretion
- Exact port numbers for dev servers (suggested: Vite :5173, Axum :3001)
- SvelteKit adapter choice (static vs node)
- Specific clippy allows beyond `module_name_repetitions`
- Cargo workspace resolver version
- GitHub Actions runner OS and tool version pinning strategy

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| INFRA-01 | Makefile with standard targets (dev, build, test, lint, clean) | Makefile patterns section with exact target definitions; dev server orchestration pattern |
| INFRA-02 | Project directory structure (frontend/, backend/, spec/) | Full directory tree documented; Cargo workspace config; SvelteKit project structure |
| INFRA-03 | GitHub Actions workflow for CI (test, lint, build on PR) | CI workflow pattern with parallel jobs, caching strategy, exact action versions |
| INFRA-04 | Development server configuration (frontend + backend concurrent) | Vite proxy config; background process pattern with trap; port assignments |
| INFRA-05 | Code formatting and linting configuration (Rust + Svelte) | ESLint flat config for Svelte; Prettier config; rustfmt.toml; clippy pedantic setup |
</phase_requirements>

## Standard Stack

### Core (verified against npm/cargo registries 2026-03-18)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| SvelteKit | 2.55.0 | Frontend framework + dev server | Official Svelte app framework |
| Svelte | 5.53.13 | UI component framework | Locked by project |
| Vite | 8.0.0 | Frontend bundler + dev server | Bundled with SvelteKit |
| Tailwind CSS | 4.2.1 | Utility-first CSS | Locked by project (Flowbite dependency) |
| @tailwindcss/vite | 4.2.1 | Tailwind v4 Vite integration | Official v4 approach, no PostCSS needed |
| Flowbite Svelte | 1.31.0 | UI component library | Locked by project |
| Flowbite Svelte Icons | 3.1.0 | Icon library | Companion to Flowbite Svelte |
| ESLint | 10.0.3 | JS/Svelte linting | Flat config only (v10+) |
| @eslint/js | 10.0.1 | ESLint recommended rules | Core rule set for flat config |
| eslint-plugin-svelte | 3.15.2 | Svelte-specific lint rules | Official Svelte ESLint plugin |
| typescript-eslint | 8.57.1 | TypeScript ESLint integration | Type-aware linting |
| Prettier | 3.8.1 | Code formatter | Locked by project |
| prettier-plugin-svelte | 3.5.1 | Svelte formatting | Locked by project |
| TypeScript | 5.9.3 | Type checking | SvelteKit uses TS by default |
| svelte-check | 4.4.5 | Svelte type checker | CLI for type checking Svelte files |
| Vitest | 4.1.0 | Test runner | Locked by project (TOOLING.md) |
| globals | 17.4.0 | ESLint globals definitions | Required by ESLint flat config |

### Backend (Rust -- versions managed by Cargo)

| Crate | Purpose | Why |
|-------|---------|-----|
| axum | Web framework | Locked by project |
| tokio | Async runtime | Required by axum |
| serde / serde_json | Serialization | Universal Rust JSON handling |
| tower-http | Static file serving, CORS | Axum middleware ecosystem |
| tracing | Logging | Locked by project |

### CI/CD Actions

| Action | Version | Purpose |
|--------|---------|---------|
| actions/checkout | v4 | Repository checkout |
| actions/setup-node | v4 | Node.js setup |
| dtolnay/rust-toolchain | stable | Rust toolchain (uses rustup) |
| Swatinem/rust-cache | v2 | Smart Cargo caching |
| actions/cache | v4 | npm cache |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| @sveltejs/adapter-static | @sveltejs/adapter-node | Static is simpler since Axum serves everything; node adapter only needed if SvelteKit handles SSR itself. **Recommend adapter-static.** |
| Cargo resolver 3 | Cargo resolver 2 | Resolver 3 is default with edition 2024; use edition 2024 + resolver 3 for the workspace since Rust 1.93 supports it. **Recommend edition 2024 (implies resolver 3).** |

**Installation (frontend):**
```bash
cd frontend
npm create svelte@latest . -- --template skeleton --types ts
npm install -D @tailwindcss/vite flowbite-svelte flowbite-svelte-icons
npm install -D eslint @eslint/js eslint-plugin-svelte typescript-eslint globals
npm install -D prettier prettier-plugin-svelte
npm install -D svelte-check typescript vitest
```

## Architecture Patterns

### Recommended Project Structure

```
marionette/
├── Makefile                    # Top-level orchestrator
├── mise.toml                   # Toolchain (existing)
├── .mcp.json                   # MCP servers (existing)
├── .github/
│   └── workflows/
│       └── ci.yml              # Single CI workflow
├── frontend/
│   ├── package.json
│   ├── svelte.config.js
│   ├── vite.config.ts
│   ├── tsconfig.json
│   ├── eslint.config.js
│   ├── .prettierrc
│   ├── .prettierignore
│   ├── src/
│   │   ├── app.html
│   │   ├── app.css             # Tailwind v4 imports
│   │   ├── lib/
│   │   │   └── index.ts        # Library entry point
│   │   └── routes/
│   │       ├── +layout.svelte  # Root layout (imports app.css)
│   │       └── +page.svelte    # CRM demo home
│   ├── static/
│   └── tests/                  # Vitest test files
├── backend/
│   ├── Cargo.toml              # Workspace root
│   ├── rustfmt.toml
│   └── crates/
│       ├── marionette-protocol/
│       │   ├── Cargo.toml
│       │   └── src/lib.rs
│       ├── marionette-macros/
│       │   ├── Cargo.toml
│       │   └── src/lib.rs
│       ├── marionette/
│       │   ├── Cargo.toml
│       │   └── src/lib.rs
│       └── crm-demo/
│           ├── Cargo.toml
│           └── src/main.rs     # Binary crate
└── spec/
    └── .gitkeep                # Placeholder for Phase 2
```

### Pattern 1: Cargo Virtual Workspace

**What:** `backend/Cargo.toml` is a virtual workspace (no `[package]` section, only `[workspace]`).
**When to use:** When there is no single "root" package -- workspace is purely an organizer.
**Example:**

```toml
# backend/Cargo.toml
[workspace]
members = [
    "crates/marionette-protocol",
    "crates/marionette-macros",
    "crates/marionette",
    "crates/crm-demo",
]

[workspace.package]
edition = "2024"
license = "MIT"
repository = "https://github.com/oetiker/marionette"

[workspace.dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
axum = "0.8"
```

Note: With edition 2024, resolver 3 is implied automatically. Virtual workspaces with edition 2024 in `[workspace.package]` get resolver 3 by default.

### Pattern 2: Makefile with Background Process Orchestration

**What:** `make dev` starts both servers as background processes, traps signals for cleanup.
**When to use:** Dev server orchestration without external tools.
**Example:**

```makefile
.PHONY: dev build test lint clean format

dev:
	@echo "Starting development servers..."
	@trap 'kill 0' EXIT; \
	cd backend && cargo run -p crm-demo &\
	cd frontend && npm run dev -- --host &\
	wait

build:
	cd backend && cargo build --release
	cd frontend && npm run build

test:
	cd backend && cargo test
	cd frontend && npm test

lint:
	cd backend && cargo fmt --check
	cd backend && cargo clippy -- -D warnings
	cd frontend && npm run lint
	cd frontend && npm run check

format:
	cd backend && cargo fmt
	cd frontend && npm run format

clean:
	cd backend && cargo clean
	cd frontend && rm -rf .svelte-kit build node_modules/.vite
```

### Pattern 3: Vite Proxy Configuration

**What:** Vite proxies API and WebSocket requests to the backend dev server.
**When to use:** Development mode -- single origin matches production behavior.
**Example:**

```typescript
// frontend/vite.config.ts
import { sveltekit } from '@sveltejs/kit/vite';
import tailwindcss from '@tailwindcss/vite';
import { defineConfig } from 'vite';

export default defineConfig({
    plugins: [tailwindcss(), sveltekit()],
    server: {
        proxy: {
            '/api': {
                target: 'http://localhost:3001',
                changeOrigin: true
            },
            '/ws': {
                target: 'ws://localhost:3001',
                ws: true
            }
        }
    }
});
```

### Pattern 4: ESLint Flat Config for SvelteKit + TypeScript

**What:** ESLint 10 flat config with Svelte and TypeScript support.
**Example:**

```javascript
// frontend/eslint.config.js
import js from '@eslint/js';
import svelte from 'eslint-plugin-svelte';
import ts from 'typescript-eslint';
import globals from 'globals';
import svelteConfig from './svelte.config.js';

export default ts.config(
    js.configs.recommended,
    ...ts.configs.recommended,
    ...svelte.configs['flat/recommended'],
    {
        languageOptions: {
            globals: {
                ...globals.browser,
                ...globals.node
            }
        }
    },
    {
        files: ['**/*.svelte', '**/*.svelte.ts', '**/*.svelte.js'],
        languageOptions: {
            parserOptions: {
                parser: ts.parser,
                svelteConfig
            }
        }
    },
    {
        ignores: ['build/', '.svelte-kit/', 'dist/']
    }
);
```

### Pattern 5: Tailwind v4 CSS Setup for Flowbite Svelte

**What:** Tailwind v4 with Flowbite plugin, no postcss.config needed.
**Example:**

```css
/* frontend/src/app.css */
@import "tailwindcss";
@plugin "flowbite/plugin";
@custom-variant dark (&:where(.dark, .dark *));
@source "../node_modules/flowbite-svelte/dist";
@source "../node_modules/flowbite-svelte-icons/dist";
```

### Anti-Patterns to Avoid

- **Nested Cargo.toml in repo root:** Keep Cargo workspace inside `backend/` -- do not put it at repo root or Rust tools will try to treat the whole repo as a Rust project.
- **PostCSS config for Tailwind v4:** The `@tailwindcss/vite` plugin replaces PostCSS-based setup entirely. Do not create `postcss.config.js`.
- **ESLint legacy config (.eslintrc):** ESLint 10 only supports flat config (`eslint.config.js`). Do not create `.eslintrc.*` files.
- **`npm run dev` without `--host`:** SvelteKit defaults to localhost only; `--host` is needed if you want to access from other machines, but for local dev the default is fine. Omit `--host` unless needed.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Cargo caching in CI | Custom cache keys | Swatinem/rust-cache@v2 | Handles registry, target dir, Cargo.lock changes automatically |
| Svelte type checking | Manual tsc invocations | `svelte-check` CLI | Understands .svelte files, integrates with SvelteKit |
| Tailwind v4 integration | PostCSS pipeline | @tailwindcss/vite plugin | Official approach for v4, zero config |
| Dev server orchestration | Process manager (pm2, concurrently) | Makefile trap pattern | User decision: simple background processes |

## Common Pitfalls

### Pitfall 1: Virtual Workspace Edition Inheritance
**What goes wrong:** Crates inside a virtual workspace do not automatically inherit the workspace edition unless they declare `edition.workspace = true` in their own `Cargo.toml`.
**Why it happens:** Virtual workspaces define `[workspace.package]` but each crate must opt in.
**How to avoid:** Every crate's `Cargo.toml` must include `edition.workspace = true` under `[package]`.
**Warning signs:** Compilation errors about unstable features or unexpected syntax behavior.

### Pitfall 2: Vite Proxy Only Works in Dev Mode
**What goes wrong:** Proxy config in `vite.config.ts` has no effect in production builds.
**Why it happens:** Vite proxy is a dev server feature only.
**How to avoid:** This is expected. In production, Axum serves the built frontend as static files and handles API routes directly. No proxy needed.
**Warning signs:** None -- just be aware when writing documentation.

### Pitfall 3: Flowbite Svelte @source Path Sensitivity
**What goes wrong:** Tailwind does not detect Flowbite utility classes, components render unstyled.
**Why it happens:** The `@source` directive path in `app.css` must be relative to the CSS file location, not the project root.
**How to avoid:** Use `../node_modules/flowbite-svelte/dist` (relative from `src/app.css`).
**Warning signs:** Flowbite components appear but lack styling.

### Pitfall 4: Clippy Pedantic Noise
**What goes wrong:** `clippy::pedantic` produces many warnings that are not useful (e.g., `must_use_candidate`, `missing_errors_doc` on internal code).
**Why it happens:** Pedantic level is intentionally aggressive.
**How to avoid:** Allow specific noisy lints at the crate level: `#![allow(clippy::module_name_repetitions)]` (user-requested). Consider also allowing `clippy::must_use_candidate` and `clippy::missing_errors_doc` for library crates during early development.
**Warning signs:** CI fails on hundreds of pedantic warnings.

### Pitfall 5: SvelteKit Library + App Dual Mode
**What goes wrong:** Confusion about what goes in `src/lib/` vs `src/routes/`.
**Why it happens:** SvelteKit supports both library packaging (`svelte-package` from `src/lib/`) and app building (from `src/routes/`).
**How to avoid:** Strict separation: `src/lib/` contains only the reusable Marionette component library. `src/routes/` contains only the CRM demo app that consumes the library via `$lib` imports.
**Warning signs:** App-specific code leaking into `src/lib/`.

### Pitfall 6: Background Process Cleanup in Makefile
**What goes wrong:** Orphan processes remain after Ctrl+C.
**Why it happens:** `trap 'kill 0' EXIT` must be in the same shell as the background processes.
**How to avoid:** Use a single shell command with `; \` continuation. The trap, background launches, and `wait` must all be in one recipe line (or use `.ONESHELL`).
**Warning signs:** Port already in use errors on subsequent `make dev` runs.

## Code Examples

### SvelteKit svelte.config.js with Static Adapter

```javascript
// frontend/svelte.config.js
import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

export default {
    preprocess: vitePreprocess(),
    kit: {
        adapter: adapter({
            pages: 'build',
            assets: 'build',
            fallback: 'index.html',  // SPA fallback for client-side routing
            strict: false
        })
    }
};
```

Note: `fallback: 'index.html'` is critical -- it enables SPA-style routing where Axum serves `index.html` for all non-API routes.

### Rustfmt Configuration

```toml
# backend/rustfmt.toml
edition = "2024"
```

Keep it minimal. Rustfmt defaults are the community standard. Only override if there is a specific reason.

### Stub Crate with Workspace Inheritance

```toml
# backend/crates/marionette-protocol/Cargo.toml
[package]
name = "marionette-protocol"
version = "0.1.0"
edition.workspace = true
license.workspace = true

[dependencies]
serde.workspace = true
serde_json.workspace = true
```

```rust
// backend/crates/marionette-protocol/src/lib.rs
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
```

### Prettier Configuration

```json
// frontend/.prettierrc
{
    "useTabs": true,
    "singleQuote": true,
    "trailingComma": "none",
    "printWidth": 100,
    "plugins": ["prettier-plugin-svelte"],
    "overrides": [
        {
            "files": "*.svelte",
            "options": {
                "parser": "svelte"
            }
        }
    ]
}
```

### GitHub Actions CI Workflow

```yaml
# .github/workflows/ci.yml
name: CI

on:
  push:
    branches: [main]
  pull_request:

jobs:
  frontend:
    runs-on: ubuntu-latest
    defaults:
      run:
        working-directory: frontend
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: 'lts/*'
          cache: 'npm'
          cache-dependency-path: frontend/package-lock.json
      - run: npm ci
      - run: npm run lint
      - run: npm run check
      - run: npm test -- --run
      - run: npm run build

  backend:
    runs-on: ubuntu-latest
    defaults:
      run:
        working-directory: backend
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      - uses: Swatinem/rust-cache@v2
        with:
          workspaces: "backend -> target"
      - run: cargo fmt --check
      - run: cargo clippy -- -D warnings
      - run: cargo test
      - run: cargo build --release
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Tailwind v3 + PostCSS | Tailwind v4 + @tailwindcss/vite | Jan 2025 | No postcss.config.js needed; use @plugin directive instead of tailwind.config.js |
| ESLint .eslintrc (legacy config) | ESLint flat config (eslint.config.js) | ESLint 9+ (2024), v10 flat-only | Must use eslint.config.js, no .eslintrc support in v10 |
| Cargo resolver 2 | Cargo resolver 3 | Rust 2024 edition | Default with edition 2024; better feature unification |
| flowbite-svelte 0.x (Svelte 4) | flowbite-svelte 1.x (Svelte 5) | Late 2024 | Complete rewrite for Svelte 5 runes; Tailwind v4 support |

**Deprecated/outdated:**
- `postcss.config.js` + `tailwind.config.js`: Replaced by `@tailwindcss/vite` plugin + CSS directives
- `@sveltejs/adapter-auto`: Still works but for this project, explicit `adapter-static` is correct since Axum serves the built files
- `.eslintrc.*` files: Not supported in ESLint 10

## Open Questions

1. **SvelteKit adapter-static vs adapter-node**
   - What we know: Axum serves the built frontend as static files in production. SvelteKit does not need its own server.
   - Recommendation: Use `adapter-static` with `fallback: 'index.html'` for SPA behavior. This is the simplest approach since all server logic lives in Axum.

2. **Clippy pedantic allows**
   - What we know: `module_name_repetitions` is user-requested. Pedantic level will produce many other warnings.
   - Recommendation: Start with only `module_name_repetitions` allowed. Add more allows as they surface during development. Do not pre-emptively suppress warnings.

3. **Cargo workspace edition**
   - What we know: Rust 1.93 supports edition 2024. Resolver 3 is implied.
   - Recommendation: Use edition 2024 for all crates via `[workspace.package]`. This is the current standard.

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework (frontend) | Vitest 4.1.0 |
| Framework (backend) | cargo test (built-in) |
| Config file (frontend) | `frontend/vitest.config.ts` or inline in `vite.config.ts` |
| Config file (backend) | None needed (Cargo default) |
| Quick run command | `cd frontend && npx vitest --run && cd ../backend && cargo test` |
| Full suite command | `make test` |

### Phase Requirements to Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| INFRA-01 | Makefile targets work | smoke | `make build && make test && make lint` | Wave 0 |
| INFRA-02 | Directory structure correct | smoke | `test -d frontend/src/lib && test -d backend/crates/marionette-protocol` | Wave 0 |
| INFRA-03 | CI workflow valid | manual-only | Push to branch, observe GitHub Actions | Manual |
| INFRA-04 | Dev servers start and proxy works | manual-only | `make dev` + manual browser check | Manual |
| INFRA-05 | Linting passes on clean code | smoke | `make lint` | Wave 0 |

### Sampling Rate
- **Per task commit:** `make lint && make test`
- **Per wave merge:** `make build && make test && make lint`
- **Phase gate:** All Makefile targets succeed, CI workflow passes on PR

### Wave 0 Gaps
- `frontend/` directory -- entire SvelteKit project needs scaffolding
- `backend/` directory -- Cargo workspace and all crates need scaffolding
- `spec/` directory -- placeholder needed
- `.github/workflows/ci.yml` -- needs creation
- `Makefile` -- needs creation

## Sources

### Primary (HIGH confidence)
- npm registry -- verified all package versions via `npm view` on 2026-03-18
- `rustc --version` -- confirmed Rust 1.93.1 (edition 2024 support)
- [Cargo Workspaces docs](https://doc.rust-lang.org/cargo/reference/workspaces.html) -- virtual workspace setup
- [Cargo Rust 2024 resolver](https://doc.rust-lang.org/edition-guide/rust-2024/cargo-resolver.html) -- resolver 3 default

### Secondary (MEDIUM confidence)
- [Tailwind CSS SvelteKit guide](https://tailwindcss.com/docs/guides/sveltekit) -- v4 Vite plugin setup verified against search results
- [Flowbite Svelte quickstart](https://flowbite-svelte.com/docs/pages/quickstart) -- @source directive paths
- [SvelteKit packaging docs](https://svelte.dev/docs/kit/packaging) -- src/lib library mode
- [eslint-plugin-svelte user guide](https://sveltejs.github.io/eslint-plugin-svelte/user-guide/) -- flat config setup
- [Swatinem/rust-cache](https://github.com/Swatinem/rust-cache) -- v2.8.1, workspace support

### Tertiary (LOW confidence)
- None -- all findings verified against official sources

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- all versions verified against registries
- Architecture: HIGH -- patterns follow official documentation
- Pitfalls: HIGH -- based on well-known ecosystem issues

**Research date:** 2026-03-18
**Valid until:** 2026-04-18 (stable ecosystem, 30-day window)
