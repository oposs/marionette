//! Tutorial: People App — binary entry point.
//!
//! See `docs/AUTHORING.md` for the line-by-line walkthrough.

use std::sync::Arc;

use axum::Router;
use marionette::Extensions;
use marionette::router::ActionRouter;
use marionette::ws::{AppState, ws_handler};
use sea_orm::{DatabaseBackend, MockDatabase};
use tower_http::services::{ServeDir, ServeFile};

use tutorial_people_app::handlers::register_app_actions;
use tutorial_people_app::state::PeopleStore;

async fn health() -> &'static str {
    "ok"
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // The framework's `AppState.db` field requires a `DatabaseConnection`.
    // This tutorial keeps everything in memory, so we hand it a `MockDatabase`
    // that never sees a query — `PeopleStore` (registered as an extension
    // below) holds the rows. A real app would `init_db("sqlite://app.db")`
    // here and run migrations before constructing `AppState`.
    let db: Arc<sea_orm::DatabaseConnection> =
        Arc::new(MockDatabase::new(DatabaseBackend::Sqlite).into_connection());

    let router = register_app_actions(ActionRouter::new());

    // Register the in-memory store as a typed extension. Handlers reach it
    // via `ctx.extensions.get_arc::<PeopleStore>()` — no global statics.
    let extensions = Extensions::new().with(PeopleStore::new());

    let state = Arc::new(AppState {
        router,
        db,
        login_form: None,
        extensions,
    });

    let serve_dir =
        ServeDir::new("../frontend/build").fallback(ServeFile::new("../frontend/build/index.html"));

    let app = Router::new()
        .route("/ws", axum::routing::any(ws_handler))
        .route("/api/health", axum::routing::get(health))
        .fallback_service(serve_dir)
        .with_state(state);

    // Port 3003: 3001 = crm-demo, 3002 = gallery-demo, 3003 = this tutorial.
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3003")
        .await
        .expect("bind 3003");
    tracing::info!(
        "tutorial-people-app listening on {}",
        listener.local_addr().expect("local_addr")
    );
    axum::serve(listener, app).await.expect("serve");
}
