# Stack Research

**Domain:** Server-Driven UI Protocol + CRM Reference Implementation
**Researched:** 2026-01-23
**Confidence:** HIGH

## Executive Summary

This stack research covers the technology choices for OpenSDUI (an open SDUI protocol specification) and Marionette (its reference implementation). The stack aligns with the project's TOOLING.md but adds precision on versions, rationale, and alternatives.

The recommended stack is mature, well-integrated, and production-ready:
- **Backend:** Rust + Axum 0.8 + SeaORM 1.1 + utoipa 5.4
- **Frontend:** Svelte 5.48 + Flowbite Svelte 1.31 + Tailwind CSS 4.0
- **Protocol:** OpenAPI 3.1 specification with JSON Schema 2020-12

---

## Recommended Stack

### Backend Core

| Technology | Version | Purpose | Why Recommended | Confidence |
|------------|---------|---------|-----------------|------------|
| **Rust** | 1.85+ (2024 edition) | Language | Memory safety without GC, excellent async, strong type system catches errors at compile time. Ideal for protocol implementations where correctness matters. | HIGH |
| **tokio** | 1.49.0 | Async runtime | De facto standard for async Rust. Powers axum, sea-orm, and the entire Tokio ecosystem. LTS releases ensure stability. | HIGH |
| **axum** | 0.8.8 | Web framework | Built by Tokio team, deep Tower/hyper integration, ergonomic extractors, excellent middleware ecosystem. New 0.8 syntax (`/{param}`) is cleaner. No `#[async_trait]` needed. | HIGH |
| **SeaORM** | 1.1.19 | Database ORM | Async-first ORM built on SQLx. Query builder DSL, entity patterns, migrations. Supports PostgreSQL and SQLite. 2.0 is in RC but 1.1 is stable. | HIGH |
| **utoipa** | 5.4.0 | OpenAPI generation | Code-first OpenAPI 3.1 generation from Rust types. Integrates with axum via `utoipa-axum`. Supports Swagger UI, Redoc, RapiDoc. | HIGH |

### Backend Supporting Libraries

| Library | Version | Purpose | When to Use | Confidence |
|---------|---------|---------|-------------|------------|
| **serde** | 1.0.228 | Serialization | Always. De facto standard for Rust serialization. Use with `derive` feature. | HIGH |
| **serde_json** | 1.0.149 | JSON handling | Always. Fast (500-1000 MB/s), battle-tested. Protocol messages are JSON. | HIGH |
| **tower-http** | 0.6.8 | HTTP middleware | CORS, compression, static file serving, request tracing. Essential for web APIs. | HIGH |
| **tracing** | 0.1.44 | Observability | Structured logging and spans. Integrates with OpenTelemetry for production observability. | HIGH |
| **validator** | 0.20.0 | Validation | Request validation with declarative rules (email, length, regex). Use with axum extractors. | HIGH |
| **reqwest** | 0.13.1 | HTTP client | External API calls (Listmonk integration). Async, supports JSON, connection pooling. | HIGH |
| **tokio-tungstenite** | 0.28.0 | WebSocket | Real-time SDUI updates. Async WebSocket for Tokio. Production-ready as of 0.28. | HIGH |
| **thiserror** | 2.x | Error types | Derive Error trait for custom error types. Clean error handling. | HIGH |
| **anyhow** | 1.x | Error handling | Application-level error handling. Use in main/handlers, not libraries. | HIGH |
| **dotenvy** | 0.15.x | Environment | Load `.env` files. Successor to dotenv crate. | MEDIUM |
| **chrono** | 0.4.x | Date/time | DateTime handling with timezone support. Consider `time` crate as alternative. | MEDIUM |
| **uuid** | 1.x | UUIDs | Generate and parse UUIDs. Use v4 for random IDs. | HIGH |

### Frontend Core

| Technology | Version | Purpose | Why Recommended | Confidence |
|------------|---------|---------|-----------------|------------|
| **Svelte** | 5.48.0 | UI framework | Runes system (`$state`, `$derived`, `$effect`) provides explicit, predictable reactivity. Deep reactivity eliminates manual reassignment. Compiles to minimal JS. | HIGH |
| **SvelteKit** | 2.x | App framework | File-based routing, SSR/SSG/hybrid rendering, adapters for deployment. Built on Vite. | HIGH |
| **Flowbite Svelte** | 1.31.0 | Component library | 63+ accessible Tailwind components. Official Flowbite integration for Svelte. MIT licensed. | HIGH |
| **Tailwind CSS** | 4.0.x | Styling | CSS-first configuration, cascade layers, 5x faster builds. Modern CSS features (color-mix, @property). | HIGH |
| **Vite** | 6.x | Build tool | Fast HMR, ES modules, minimal config. Powers SvelteKit. | HIGH |

