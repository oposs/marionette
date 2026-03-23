#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

mod audit;
mod entities;
mod handlers;
mod migration;
mod seed;

use std::collections::HashMap;
use std::sync::Arc;

use axum::Router;
use sea_orm_migration::MigratorTrait;
use tower_http::services::{ServeDir, ServeFile};

use marionette::builders::standard::{
    Button, Container, Form, Heading, NavItem, SideNav, TextInput,
};
use marionette::extractors::{FromHandlerContext, Session};
use marionette::error::ActionResult;
use marionette::extractors::HandlerContext;
use marionette::router::{box_handler, ActionRouter};
use marionette::ws::{ws_handler, AppState};
use marionette_protocol::common::AuthRequirement;
use marionette_protocol::{ComponentAction, ProtocolMessage, RenderMessage};

/// Handle the `navigate` action: default authenticated view is the contact list.
async fn handle_navigate(ctx: HandlerContext) -> ActionResult {
    let session = Session::from_context(&ctx)?;
    let is_admin = session.roles.contains(&"admin".to_string());

    // Default authenticated view: show contact list
    let mut messages = handlers::contact::handle_contact_list(HandlerContext {
        action: ctx.action.clone(),
        db: ctx.db.clone(),
        session: ctx.session.clone(),
    })
    .await?;

    // Build sidebar navigation
    let mut nav_items: Vec<(String, marionette_protocol::Component)> = Vec::new();
    let home_item = NavItem::new("Home", "/")
        .id("nav-home")
        .action(ComponentAction::click("navigate"))
        .build();
    nav_items.push(home_item);

    let contacts_item = NavItem::new("Contacts", "/contacts")
        .id("nav-contacts")
        .action(ComponentAction::click("contact_list"))
        .build();
    nav_items.push(contacts_item);

    let companies_item = NavItem::new("Companies", "/companies")
        .id("nav-companies")
        .action(ComponentAction::click("company_list"))
        .build();
    nav_items.push(companies_item);

    if is_admin {
        let users_item = NavItem::new("Users", "/users")
            .id("nav-users")
            .action(ComponentAction::click("user_list"))
            .build();
        nav_items.push(users_item);

        let audit_item = NavItem::new("Audit Log", "/audit")
            .id("nav-audit")
            .action(ComponentAction::click("audit_list"))
            .build();
        nav_items.push(audit_item);
    }

    let side_nav_nodes = SideNav::new()
        .id("side-nav")
        .children(nav_items)
        .build_with_children();

    let mut nav_nodes_map = HashMap::new();
    for (id, component) in side_nav_nodes {
        nav_nodes_map.insert(id, component);
    }

    // Send sidebar render as a separate surface
    messages.push(ProtocolMessage::Render(RenderMessage {
        id: None,
        surface: "nav".into(),
        root: "side-nav".into(),
        nodes: nav_nodes_map,
        data: serde_json::json!({}),
    }));

    Ok(messages)
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

    // Seed demo companies and contacts
    seed::seed_companies(&db)
        .await
        .expect("failed to seed companies");
    seed::seed_contacts(&db)
        .await
        .expect("failed to seed contacts");
    seed::seed_tags(&db)
        .await
        .expect("failed to seed tags");
    seed::seed_notes(&db)
        .await
        .expect("failed to seed notes");
    seed::seed_interactions(&db)
        .await
        .expect("failed to seed interactions");

    let db = Arc::new(db);

    let action_router = ActionRouter::new()
        .action(
            "navigate",
            box_handler(handle_navigate),
            AuthRequirement::Authenticated,
        )
        .action(
            "contact_list",
            box_handler(handlers::contact::handle_contact_list),
            AuthRequirement::Authenticated,
        )
        .action(
            "contact_new",
            box_handler(handlers::contact::handle_contact_form),
            AuthRequirement::Authenticated,
        )
        .action(
            "contact_edit",
            box_handler(handlers::contact::handle_contact_form),
            AuthRequirement::Authenticated,
        )
        .action(
            "contact_save",
            box_handler(handlers::contact::handle_contact_save),
            AuthRequirement::Authenticated,
        )
        .action(
            "contact_delete",
            box_handler(handlers::contact::handle_contact_delete),
            AuthRequirement::Authenticated,
        )
        .action(
            "user_list",
            box_handler(handlers::user::handle_user_list),
            AuthRequirement::Role("admin"),
        )
        .action(
            "user_new",
            box_handler(handlers::user::handle_user_form),
            AuthRequirement::Role("admin"),
        )
        .action(
            "user_edit",
            box_handler(handlers::user::handle_user_form),
            AuthRequirement::Role("admin"),
        )
        .action(
            "user_save",
            box_handler(handlers::user::handle_user_save),
            AuthRequirement::Role("admin"),
        )
        .action(
            "user_delete",
            box_handler(handlers::user::handle_user_delete),
            AuthRequirement::Role("admin"),
        )
        .action(
            "audit_list",
            box_handler(handlers::audit::handle_audit_list),
            AuthRequirement::Role("admin"),
        )
        .action(
            "company_list",
            box_handler(handlers::company::handle_company_list),
            AuthRequirement::Authenticated,
        )
        .action(
            "company_new",
            box_handler(handlers::company::handle_company_form),
            AuthRequirement::Authenticated,
        )
        .action(
            "company_edit",
            box_handler(handlers::company::handle_company_form),
            AuthRequirement::Authenticated,
        )
        .action(
            "company_save",
            box_handler(handlers::company::handle_company_save),
            AuthRequirement::Authenticated,
        )
        .action(
            "company_delete",
            box_handler(handlers::company::handle_company_delete),
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
