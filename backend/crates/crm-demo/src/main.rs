#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

mod entities;
mod handlers;
mod migration;
mod seed;

use std::collections::HashMap;
use std::sync::Arc;

use axum::Router;
use sea_orm_migration::MigratorTrait;
use tower_http::services::{ServeDir, ServeFile};

use marionette::builders::standard::{Button, Container, Form, Heading, Text, TextInput};
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

/// Build the login form as a SDUI render message.
fn build_login_form() -> ProtocolMessage {
    let heading = Heading::new("Login").id("login-heading").build();
    let email_input = TextInput::new("Email")
        .id("login-email")
        .placeholder("Enter your email")
        .bind("/login/email")
        .build();
    let password_input = TextInput::new("Password")
        .id("login-password")
        .input_type("password")
        .bind("/login/password")
        .build();
    let submit_button = Button::new("Log In")
        .id("login-submit")
        .action(ComponentAction::submit("login"))
        .build();

    let form = Form::new()
        .id("login-form")
        .children(vec![email_input, password_input, submit_button])
        .build_with_children();

    let mut all_nodes = Vec::new();
    all_nodes.push(heading);
    all_nodes.extend(form);

    let container_nodes = Container::new()
        .id("login-root")
        .children(all_nodes)
        .build_with_children();

    let mut nodes = HashMap::new();
    for (id, component) in container_nodes {
        nodes.insert(id, component);
    }

    ProtocolMessage::Render(RenderMessage {
        id: None,
        surface: "main".into(),
        root: "login-root".into(),
        nodes,
        data: serde_json::json!({ "login": { "email": "", "password": "" } }),
    })
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // Initialize database with real SQLite
    let db = marionette::init_db("sqlite://crm.db?mode=rwc")
        .await
        .expect("failed to initialize database");

    // Run CRM-specific migrations
    migration::Migrator::up(&db, None)
        .await
        .expect("failed to run CRM migrations");

    // Seed default admin account
    seed::seed_admin(&db)
        .await
        .expect("failed to seed admin");

    let db = Arc::new(db);

    let action_router = ActionRouter::new()
        .action(
            "navigate",
            box_handler(handle_navigate),
            AuthRequirement::Authenticated,
        )
        .action(
            "demo_click",
            box_handler(handle_demo_click),
            AuthRequirement::Authenticated,
        );

    let state = Arc::new(AppState {
        router: action_router,
        db,
        login_form: Some(build_login_form()),
    });

    // Static files with SPA fallback
    let serve_dir = ServeDir::new("../frontend/build")
        .fallback(ServeFile::new("../frontend/build/index.html"));

    let app = Router::new()
        .route("/ws", axum::routing::any(ws_handler))
        .route("/api/health", axum::routing::get(health))
        .route(
            "/api/login",
            axum::routing::post(handlers::auth::handle_login),
        )
        .route(
            "/api/logout",
            axum::routing::post(handlers::auth::handle_logout),
        )
        .fallback_service(serve_dir)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001")
        .await
        .unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
