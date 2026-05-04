# Marionette

Reference implementation of **OpenSDUI** — an open server-driven UI protocol.

> **Status: pre-release.** The protocol, frontend library, and backend toolkit are all under active development. Expect breaking changes.

## What is this?

OpenSDUI lets a backend describe *what* a UI should render, and any conforming frontend renders it. New screens become a backend change. Different backends share one frontend.

Companies like Airbnb, Lyft, Netflix, and Zalando run on internal SDUI systems. OpenSDUI aims to make that pattern available as open infrastructure: a published protocol with a reference frontend (Svelte 5) and backend toolkit (Rust).

For the full vision, design rationale, and protocol primitives, read **[`docs/OpenSDUI-CONCEPT.md`](docs/OpenSDUI-CONCEPT.md)**.

For the authoritative protocol spec, read **[`spec/PROTOCOL.md`](spec/PROTOCOL.md)**.

## Layout

| Path        | Contents                                                                |
| ----------- | ----------------------------------------------------------------------- |
| `docs/`     | Design exposé, motivation, prior art                                    |
| `spec/`     | Protocol specification, JSON schemas, OpenAPI                           |
| `frontend/` | SvelteKit reference frontend (`marionette-frontend`)                    |
| `backend/`  | Rust workspace: protocol crate, builder library, demo apps              |

## Quickstart

Toolchain is pinned via [`mise`](https://mise.jdx.dev/) (`mise.toml`); see [`TOOLING.md`](TOOLING.md).

```sh
# Run the CRM demo (backend on :3001, frontend on :5173)
make dev

# Run the gallery demo (showcases every supported component)
make gallery-dev
```

Other targets:

```sh
make build   # release build (frontend + backend)
make test    # cargo test + vitest
make lint    # rustfmt, clippy, eslint, svelte-check, redocly
make e2e     # Playwright end-to-end tests
```

## License

[MIT](LICENSE) — Tobias Oetiker and contributors.
