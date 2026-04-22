//! CRATE-02 integration test — the AppShell's sidebar nav contains one
//! NavItem per entry in `registered_demos()`. Auto-discovery is proven
//! by iterating the registry and asserting every key has a matching
//! `nav-<key>` node in the shell Render.

use std::sync::Arc;

use futures::{SinkExt, StreamExt};
use sea_orm::{DatabaseBackend, MockDatabase};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::connect_async;
use tower_http::services::{ServeDir, ServeFile};

use marionette::gallery::registered_demos;
use marionette::router::ActionRouter;
use marionette::ws::{AppState, ws_handler};

async fn start_server() -> String {
    let action_router =
        gallery_demo::handlers::register_gallery_actions(ActionRouter::new());
    let state = Arc::new(AppState {
        router: action_router,
        db: Arc::new(MockDatabase::new(DatabaseBackend::Sqlite).into_connection()),
        login_form: None,
        listmonk: None,
    });
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
    format!("ws://127.0.0.1:{port}/ws")
}

#[tokio::test]
async fn navigate_shell_render_includes_one_nav_item_per_registered_demo() {
    let url = start_server().await;
    let (mut ws, _resp) = connect_async(&url).await.expect("ws connect");

    // Frame 1: hello
    let _hello = ws.next().await.expect("hello").expect("hello ok");

    // Dispatch `navigate` (the WS-connect action the frontend fires).
    let nav_msg = serde_json::json!({
        "type": "action",
        "name": "navigate"
    });
    ws.send(Message::Text(nav_msg.to_string().into()))
        .await
        .expect("send navigate");

    // Read frames until we find a Render for surface "main" (the shell).
    // `navigate` emits three Renders (main, content, toasts) — buffering
    // may interleave, so we scan up to 5 frames.
    let mut shell_value: Option<serde_json::Value> = None;
    for _ in 0..5 {
        let msg = ws.next().await.expect("frame").expect("frame ok");
        let text = msg.to_text().expect("text").to_string();
        let v: serde_json::Value = serde_json::from_str(&text).expect("valid json");
        if v.get("type").and_then(|t| t.as_str()) == Some("render")
            && v.get("surface").and_then(|s| s.as_str()) == Some("main")
        {
            shell_value = Some(v);
            break;
        }
    }
    let shell = shell_value.expect("shell Render not received after 5 frames");

    let nodes = shell["nodes"]
        .as_object()
        .expect("shell Render must have nodes object");

    let expected_keys: Vec<&'static str> = registered_demos().map(|e| e.key).collect();
    assert!(
        !expected_keys.is_empty(),
        "registered_demos() should yield at least gallery-smoke's 'smoke' key \
         (Phase 16 landed that); Plan 04 adds 19 more"
    );

    for key in &expected_keys {
        let nav_id = format!("nav-{key}");
        assert!(
            nodes.contains_key(&nav_id),
            "shell Render missing NavItem '{nav_id}' for registered demo key '{key}'; \
             actual node ids: {:?}",
            nodes.keys().collect::<Vec<_>>()
        );
    }
}
