# Writing a Marionette App in Rust

This guide walks through building a complete Marionette application from
scratch, using the **`tutorial-people-app`** crate (`backend/crates/tutorial-people-app/`)
as the worked example. The app shows an `AppShell` with a form and a table:
fill in name / email / country, click **Add person**, the row appears in the
table below.

When you finish reading this guide you should be able to:

1. Bootstrap a new Marionette server.
2. Define an action handler and register it.
3. Compose a UI from typed builders.
4. Hold app-specific state without leaning on global statics.

> All paths in this document are repo-relative. The framework lives in
> `backend/crates/marionette/`. The protocol spec is `spec/PROTOCOL.md`.
> Higher-level design rationale is `docs/OpenSDUI-CONCEPT.md`.

---

## Mental model in one paragraph

The frontend is a generic SDUI renderer. Your backend talks to it over a
single WebSocket. On every connect the frontend sends an `action` named
`navigate`; your `navigate` handler responds with one or more `Render`
messages that describe component trees and seed data. After that, every UI
event (button click, form submit, navigation) becomes another `action`
that hits one of your handlers. Handlers respond with `Patch` messages
(small data + tree edits), `Render` messages (whole-surface replacements),
or `Event` messages (toasts, dismissals). That's the whole loop.

---

## Project skeleton

A minimal app crate has six files:

```
backend/crates/your-app/
├── Cargo.toml
└── src/
    ├── main.rs            # binary entry point
    ├── lib.rs             # re-exports for tests
    ├── state.rs           # your app's data
    ├── ui.rs              # builders that compose your screens
    └── handlers/
        ├── mod.rs         # register_app_actions(router) -> ActionRouter
        ├── navigate.rs    # the entry-point handler
        └── …              # one file per handler family
```

`Cargo.toml` depends on the three Marionette crates plus the usual web stack:

```toml
[package]
name = "your-app"
edition.workspace = true
license.workspace = true
publish = false

[dependencies]
marionette = { path = "../marionette" }
marionette-protocol = { path = "../marionette-protocol" }
marionette-macros = { path = "../marionette-macros" }
axum.workspace = true
sea-orm.workspace = true
serde.workspace = true
serde_json.workspace = true
tokio.workspace = true
tower-http.workspace = true
tracing.workspace = true
tracing-subscriber.workspace = true
uuid.workspace = true
```

Add the crate to `backend/Cargo.toml` workspace members.

---

## The four macros

Marionette provides four macros, all re-exported from the `marionette` crate:

| Macro                        | Source                       | What it emits |
| ---------------------------- | ---------------------------- | ------------- |
| `#[derive(ComponentBuilder)]` | `marionette::ComponentBuilder` | A fluent `{Type}Builder` with `.id()`, `.bind()`, `.action()`, `.children()`, plus one setter per `#[builder(optional)]` field, and three build modes (`.build()`, `.build_tree()`, `.build_with_children()`). |
| `#[action(name = "…")]`       | `marionette::action`           | A `pub const FOO: &str = "…"` next to the handler. Non-identifier characters in the name (`-`, `/`, `.`) become `_` in the const. |
| `#[requires(authenticated)]` / `#[requires(role = "admin")]` | `marionette::requires` | A `pub const FOO_AUTH: AuthRequirement` next to the handler, used at registration. |
| `#[gallery_demo(key = "…")]`  | `marionette::gallery_demo`     | Gallery-only — registers the function in the `marionette::gallery::DEMOS` slice. Ignore for real apps. |

You will mostly write the first two. The `#[derive(ComponentBuilder)]` is
already applied to every shipped builder; you only need it if you author a
new component type.

---

## Bootstrap (`main.rs`)

Compare against `backend/crates/tutorial-people-app/src/main.rs`. The
shape is:

