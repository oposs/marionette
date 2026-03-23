#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

mod entities;
mod handlers;
mod migration;
mod seed;

use std::collections::HashMap;
use std::sync::Arc;

use axum::Router;
use sea_orm::{DatabaseBackend, MockDatabase};
use tower_http::services::{ServeDir, ServeFile};

use marionette::builders::standard::{Button, Container, Heading, Text};
use marionette::error::ActionResult;
use marionette::extractors::HandlerContext;
use marionette::router::{box_handler, ActionRouter};
use marionette::ws::{ws_handler, AppState};
use marionette_protocol::common::AuthRequirement;
use marionette_protocol::data::PatchOperation;
use marionette_protocol::{ComponentAction, PatchMessage, ProtocolMessage, RenderMessage};

/// Handle the `navigate` action by returning a demo render message.
async fn handle_navigate(ctx: HandlerContext) -> ActionResult {
    let heading = Heading::new("Welcome to Marionette").id("h1").build();
    let text = Text::new("This demo proves the end-to-end protocol round-trip works.")
        .id("t1")
        .build();
    let button = Button::new("Click Me")
        .id("btn1")
        .action(ComponentAction::click("demo_click"))
        .build();
    let msg_text = Text::new("")
        .id("msg1")
        .bind("/message")
        .build();

    let nodes_vec = Container::new()
        .id("root")
        .children(vec![heading, text, button, msg_text])
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

/// Handle the `demo_click` action by returning a patch that updates the message.
async fn handle_demo_click(_ctx: HandlerContext) -> ActionResult {
    Ok(vec![ProtocolMessage::Patch(PatchMessage {
        id: None,
        patch: vec![PatchOperation {
            path: "/message".into(),
            value: serde_json::json!("Button was clicked!"),
        }],
    })])
}

/// Simple health check endpoint.
async fn health() -> &'static str {
    "ok"
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let action_router = ActionRouter::new()
        .action("navigate", box_handler(handle_navigate), AuthRequirement::None)
        .action(
            "demo_click",
            box_handler(handle_demo_click),
            AuthRequirement::None,
        );

    let state = Arc::new(AppState {
        router: action_router,
        db: Arc::new(MockDatabase::new(DatabaseBackend::Sqlite).into_connection()),
    });

    // Static files with SPA fallback
    let serve_dir = ServeDir::new("../frontend/build")
        .fallback(ServeFile::new("../frontend/build/index.html"));

    let app = Router::new()
        .route("/ws", axum::routing::any(ws_handler))
        .route("/api/health", axum::routing::get(health))
        .fallback_service(serve_dir)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001")
        .await
        .unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