### Frontend Supporting Libraries

| Library | Version | Purpose | When to Use | Confidence |
|---------|---------|---------|-------------|------------|
| **@sveltejs/adapter-static** | 3.x | Static adapter | Deploy as static files served by Rust backend | HIGH |
| **openapi-fetch** | 0.x | API client | Type-safe fetch client generated from OpenAPI spec. Lighter than full codegen. | MEDIUM |
| **zod** | 3.x | Schema validation | Runtime validation for API responses. TypeScript-first. | MEDIUM |

### Development & Testing

| Tool | Version | Purpose | Notes | Confidence |
|------|---------|---------|-------|------------|
| **Vitest** | 4.0.x | Unit/component tests | Next-gen testing, Vite-native, Jest-compatible. Browser mode for component tests. | HIGH |
| **Playwright** | 1.57.x | E2E testing | Cross-browser testing, traces, visual snapshots. Now uses Chrome for Testing. | HIGH |
| **vitest-browser-svelte** | latest | Svelte component tests | Real browser testing (not jsdom). Use with Playwright provider. | HIGH |
| **cargo-nextest** | latest | Rust test runner | Faster than `cargo test`, better output, parallel execution. | MEDIUM |
| **mise** | latest | Tool version manager | Polyglot version manager (Rust, Node, etc.). Replaces asdf. | MEDIUM |

### Protocol Specification

| Technology | Version | Purpose | Why Recommended | Confidence |
|------------|---------|---------|-----------------|------------|
| **OpenAPI** | 3.1.0 | API specification | Full JSON Schema 2020-12 compatibility. Industry standard for REST APIs. | HIGH |
| **JSON Schema** | Draft 2020-12 | Data validation | Validates SDUI messages. OpenAPI 3.1 native support. | HIGH |
| **JSON Pointer** | RFC 6901 | Data binding | Standard path syntax for data binding in SDUI protocol. | HIGH |

---

## Installation

### Rust Backend (Cargo.toml)

```toml
[package]
name = "marionette"
version = "0.1.0"
edition = "2024"
rust-version = "1.85"

[dependencies]
# Web framework
axum = { version = "0.8", features = ["macros", "ws"] }
tokio = { version = "1.49", features = ["full"] }
tower = "0.5"
tower-http = { version = "0.6", features = ["cors", "compression-gzip", "fs", "trace"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Database
sea-orm = { version = "1.1", features = ["sqlx-sqlite", "sqlx-postgres", "runtime-tokio-rustls", "macros"] }
sea-orm-migration = "1.1"

# OpenAPI
utoipa = { version = "5.4", features = ["axum_extras"] }
utoipa-axum = "0.2"
utoipa-swagger-ui = { version = "9", features = ["axum"] }

# Validation
validator = { version = "0.20", features = ["derive"] }

# HTTP client (for Listmonk)
reqwest = { version = "0.13", features = ["json", "rustls-tls"] }

# WebSocket
tokio-tungstenite = "0.28"

# Observability
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }

# Utilities
thiserror = "2"
anyhow = "1"
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
dotenvy = "0.15"
```

### Frontend (package.json)

```json
{
  "devDependencies": {
    "@sveltejs/adapter-static": "^3.0.0",
    "@sveltejs/kit": "^2.0.0",
    "@sveltejs/vite-plugin-svelte": "^5.0.0",
    "svelte": "^5.48.0",
    "svelte-check": "^4.0.0",
    "tailwindcss": "^4.0.0",
    "typescript": "^5.7.0",
    "vite": "^6.0.0",
    "vitest": "^4.0.0",
    "@vitest/browser": "^4.0.0",
    "vitest-browser-svelte": "^0.1.0",
    "playwright": "^1.57.0"
  },
  "dependencies": {
    "flowbite-svelte": "^1.31.0",
    "flowbite-svelte-icons": "^2.0.0"
  }
}
```

---

## Alternatives Considered

### Backend Framework