```rust
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // 1. A DatabaseConnection. Use SeaORM's MockDatabase if you don't
    //    have a real DB yet — AppState requires the field but nothing
    //    forces you to issue queries.
    let db: Arc<sea_orm::DatabaseConnection> =
        Arc::new(MockDatabase::new(DatabaseBackend::Sqlite).into_connection());

    // 2. Register every handler on a fresh ActionRouter.
    let router = register_app_actions(ActionRouter::new());

    // 3. Build the typed Extensions registry. Handlers reach app state
    //    via ctx.extensions.get::<T>() — never through global statics.
    let extensions = Extensions::new().with(PeopleStore::new());

    // 4. Hand everything to AppState and wrap in Arc.
    let state = Arc::new(AppState {
        router,
        db,
        login_form: None,         // Some(ProtocolMessage::Render(_)) for auth flows
        extensions,
    });

    // 5. Mount the WebSocket route + serve the frontend build.
    let serve_dir = ServeDir::new("../frontend/build")
        .fallback(ServeFile::new("../frontend/build/index.html"));
    let app = axum::Router::new()
        .route("/ws", axum::routing::any(ws_handler))
        .route("/api/health", axum::routing::get(|| async { "ok" }))
        .fallback_service(serve_dir)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3003").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

Three slots that confuse people:

- **`login_form`** — `Some(ProtocolMessage::Render(_))` if your app shows a
  login screen to unauthenticated WebSocket connections. `None` for apps
  with no auth.
- **`db`** — must be present even if you never query. Use `MockDatabase` for
  in-memory apps; the framework never issues queries on its own. (A future
  release will probably make this optional; for now, hand it the mock.)
- **`extensions`** — a typed registry where you register your own services
  (DB clients, in-memory stores, message brokers). See the next section.

---

## Holding app state — `Extensions`

`marionette::Extensions` is a typed registry keyed by `TypeId`. Apps insert
exactly one value per concrete type at bootstrap; handlers retrieve them
through `HandlerContext.extensions`. This is the **only** sanctioned way to
share app-specific state with handlers — do not use crate-local
`OnceLock` singletons or globals.

```rust
// state.rs
#[derive(Default)]
pub struct PeopleStore {
    rows: tokio::sync::RwLock<Vec<Person>>,
}

impl PeopleStore {
    pub fn new() -> Self { Self::default() }
    pub async fn snapshot(&self) -> Vec<Person> { … }
    pub async fn add(&self, p: Person) { … }
}
```

```rust
// main.rs
let extensions = Extensions::new()
    .with(PeopleStore::new())
    .with(MyOtherClient::connect(…).await?);
```

```rust
// any handler
let store = ctx
    .extensions
    .get_arc::<PeopleStore>()
    .ok_or_else(|| ActionError::Internal("PeopleStore not registered".into()))?;
let rows = store.snapshot().await;
```

`get::<T>()` returns `Option<&T>` for borrowed access; `get_arc::<T>()`
returns `Option<Arc<T>>` when you need to hold the value past the registry
borrow (e.g. across an `await`). `Extensions` is `Clone` and shares its
inner map via `Arc`, so the per-handler clone is cheap.

---

## Writing a handler

Every handler has the signature `async fn(HandlerContext) -> ActionResult`,
where `ActionResult = Result<Vec<ProtocolMessage>, ActionError>`. You wrap
the function with `box_handler(f)` at registration time.

```rust
#[action(name = "app/add-person")]
pub async fn handle_add_person(ctx: HandlerContext) -> ActionResult {
    let store = ctx.extensions
        .get_arc::<PeopleStore>()
        .ok_or_else(|| ActionError::Internal("…".into()))?;

    // The frontend's Form gathers values bound under its `bind` path
    // (here: `/form`) and ships them as the action payload.
    let payload: AddPersonPayload = ctx.action.payload
        .clone()
        .map(serde_json::from_value)
        .transpose()
        .map_err(|e| ActionError::BadPayload(e.to_string()))?
        .unwrap_or_default();

    store.add(Person { … }).await;
    let snapshot = store.snapshot().await;

    Ok(vec![
        // Replace /people in the content surface; clear the form fields.
        ProtocolMessage::Patch(PatchMessage {
            id: ctx.action.id.clone(),
            surface: "content".into(),
            patch: vec![
                PatchOperation::Set { path: "/people".into(), value: serde_json::to_value(&snapshot)? },
                PatchOperation::Set { path: "/form/name".into(), value: "".into() },
                PatchOperation::Set { path: "/form/email".into(), value: "".into() },
                PatchOperation::Set { path: "/form/country".into(), value: "".into() },
            ],
        }),
        // Toast events go to svelte-sonner at the layout root.
        ProtocolMessage::Event(EventMessage {
            id: None,
            surface: None,
            name: "toast".into(),
            hint: Some(serde_json::json!({ "message": "Person added.", "severity": "success" })),
        }),
    ])
}
```

`HandlerContext` carries:

- `action: ActionMessage` — the inbound action: `name`, `payload`, `id`, `source`, `optimistic`.
- `db: Arc<DatabaseConnection>` — your SeaORM connection.
- `session: Session` — `user_id`, `roles`. `Session::from_context(&ctx)?` works too if you prefer the extractor style.
- `extensions: Extensions` — your registry. Cheap to clone.

The three response shapes:

- **`Render`** — full replacement of one surface. Use on `navigate` and on
  major navigation events.
- **`Patch`** — `surface` + a vec of `PatchOperation`s applied in order. Cheap
  per-field updates that preserve focus, scroll, and unrelated state. Pick
  this whenever you can.
- **`Event`** — fire-and-forget signals. Toasts and modal-close use this.

### Registering handlers

```rust
// handlers/mod.rs
pub fn register_app_actions(router: ActionRouter) -> ActionRouter {
    router
        .action("navigate", box_handler(navigate::handle_navigate), AuthRequirement::None)
        .action(people::APP_ADD_PERSON, box_handler(people::handle_add_person), AuthRequirement::None)
}
```

Note `people::APP_ADD_PERSON` — that's the const the `#[action]` macro
emitted from `name = "app/add-person"`. The same const is used in the UI
builder's `Button::action(ComponentAction::submit(handlers::people::APP_ADD_PERSON))`,
so adding a handler and wiring the button reach for the same symbol — no
string-literal action names anywhere in your app.

