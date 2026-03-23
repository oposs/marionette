use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use futures::{SinkExt, StreamExt};
use sea_orm::{DatabaseBackend, MockDatabase};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use tower_http::services::{ServeDir, ServeFile};

use marionette::builders::standard::{Button, Container, Heading, Text};
use marionette::error::ActionResult;
use marionette::extractors::HandlerContext;
use marionette::router::{box_handler, ActionRouter};
use marionette::ws::{ws_handler, AppState};
use marionette_protocol::common::AuthRequirement;
use marionette_protocol::data::PatchOperation;
use marionette_protocol::{ComponentAction, PatchMessage, ProtocolMessage, RenderMessage};

/// Build the same handler as the crm-demo binary.
async fn handle_navigate(ctx: HandlerContext) -> ActionResult {
    let heading = Heading::new("Welcome to Marionette").id("h1").build();
    let text = Text::new("This demo proves the end-to-end protocol round-trip works.")
        .id("t1")
        .build();
    let button = Button::new("Click Me")
        .id("btn1")
        .action(ComponentAction::click("demo_click"))
        .build();

    let nodes_vec = Container::new()
        .id("root")
        .children(vec![heading, text, button])
        .build_with_children();

    let mut nodes = HashMap::new();
    for (id, component) in nodes_vec {
        nodes.insert(id, component);
    }

    Ok(vec![ProtocolMessage::Render(RenderMessage {
        id: ctx.action.id.clone(),
        surface: "main".into(),
        root: "root".into(),
        nodes,
        data: serde_json::json!({ "message": "Hello from the backend!" }),
    })])
}

async fn handle_demo_click(_ctx: HandlerContext) -> ActionResult {
    Ok(vec![ProtocolMessage::Patch(PatchMessage {
        id: None,
        patch: vec![PatchOperation {
            path: "/message".into(),
            value: serde_json::json!("Button was clicked!"),
        }],
    })])
}

async fn health() -> &'static str {
    "ok"
}

/// Start a test server with the full crm-demo router (including static file serving).
/// Returns (ws_url, port).
async fn start_server() -> (String, u16) {
    let action_router = ActionRouter::new()
        .action(
            "navigate",
            box_handler(handle_navigate),
            AuthRequirement::None,
        )
        .action(
            "demo_click",
            box_handler(handle_demo_click),
            AuthRequirement::None,
        );

    let state = Arc::new(AppState {
        router: action_router,
        db: Arc::new(MockDatabase::new(DatabaseBackend::Sqlite).into_connection()),
        login_form: None,
    });

    // Use the actual frontend build directory for SPA fallback testing.
    // Tests that need this should ensure `frontend/build/` exists (via `make build`).
    let frontend_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../frontend/build");

    let serve_dir = ServeDir::new(&frontend_dir)
        .fallback(ServeFile::new(frontend_dir.join("index.html")));

    let app = axum::Router::new()
        .route("/ws", axum::routing::any(ws_handler))
        .route("/api/health", axum::routing::get(health))
        .fallback_service(serve_dir)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let port = addr.port();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    (format!("ws://127.0.0.1:{port}/ws"), port)
}

#[tokio::test]
async fn hello_exchange() {
    let (url, _) = start_server().await;
    let (mut ws, _) = connect_async(&url).await.unwrap();

    // First message should be server hello
    let msg = ws.next().await.unwrap().unwrap();
    let text = msg.into_text().unwrap();
    let value: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(value["type"], "hello");
    assert_eq!(value["version"], "1.0.0");

    // Send client hello
    let client_hello = serde_json::json!({"type": "hello", "version": "1.0.0"});
    ws.send(Message::Text(serde_json::to_string(&client_hello).unwrap().into()))
        .await
        .unwrap();

    // Verify no error response comes back (short timeout)
    let result = tokio::time::timeout(Duration::from_millis(200), ws.next()).await;
    assert!(
        result.is_err(),
        "Expected timeout (no response to client hello), but got a message"
    );

    ws.close(None).await.unwrap();
}