| Recommended | Alternative | When to Use Alternative |
|-------------|-------------|-------------------------|
| **axum** | actix-web | If you need maximum raw throughput and are comfortable with actor model. Actix is slightly faster in benchmarks but axum's Tower ecosystem and ergonomics are better for most use cases. |
| **axum** | Rocket | If you prefer more "magic" (e.g., automatic form handling). Rocket 0.5 is async but has smaller middleware ecosystem. |
| **axum** | warp | If you want purely functional filter-based composition. Less ergonomic for typical CRUD APIs. |

### Database/ORM

| Recommended | Alternative | When to Use Alternative |
|-------------|-------------|-------------------------|
| **SeaORM** | SQLx | If you prefer raw SQL with compile-time checking. No ORM abstractions. Use for complex queries or when SeaORM's DSL is limiting. |
| **SeaORM** | Diesel | If you need synchronous ORM (rare with axum). Diesel's query DSL is more mature but sync-only is a dealbreaker for async stacks. |
| **SeaORM** | sea-query | If you want just the query builder without entity layer. SeaORM uses sea-query internally. |

### Frontend Framework

| Recommended | Alternative | When to Use Alternative |
|-------------|-------------|-------------------------|
| **Svelte 5** | React | If team has React expertise and migration cost is too high. React's ecosystem is larger but Svelte's reactivity model is cleaner for SDUI. |
| **Svelte 5** | Vue 3 | Similar reactivity model to Svelte but with virtual DOM. Choose if team prefers template syntax and larger ecosystem. |
| **Svelte 5** | SolidJS | If you want fine-grained reactivity without compilation. Smaller ecosystem than Svelte. |

### Component Library

| Recommended | Alternative | When to Use Alternative |
|-------------|-------------|-------------------------|
| **Flowbite Svelte** | Skeleton UI | More opinionated design system with theme customization. Choose if Flowbite's Tailwind-native approach feels too low-level. |
| **Flowbite Svelte** | shadcn-svelte | Unstyled/copy-paste components. Choose if you want full control over component implementation. |
| **Flowbite Svelte** | Melt UI | Headless components for maximum customization. More work but more control. |

### Validation

| Recommended | Alternative | When to Use Alternative |
|-------------|-------------|-------------------------|
| **validator** | garde | If you want a cleaner, rewritten version of validator. Inspired by validator but with better API. Consider for new projects. |
| **validator** | Custom validation | For complex cross-field validation logic that declarative rules can't express. |

---

## What NOT to Use

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| **Diesel** | Synchronous-only, doesn't fit async axum stack. Migration friction with SQLx ecosystem. | SeaORM |
| **actix-web (unless necessary)** | Actor model adds complexity. axum's Tower middleware ecosystem is more composable. | axum |
| **Svelte 4** | Svelte 5's runes system is significantly better for complex state. Svelte 4 is maintenance mode. | Svelte 5 |
| **Tailwind CSS 3.x** | v4 has major performance improvements (5x faster builds) and modern CSS features. | Tailwind CSS 4.0 |
| **Jest** | Vitest is faster, Vite-native, and Jest-compatible. No reason to use Jest in Vite projects. | Vitest |
| **jsdom for component tests** | Unreliable, doesn't match real browser behavior. Use Vitest browser mode. | vitest-browser-svelte + Playwright |
| **OpenAPI 3.0** | 3.1 has full JSON Schema compatibility. 3.0 has schema divergence issues. | OpenAPI 3.1 |
| **native-tls** | Platform-dependent, security update delays. rustls is pure Rust and consistent. | rustls |
| **dotenv** | Unmaintained. dotenvy is the maintained fork. | dotenvy |

---

## Stack Patterns by Variant

### If SQLite-only (development/small deployments):

```toml
sea-orm = { version = "1.1", features = ["sqlx-sqlite", "runtime-tokio-rustls", "macros"] }
```
- Simpler setup, no external database
- Use migrations to ensure schema portability to PostgreSQL

### If PostgreSQL (production):

```toml
sea-orm = { version = "1.1", features = ["sqlx-postgres", "runtime-tokio-rustls", "macros"] }
```
- Connection pooling via SQLx
- Consider PgBouncer for high-concurrency scenarios

### If WebSocket-heavy (real-time SDUI):

```toml
axum = { version = "0.8", features = ["macros", "ws"] }
tokio-tungstenite = "0.28"
```
- axum's WebSocket support is built on tokio-tungstenite
- Consider message batching for high-frequency updates

### If generating TypeScript client:

```bash
# Use openapi-typescript for type generation
npx openapi-typescript ./spec/openapi.yaml -o ./frontend/src/lib/api/types.ts
```
- Generates TypeScript types from OpenAPI spec
- Use with `openapi-fetch` for type-safe API calls

