#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::doc_markdown)]

//! Gallery-demo binary entry point.
//!
//! Thin backend per Plan 17 CRATE-01: no auth, no real DB, no migrations.
//! Uses `sea_orm::MockDatabase` to satisfy `AppState.db: Arc<DatabaseConnection>`
//! without issuing any SQL (RESEARCH.md §Pitfall #2).
//!
//! Modules (`handlers`, `home`, `state`) live in the sibling `lib.rs` so
//! integration tests in `tests/*.rs` can `use gallery_demo::handlers::...`.

use std::sync::Arc;

use axum::Router;
use sea_orm::{DatabaseBackend, MockDatabase};
use tower_http::services::{ServeDir, ServeFile};

use marionette::router::ActionRouter;
use marionette::ws::{AppState, ws_handler};

/// Simple health check endpoint.
async fn health() -> &'static str {
    "ok"
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // MockDatabase satisfies AppState.db without a real DB backend
    // (per RESEARCH.md §Pitfall #2 — ws.rs session-auth path requires the
    //  Arc<DatabaseConnection> field; MockDatabase returns immediately
    //  without issuing any query plan. Zero SQL in the gallery by design.)
    let db: Arc<sea_orm::DatabaseConnection> =
        Arc::new(MockDatabase::new(DatabaseBackend::Sqlite).into_connection());

    let action_router =
        gallery_demo::handlers::register_gallery_actions(ActionRouter::new());

    let state = Arc::new(AppState {
        router: action_router,
        db,
        login_form: None,
        listmonk: None,
    });

    // Static file serving with SPA fallback (tower-http::ServeDir).
    // Path is relative to the binary's runtime cwd — `cargo run -p gallery-demo`
    // invoked from `backend/` (per the `gallery-dev` Makefile target) resolves
    // `../frontend/build/` to the repo's `frontend/build/` directory, matching
    // the CRM demo's pattern (crm-demo/src/main.rs).
    let serve_dir = ServeDir::new("../frontend/build")
        .fallback(ServeFile::new("../frontend/build/index.html"));

    let app = Router::new()
        .route("/ws", axum::routing::any(ws_handler))
        .route("/api/health", axum::routing::get(health))
        .fallback_service(serve_dir)
        .with_state(state);

    // Port 3002 (CRM is 3001; gallery-demo is 3002 per CONTEXT.md §Claude's Discretion).
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3002")
        .await
        .expect("bind 3002");
    tracing::info!(
        "gallery-demo listening on {}",
        listener.local_addr().expect("local_addr")
    );
    axum::serve(listener, app).await.expect("serve");
}