---

## Building the UI

Marionette UIs are flat lists of typed nodes. Every builder produces tuples
or vecs of `(String, Component)` — the string is the node id, the component
holds props, bind path, action, children-by-id. There are three build
methods on every builder:

| Method | Returns | Use when |
| ------ | ------- | -------- |
| `.build()` | `(String, Component)` | The component is a leaf — no children, or you have already built and flattened them yourself. |
| `.build_tree()` | `((String, Component), Vec<(String, Component)>)` | You want the root tuple to feed into a parent's `.children()`, AND a separate descendants vec to merge into the parent's `.with_descendants()`. Used for `Form`, `Container`, `SideNav` slots. |
| `.build_with_children()` | `Vec<(String, Component)>` | You want one flat vec, ready to drop into `RenderMessage.nodes`. Used at the top of an assembly (the page root, or `AppShell`). |

### AppShell

`AppShell` has six named slots. You build each slot's child separately,
then hand it to the matching slot method. Descendants from each slot's
`.build_tree()` go into `.with_descendants()`.

```rust
let (sidebar_root, sidebar_desc) = SideNav::new()
    .id("shell-side-nav")
    .children(vec![nav_item])
    .build_tree();

let (header_root, header_desc) = Container::new()
    .id("shell-header")
    .children(vec![Heading::new("App title").id("header-title").build()])
    .build_tree();

let content_mount = SurfaceMount::new("content").id("shell-content-mount").build();

let mut descendants = Vec::new();
descendants.extend(sidebar_desc);
descendants.extend(header_desc);

let shell_nodes: Vec<(String, Component)> = AppShell::new()
    .id("app-shell-root")
    .sidebar(sidebar_root)
    .header(header_root)
    .main(content_mount)
    .with_descendants(descendants)
    .build_with_children();
```

Slots are: `sidebar`, `header`, `footer`, `main`, `popups`, `toasts`. You
do not need to populate `popups` or `toasts` — modal overlays are rendered
by a global `ModalSurface` mounted at the layout root, and toasts ride on
`ProtocolMessage::Event`.

### Form + DataTable

The Form's `bind` path defines the data subtree the frontend gathers as the
submit payload. Each input's `bind` is rooted under the form's bind:

```rust
let name_field = TextInput::new("Name")
    .id("field-name")
    .placeholder("Ada Lovelace")
    .bind("/form/name")
    .build();
let email_field = TextInput::new("Email").id("field-email").bind("/form/email").build();
let country_field = Select::new("Country", country_options())
    .id("field-country")
    .bind("/form/country")
    .build();
let submit = Button::new("Add person")
    .id("btn-add")
    .action(ComponentAction::submit(handlers::people::APP_ADD_PERSON))
    .build();

let (form_root, form_desc) = Form::new()
    .id("people-form")
    .bind("/form")          // <- payload root
    .children(vec![name_field, email_field, country_field, submit])
    .build_tree();
```