#[tokio::test]
async fn navigate_round_trip() {
    let (url, _) = start_server().await;
    let (mut ws, _) = connect_async(&url).await.unwrap();

    // Skip hello
    let _ = ws.next().await.unwrap().unwrap();

    // Send navigate action
    let action = serde_json::json!({
        "type": "action",
        "name": "navigate",
        "payload": {"path": "/"}
    });
    ws.send(Message::Text(serde_json::to_string(&action).unwrap().into()))
        .await
        .unwrap();

    // Read render response
    let msg = ws.next().await.unwrap().unwrap();
    let text = msg.into_text().unwrap();
    let value: serde_json::Value = serde_json::from_str(&text).unwrap();

    assert_eq!(value["type"], "render");
    assert_eq!(value["surface"], "main");
    assert_eq!(value["root"], "root");

    // Verify component tree
    let nodes = value["nodes"].as_object().unwrap();
    assert!(nodes.contains_key("root"), "Missing root node");
    assert!(nodes.contains_key("h1"), "Missing heading node");
    assert!(nodes.contains_key("t1"), "Missing text node");
    assert!(nodes.contains_key("btn1"), "Missing button node");

    // Verify types
    assert_eq!(nodes["h1"]["type"], "heading");
    assert_eq!(nodes["t1"]["type"], "text");
    assert_eq!(nodes["btn1"]["type"], "button");

    ws.close(None).await.unwrap();
}

#[tokio::test]
async fn demo_click_patch() {
    let (url, _) = start_server().await;
    let (mut ws, _) = connect_async(&url).await.unwrap();

    // Skip hello
    let _ = ws.next().await.unwrap().unwrap();

    // Send navigate first
    let nav = serde_json::json!({
        "type": "action",
        "name": "navigate",
        "payload": {"path": "/"}
    });
    ws.send(Message::Text(serde_json::to_string(&nav).unwrap().into()))
        .await
        .unwrap();
    // Read render response
    let _ = ws.next().await.unwrap().unwrap();

    // Send demo_click action
    let click = serde_json::json!({
        "type": "action",
        "name": "demo_click"
    });
    ws.send(Message::Text(serde_json::to_string(&click).unwrap().into()))
        .await
        .unwrap();

    // Read patch response
    let msg = ws.next().await.unwrap().unwrap();
    let text = msg.into_text().unwrap();
    let value: serde_json::Value = serde_json::from_str(&text).unwrap();

    assert_eq!(value["type"], "patch");
    let patches = value["patch"].as_array().unwrap();
    assert_eq!(patches.len(), 1);
    assert_eq!(patches[0]["path"], "/message");
    assert_eq!(patches[0]["value"], "Button was clicked!");

    ws.close(None).await.unwrap();
}

#[tokio::test]
async fn health_endpoint() {
    let (_, port) = start_server().await;

    let resp = reqwest::get(format!("http://127.0.0.1:{port}/api/health"))
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    assert_eq!(resp.text().await.unwrap(), "ok");
}

#[tokio::test]
async fn spa_fallback_serves_index_for_deep_routes() {
    // This test verifies INTEG-01 SPA fallback behavior.
    // Requires frontend/build/index.html to exist (run `make build` first).
    let frontend_index = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../frontend/build/index.html");
    if !frontend_index.exists() {
        eprintln!(
            "SKIP: frontend/build/index.html not found. Run `make build` to enable this test."
        );
        return;
    }

    let (_, port) = start_server().await;

    // GET a deep route that has no corresponding static file
    let resp = reqwest::get(format!("http://127.0.0.1:{port}/some/deep/route"))
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "SPA fallback should return 200 for deep routes");
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("<!doctype html>") || body.contains("<!DOCTYPE html>") || body.contains("<html"),
        "Deep route should return index.html content"
    );

    // GET root should also return index.html
    let resp_root = reqwest::get(format!("http://127.0.0.1:{port}/"))
        .await
        .unwrap();
    assert_eq!(resp_root.status(), 200);
    let root_body = resp_root.text().await.unwrap();
    assert!(
        root_body.contains("<!doctype html>") || root_body.contains("<!DOCTYPE html>") || root_body.contains("<html"),
        "Root should return index.html content"
    );
}
