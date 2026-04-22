//! Integration smoke test — boots the gallery-demo app on an ephemeral port
//! and asserts the WebSocket hello frame is emitted correctly.
//!
//! Per Plan 17-03 Task 3 Step 1: uses `gallery_demo::handlers::register_gallery_actions`
//! (reachable because the crate exposes a lib.rs). Binds `127.0.0.1:0`
//! rather than the fixed `3002` so multiple test runs don't collide.

use std::sync::Arc;

use futures::StreamExt;
use sea_orm::{DatabaseBackend, MockDatabase};
use tokio_tungstenite::connect_async;
use tower_http::services::{ServeDir, ServeFile};

use marionette::router::ActionRouter;
use marionette::ws::{AppState, ws_handler};

async fn start_server() -> (String, u16) {
    let action_router =
        gallery_demo::handlers::register_gallery_actions(ActionRouter::new());
    let state = Arc::new(AppState {
        router: action_router,
        db: Arc::new(MockDatabase::new(DatabaseBackend::Sqlite).into_connection()),
        login_form: None,
        listmonk: None,
    });

    // Frontend build path, relative to the test's CARGO_MANIFEST_DIR
    // (backend/crates/gallery-demo).
    let frontend_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../frontend/build");
    let serve_dir = ServeDir::new(&frontend_dir)
        .fallback(ServeFile::new(frontend_dir.join("index.html")));

    let app = axum::Router::new()
        .route("/ws", axum::routing::any(ws_handler))
        .fallback_service(serve_dir)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let port = listener.local_addr().expect("local_addr").port();
    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve");
    });
    (format!("ws://127.0.0.1:{port}/ws"), port)
}

#[tokio::test]
async fn gallery_demo_boots_and_emits_hello() {
    let (url, _port) = start_server().await;
    let (mut ws, _resp) = connect_async(&url).await.expect("ws connect");
    let msg = ws.next().await.expect("has msg").expect("msg ok");
    let text = msg.into_text().expect("text frame");

    // Parse as serde_json::Value so the assertion is robust to serde field-ordering.
    // ws.rs:108-110 sends ProtocolMessage::Hello(HelloMessage { version: "1.1.0" })
    // and the enum is tagged `#[serde(tag = "type", rename_all = "lowercase")]`
    // (marionette-protocol/src/messages.rs:13), so the wire shape is
    //   {"type":"hello","version":"1.1.0"}
    // (with possibly different key ordering — the Value lookup doesn't care).
    let v: serde_json::Value = serde_json::from_str(&text).expect("valid json");
    assert_eq!(
        v.get("type").and_then(|t| t.as_str()),
        Some("hello"),
        "first WS frame should have type=hello, got: {text}"
    );
    assert!(
        v.get("version").and_then(|s| s.as_str()).is_some(),
        "hello frame should include a version field, got: {text}"
    );
}
