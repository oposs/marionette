use std::sync::Arc;

use axum::routing::get;
use futures::{SinkExt, StreamExt};
use sea_orm::{DatabaseBackend, MockDatabase};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

use marionette::router::ActionRouter;
use marionette::ws::{ws_handler, AppState};
use marionette_protocol::common::AuthRequirement;
use marionette_protocol::data::PatchOperation;
use marionette_protocol::{ErrorMessage, HelloMessage, PatchMessage, ProtocolMessage};

fn mock_db() -> Arc<sea_orm::DatabaseConnection> {
    Arc::new(MockDatabase::new(DatabaseBackend::Sqlite).into_connection())
}

/// Start a test server with optional router configuration.
async fn start_test_server(router: ActionRouter) -> String {
    let app_state = Arc::new(AppState {
        router,
        db: mock_db(),
    });

    let app = axum::Router::new()
        .route("/ws", get(ws_handler))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    format!("ws://127.0.0.1:{}/ws", addr.port())
}

fn echo_handler() -> marionette::router::BoxedHandler {
    marionette::router::box_handler(|_ctx| async move {
        Ok(vec![ProtocolMessage::Patch(PatchMessage {
            id: None,
            patch: vec![PatchOperation {
                path: "/echo".into(),
                value: serde_json::json!("ok"),
            }],
        })])
    })
}

#[tokio::test]
async fn ws_connects_and_receives_hello() {
    let url = start_test_server(ActionRouter::new()).await;
    let (mut ws, _) = connect_async(&url).await.unwrap();

    // First message should be HelloMessage
    let msg = ws.next().await.unwrap().unwrap();
    let text = msg.into_text().unwrap();
    let proto: ProtocolMessage = serde_json::from_str(&text).unwrap();

    match proto {
        ProtocolMessage::Hello(HelloMessage { version }) => {
            assert_eq!(version, "1.0.0");
        }
        other => panic!("Expected Hello, got {other:?}"),
    }

    // Clean close
    ws.close(None).await.unwrap();
}

#[tokio::test]
async fn ws_dispatches_action() {
    let router = ActionRouter::new()
        .action("echo", echo_handler(), AuthRequirement::None);

    let url = start_test_server(router).await;
    let (mut ws, _) = connect_async(&url).await.unwrap();

    // Consume hello
    let _ = ws.next().await.unwrap().unwrap();

    // Send an action
    let action = serde_json::json!({
        "name": "echo",
        "payload": { "data": "test" }
    });
    ws.send(Message::Text(serde_json::to_string(&action).unwrap().into())).await.unwrap();

    // Should receive a PatchMessage response
    let msg = ws.next().await.unwrap().unwrap();
    let text = msg.into_text().unwrap();
    let proto: ProtocolMessage = serde_json::from_str(&text).unwrap();

    assert!(
        matches!(proto, ProtocolMessage::Patch(_)),
        "Expected Patch, got {proto:?}"
    );

    ws.close(None).await.unwrap();
}

#[tokio::test]
async fn ws_unknown_action_returns_error() {
    let url = start_test_server(ActionRouter::new()).await;
    let (mut ws, _) = connect_async(&url).await.unwrap();

    // Consume hello
    let _ = ws.next().await.unwrap().unwrap();

    // Send an action with unregistered name
    let action = serde_json::json!({
        "name": "nonexistent"
    });
    ws.send(Message::Text(serde_json::to_string(&action).unwrap().into())).await.unwrap();

    // Should receive ErrorMessage with "not found"
    let msg = ws.next().await.unwrap().unwrap();
    let text = msg.into_text().unwrap();
    let proto: ProtocolMessage = serde_json::from_str(&text).unwrap();

    match proto {
        ProtocolMessage::Error(ErrorMessage { errors, .. }) => {
            assert!(
                errors[0].message.to_lowercase().contains("not found"),
                "Expected 'not found' in error message, got: {}",
                errors[0].message
            );
        }
        other => panic!("Expected Error, got {other:?}"),
    }

    ws.close(None).await.unwrap();
}

#[tokio::test]
async fn ws_invalid_json_returns_error() {
    let url = start_test_server(ActionRouter::new()).await;
    let (mut ws, _) = connect_async(&url).await.unwrap();

    // Consume hello
    let _ = ws.next().await.unwrap().unwrap();

    // Send malformed JSON
    ws.send(Message::Text("{ invalid json!!!".into())).await.unwrap();

    // Should receive ErrorMessage
    let msg = ws.next().await.unwrap().unwrap();
    let text = msg.into_text().unwrap();
    let proto: ProtocolMessage = serde_json::from_str(&text).unwrap();

    match proto {
        ProtocolMessage::Error(ErrorMessage { errors, .. }) => {
            assert!(
                errors[0].message.to_lowercase().contains("invalid"),
                "Expected 'invalid' in error message, got: {}",
                errors[0].message
            );
        }
        other => panic!("Expected Error, got {other:?}"),
    }

    ws.close(None).await.unwrap();
}

#[tokio::test]
async fn ws_connection_closes_gracefully() {
    let url = start_test_server(ActionRouter::new()).await;
    let (mut ws, _) = connect_async(&url).await.unwrap();

    // Consume hello
    let _ = ws.next().await.unwrap().unwrap();

    // Send close frame
    ws.close(None).await.unwrap();

    // The stream should terminate without error (next returns None or Close)
    loop {
        match ws.next().await {
            None => break,
            Some(Ok(Message::Close(_))) => break,
            Some(Ok(_)) => continue, // skip any remaining messages
            Some(Err(_)) => break,   // connection closed, that's fine
        }
    }
}
