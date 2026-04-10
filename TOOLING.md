# Rust + Svelte Web Application

## Stack Overview

This is a web application with a Rust backend and Svelte frontend.

## Backend (Rust)

### Web Framework
- **axum** - Modern, ergonomic web framework built by the Tokio team
  - Tower middleware ecosystem
  - Type-safe extractors
  - First-class async support

### OpenAPI
- **utoipa** - Generate OpenAPI documentation from Rust code
- **utoipa-swagger-ui** - Serve Swagger UI for API exploration

### Validation
- **validator** - Declarative validation rules (email, length, regex, etc.)
- **axum-valid** - Integrates validator with Axum extractors

### Database
- **sea-orm** - Async ORM built on SQLx
  - Query builder DSL (SQL::Abstract-like)
  - Entity-based patterns
  - Supports PostgreSQL and SQLite
- **sea-orm-migration** - Database migrations

### Core Dependencies
- **tokio** - Async runtime
- **serde** / **serde_json** - Serialization/deserialization
- **tower-http** - CORS, compression, static file serving
- **tracing** - Logging and observability

## Frontend (Svelte)

- **Svelte 5** - Modern reactive UI framework
- **Flowbite Svelte** - Tailwind CSS component library

## Development Approach

- Code-first OpenAPI: Define types in Rust, generate OpenAPI spec automatically
- Request validation via Rust's type system + validator crate
- Serve built Svelte app as static files from Axum

## SQL Conventions

- **Artificial keys**: Always use auto-increment INTEGER primary keys
- **Table names**: Singular (`customer`, `location`, `task_log`)
- **Field names**: `<table>_<field>` (e.g., `customer_name`, `customer_email`)
- **Foreign keys**: `<table>_<other_table>` (e.g., `location_customer` → `customer.customer_id`)
- **Primary keys**: `<table>_id` (e.g., `customer_id`, `location_id`)
- **Translatable text**: JSON columns with language keys: `{"de": "...", "fr": "..."}`
- **JSON validation**: Always add CHECK constraints for JSON fields:
  ```sql
  CHECK (json_valid(field_name) AND json_type(field_name) = 'object')
  ```

## Build & Development

- **Makefile**: All build/dev tasks controlled via Makefile targets
  - `make dev` - Start development servers
  - `make build` - Production build
  - `make test` - Run all tests
  - `make lint` - Run linters
  - `make clean` - Clean build artifacts

- **GitHub Workflows**: CI/CD via `.github/workflows/`
  - Automated unit tests on PR
  - Automated integration tests
  - Build verification

- **Testing**: Unit, component, and integration tests required
  - Rust: `cargo test` for unit tests
  - Frontend unit: Vitest for business logic, utilities, stores
  - Frontend component: `vitest-browser-svelte` + Playwright (real browser, not jsdom)
  - E2E: Playwright for full user flows
  - Integration: API endpoint tests