---

## Version Compatibility Matrix

| Package A | Compatible With | Notes |
|-----------|-----------------|-------|
| axum 0.8 | tower 0.5, tower-http 0.6, tokio 1.x | All part of Tokio ecosystem, tested together |
| sea-orm 1.1 | sqlx 0.8, tokio 1.x | sea-orm bundles compatible sqlx version |
| utoipa 5.4 | axum 0.8 via utoipa-axum 0.2 | Check utoipa-axum version matches |
| Svelte 5.48 | flowbite-svelte 1.31, Tailwind 4.0 | Flowbite 2.0-next targets Svelte 5 runes |
| Vitest 4.0 | Playwright 1.57 | Use Playwright as browser provider |

---

## Listmonk Integration Pattern

Listmonk provides a REST API for subscriber and campaign management. Integration approach:

```rust
// Create a typed client for Listmonk API
pub struct ListmonkClient {
    client: reqwest::Client,
    base_url: String,
    api_key: String,
}

impl ListmonkClient {
    pub async fn create_subscriber(&self, email: &str, name: &str, lists: &[i64]) -> Result<Subscriber> {
        self.client
            .post(format!("{}/api/subscribers", self.base_url))
            .basic_auth("", Some(&self.api_key))
            .json(&CreateSubscriberRequest { email, name, lists, status: "enabled" })
            .send()
            .await?
            .json()
            .await
    }
}
```

Key endpoints:
- `POST /api/subscribers` - Create/update subscriber
- `GET /api/subscribers` - List subscribers
- `POST /api/tx` - Send transactional email
- `GET /api/lists` - Get mailing lists

---

## Sources

### Rust Crate Versions (verified via docs.rs 2026-01-23)
- [axum 0.8.8](https://docs.rs/axum/0.8.8/axum/) - HIGH confidence
- [tokio 1.49.0](https://docs.rs/tokio/1.49.0/tokio/) - HIGH confidence
- [sea-orm 1.1.19](https://docs.rs/sea-orm/1.1.19/sea_orm/) - HIGH confidence
- [utoipa 5.4.0](https://docs.rs/utoipa/5.4.0/utoipa/) - HIGH confidence
- [tower-http 0.6.8](https://docs.rs/tower-http/0.6.8/tower_http/) - HIGH confidence
- [serde 1.0.228](https://docs.rs/serde/1.0.228/serde/) - HIGH confidence
- [serde_json 1.0.149](https://docs.rs/serde_json/1.0.149/serde_json/) - HIGH confidence
- [reqwest 0.13.1](https://docs.rs/reqwest/0.13.1/reqwest/) - HIGH confidence
- [tracing 0.1.44](https://docs.rs/tracing/0.1.44/tracing/) - HIGH confidence
- [tokio-tungstenite 0.28.0](https://docs.rs/tokio-tungstenite/0.28.0/tokio_tungstenite/) - HIGH confidence
- [validator 0.20.0](https://docs.rs/validator/0.20.0/validator/) - HIGH confidence

### Axum 0.8 Announcement
- [Tokio Blog - Announcing axum 0.8.0](https://tokio.rs/blog/2025-01-01-announcing-axum-0-8-0) - HIGH confidence

### SeaORM 2.0 Preview
- [SeaORM 2.0 Migration Guide](https://www.sea-ql.org/blog/2026-01-12-sea-orm-2.0/) - MEDIUM confidence (RC, not stable)

### Frontend Versions
- [Svelte 5.48.0 - What's New January 2026](https://svelte.dev/blog/whats-new-in-svelte-january-2026) - HIGH confidence
- [Flowbite Svelte 1.31.0](https://flowbite-svelte.com/docs/pages/introduction) - HIGH confidence
- [Tailwind CSS 4.0](https://tailwindcss.com/blog/tailwindcss-v4) - HIGH confidence
- [Vitest 4.0](https://vitest.dev/blog/vitest-4) - HIGH confidence
- [Playwright 1.57](https://playwright.dev/docs/release-notes) - HIGH confidence

### OpenAPI/JSON Schema
- [OpenAPI 3.1.0 Specification](https://swagger.io/specification/) - HIGH confidence
- [JSON Schema 2020-12](https://json-schema.org/) - HIGH confidence

### Integration
- [Listmonk Documentation](https://listmonk.app/docs/) - HIGH confidence

---

*Stack research for: OpenSDUI Protocol + Marionette Reference Implementation*
*Researched: 2026-01-23*