When the user clicks **Add person**, the frontend dispatches an `action`
named `app/add-person` with `payload = data at /form` — the
`AddPersonPayload` struct in your handler matches the field names verbatim.

`DataTable` reads its rows from a `bind` path (no fetch handler needed for
small in-memory data sets):

```rust
let columns = vec![
    TableColumn::new("name", "Name").sortable(),
    TableColumn::new("email", "Email"),
    TableColumn::new("country", "Country"),
];
let (table_id, table_component) = DataTable::new(columns)
    .id("people-table")
    .bind("/people")
    .row_id_key("id")
    .build();
```

For larger / paginated tables, set `.source("…")` and write a corresponding
`fetch-rows` handler — see `gallery-demo/src/handlers/fetch_rows.rs` for
the canonical pattern.

### The `navigate` handler

`navigate` is the entry point — fired automatically by the frontend on
every WebSocket connect. Emit three Renders in this order: `main` (the
shell), `content` (your home page), `modal` (an empty `Container` with id
`"modal-empty"`). The third Render keeps `ModalSurface` from showing a
loading skeleton — required even though your app has no modals.

```rust
pub async fn handle_navigate(ctx: HandlerContext) -> ActionResult {
    let store = ctx.extensions.get_arc::<PeopleStore>().ok_or_else(|| …)?;

    // Build the shell once.
    let shell_nodes: HashMap<String, Component> =
        ui::build_app_shell().into_iter().collect();

    // Build the page (form + table) seeded with current data.
    let rows = store.snapshot().await;
    let (page_root, page_nodes, page_data) = ui::build_people_page(rows);

    // Modal sentinel — see comment above.
    let (modal_root, modal_component) = Container::new().id("modal-empty").build();
    let mut modal_nodes = HashMap::new();
    modal_nodes.insert(modal_root.clone(), modal_component);

    Ok(vec![
        ProtocolMessage::Render(RenderMessage {
            id: None, surface: "main".into(), root: "app-shell-root".into(),
            nodes: shell_nodes,
            data: serde_json::json!({ "system": { "connectionStatus": "connected" } }),
        }),
        ProtocolMessage::Render(RenderMessage {
            id: None, surface: "content".into(), root: page_root,
            nodes: page_nodes, data: page_data,
        }),
        ProtocolMessage::Render(RenderMessage {
            id: None, surface: "modal".into(), root: modal_root,
            nodes: modal_nodes, data: serde_json::json!({}),
        }),
    ])
}
```

---

## Running the app

```sh
make tutorial-people-app
```

This launches the backend on `:3003` and the frontend dev server on `:5173`.
Open <http://localhost:5173/>. You'll see the AppShell with a header, a
sidebar entry "People", a form, and an empty table. Add a person — the row
appears, the form clears, a success toast fires.

For your own app, replicate the `tutorial-people-app` Makefile target with
your own port. Production builds use `npm run build` then serve the static
build from `tower-http::ServeDir` (already wired in `main.rs` above).

---

## Where to go from here

- **Per-row actions in DataTable** — use `ColumnKind::Actions` to render a
  dropdown menu per row. See `gallery-demo` examples.
- **Validation feedback on individual fields** — the `_errors/<bind>`
  convention shows error text inline. See
  `gallery-demo/src/handlers/catalog_forms.rs`.
- **A real database** — swap `MockDatabase` for `init_db("sqlite://app.db")`
  and add SeaORM entities + migrations. `crm-demo` is the (currently
  outdated) reference; treat its overall shape as illustrative but not its
  per-handler patterns.
- **Auth + sessions** — set `AppState.login_form` to a Render of your login
  screen. Handlers wrap with `#[requires(authenticated)]`. Look at
  `marionette/src/auth.rs` and `marionette/src/session.rs`.

The protocol primitives (Component, Data tree, Action) are documented in
`spec/PROTOCOL.md`. The design rationale (and what the framework
deliberately does NOT do) is in `docs/OpenSDUI-CONCEPT.md`.
