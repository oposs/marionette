#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

mod audit;
mod entities;
mod handlers;
mod listmonk;
mod migration;
mod seed;

use std::collections::HashMap;
use std::sync::Arc;

use axum::Router;
use sea_orm_migration::MigratorTrait;
use tower_http::services::{ServeDir, ServeFile};

use marionette::builders::app_shell::AppShell;
use marionette::builders::standard::{
    Button, Container, Form, Heading, NavItem, SideNav, SurfaceMount, TextInput,
};
use marionette::extractors::{FromHandlerContext, Session};
use marionette::error::ActionResult;
use marionette::extractors::HandlerContext;
use marionette::router::{box_handler, ActionRouter};
use marionette::ws::{ws_handler, AppState};
use marionette_protocol::common::AuthRequirement;
use marionette_protocol::{ComponentAction, ProtocolMessage, RenderMessage};

/// Handle the `login` action via WebSocket.
///
/// Validates credentials, updates the WsSession auth state inline, and sends
/// the authenticated view (contact list + sidebar) as the response.
async fn handle_login_action(ctx: HandlerContext) -> ActionResult {
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, ActiveModelTrait, ActiveValue::Set};
    use chrono::Utc;

    let payload = ctx.action.payload.clone().unwrap_or_default();
    // Form data is under /login/email and /login/password in the surface data store
    let login_data = payload.get("login").and_then(|v| v.as_object());
    let email = login_data
        .and_then(|d| d.get("email"))
        .and_then(|v| v.as_str())
        .or_else(|| payload.get("email").and_then(|v| v.as_str()))
        .unwrap_or("");
    let password = login_data
        .and_then(|d| d.get("password"))
        .and_then(|v| v.as_str())
        .or_else(|| payload.get("password").and_then(|v| v.as_str()))
        .unwrap_or("");

    if email.is_empty() || password.is_empty() {
        return Ok(vec![ProtocolMessage::Error(marionette_protocol::ErrorMessage {
            id: ctx.action.id.clone(),
            errors: vec![marionette_protocol::ValidationError {
                path: None,
                message: "Email and password are required".into(),
            }],
        })]);
    }

    // Look up user
    let user = entities::user::Entity::find()
        .filter(entities::user::Column::UserEmail.eq(email))
        .one(&*ctx.db)
        .await
        .map_err(|e| marionette::error::ActionError::Internal(e.to_string()))?
        .ok_or_else(|| marionette::error::ActionError::BadPayload("Invalid credentials".into()))?;

    // Verify password
    let hash = user.user_password.clone();
    let pw = password.to_string();
    let valid = tokio::task::spawn_blocking(move || bcrypt::verify(pw, &hash))
        .await
        .map_err(|e| marionette::error::ActionError::Internal(e.to_string()))?
        .unwrap_or(false);

    if !valid {
        return Ok(vec![ProtocolMessage::Error(marionette_protocol::ErrorMessage {
            id: ctx.action.id.clone(),
            errors: vec![marionette_protocol::ValidationError {
                path: None,
                message: "Invalid email or password".into(),
            }],
        })]);
    }

    // Update last login
    let user_id = user.user_id;
    let user_role = user.user_role.clone();
    let mut active_user: entities::user::ActiveModel = user.into();
    active_user.user_last_login = Set(Some(Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()));
    let _ = active_user.update(&*ctx.db).await;

    // Temporarily set session auth so navigate handler works
    let authenticated_ctx = HandlerContext {
        action: ctx.action.clone(),
        db: ctx.db.clone(),
        session: marionette::extractors::Session {
            user_id: Some(user_id.to_string()),
            roles: vec![user_role.clone()],
        },
    };

    // Get the authenticated view
    let mut messages = handle_navigate(authenticated_ctx).await?;

    // Embed auth info in the first Render message so ws.rs can update the WsSession
    for msg in &mut messages {
        if let ProtocolMessage::Render(render) = msg {
            if let Some(data) = render.data.as_object_mut() {
                data.insert("_auth_user_id".into(), serde_json::json!(user_id));
                data.insert("_auth_role".into(), serde_json::json!(user_role));
            }
            break;
        }
    }

    Ok(messages)
}

/// Handle the `navigate` action: build `AppShell` + render the contact list
/// into the `content` sub-surface as the default authenticated view.
///
/// Per D-B5/D-B6/D-B9/D-B11/D-B12/D-B13: the shell is rendered into the
/// `main` surface exactly once; screen content lives in the `content`
/// sub-surface so per-screen navigation patches the content region
/// without tearing down the shell tree.
#[allow(clippy::too_many_lines)]
async fn handle_navigate(ctx: HandlerContext) -> ActionResult {
    let session = Session::from_context(&ctx)?;
    let is_admin = session.roles.contains(&"admin".to_string());
    let user_name = session
        .user_id
        .clone()
        .unwrap_or_else(|| "User".to_string());

    // -- Build the content (screen) render first --
    // The contact_list handler renders into surface "content" (Task 2) and
    // also emits a nav_active_patch targeting /nav/active/contacts = true
    // (Plan 07 Task 2 / D-B13). We append its messages after the shell
    // Render below so they apply in order.
    let mut content_messages = handlers::contact::handle_contact_list(HandlerContext {
        action: ctx.action.clone(),
        db: ctx.db.clone(),
        session: ctx.session.clone(),
    })
    .await?;

    // -- Build the sidebar sub-tree (SideNav with NavItems) --
    // Per D-B13: NavItems bind to /nav/active/<slug>; the per-screen
    // handlers emit a PatchMessage (Set op) updating that path on every
    // navigation so the active indicator tracks the visible screen.
    let mut nav_items: Vec<(String, marionette_protocol::Component)> = Vec::new();
    nav_items.push(
        NavItem::new("Home", "/")
            .id("nav-home")
            .bind("/nav/active/home")
            .action(ComponentAction::click("navigate"))
            .build(),
    );
    nav_items.push(
        NavItem::new("Contacts", "/contacts")
            .id("nav-contacts")
            .bind("/nav/active/contacts")
            .action(ComponentAction::click("contact_list"))
            .build(),
    );
    nav_items.push(
        NavItem::new("Companies", "/companies")
            .id("nav-companies")
            .bind("/nav/active/companies")
            .action(ComponentAction::click("company_list"))
            .build(),
    );
    if is_admin {
        nav_items.push(
            NavItem::new("Users", "/users")
                .id("nav-users")
                .bind("/nav/active/users")
                .action(ComponentAction::click("user_list"))
                .build(),
        );
        nav_items.push(
            NavItem::new("Audit Log", "/audit")
                .id("nav-audit")
                .bind("/nav/active/audit")
                .action(ComponentAction::click("audit_list"))
                .build(),
        );
    }
    let (sidebar_root, sidebar_desc) = SideNav::new()
        .id("shell-side-nav")
        .children(nav_items)
        .build_tree();

    // -- Build the header sub-tree (title + user menu placeholder, D-B5) --
    let header_title = Heading::new("Marionette CRM").id("header-title").build();
    let header_user = Heading::new(format!("User: {user_name}"))
        .id("header-user")
        .build();
    let (header_root, header_desc) = Container::new()
        .id("shell-header")
        .children(vec![header_title, header_user])
        .build_tree();

    // -- Build the footer sub-tree (D-B6: version + connection status + legal) --
    // The three footer children correspond verbatim to D-B6:
    //   (1) version info       — static literal text
    //   (2) connection status  — data-bound Heading tracking
    //                            /system/connectionStatus. This is the role
    //                            the retired ConnectionBanner played: less
    //                            obtrusive than a top banner, always visible.
    //   (3) legal / copyright  — static literal text
    let footer_version = Heading::new("Marionette v1.1 · Protocol 1.1.0")
        .id("footer-version")
        .build();
    // D-B6 connection-status indicator. The text is populated by the frontend
    // transport layer (Plan 06 Task 4) via an internal
    // `applyPatch('main', [{op:'set', path:'/system/connectionStatus', ...}])`
    // on WebSocket connect/disconnect events. Initial value seeded from
    // shell_data below.
    let footer_status = Heading::new("connected")
        .id("footer-connection-status")
        .bind("/system/connectionStatus")
        .build();
    let footer_legal = Heading::new("© 2026 Marionette")
        .id("footer-legal")
        .build();
    let (footer_root, footer_desc) = Container::new()
        .id("shell-footer")
        .children(vec![footer_version, footer_status, footer_legal])
        .build_tree();

    // -- Build the three sub-surface mounts (D-B8) --
    let content_mount = SurfaceMount::new("content")
        .id("shell-content-mount")
        .build();
    let modal_mount = SurfaceMount::new("modal")
        .id("shell-modal-mount")
        .build();
    let toasts_mount = SurfaceMount::new("toasts")
        .id("shell-toasts-mount")
        .build();

    // -- Assemble the AppShell --
    let mut descendants: Vec<(String, marionette_protocol::Component)> = Vec::new();
    descendants.extend(sidebar_desc);
    descendants.extend(header_desc);
    descendants.extend(footer_desc);

    let shell_nodes = AppShell::new()
        .id("app-shell-root")
        .sidebar(sidebar_root)
        .header(header_root)
        .footer(footer_root)
        .main(content_mount)
        .popups(modal_mount)
        .toasts(toasts_mount)
        .with_descendants(descendants)
        .build_with_children();

    let mut shell_map = HashMap::new();
    for (id, component) in shell_nodes {
        shell_map.insert(id, component);
    }

    // Initial shell data (D-B6 + D-B13):
    //   /auth/currentUser      — future data-bound user menus
    //   /system/connectionStatus — seeds footer indicator on first mount
    //   /nav/active/*          — seeds landing nav state (contacts = true)
    let shell_data = serde_json::json!({
        "auth": {
            "currentUser": {
                "name": user_name,
                "roles": session.roles,
            }
        },
        "system": {
            "connectionStatus": "connected"
        },
        "nav": {
            "active": {
                "home": false,
                "contacts": true,
                "companies": false,
                "users": false,
                "audit": false
            }
        }
    });

    // -- Compose response messages --
    // Order: shell render first (must exist before content patches/renders
    // can reference its nodes), then the content sub-surface render.
    let mut messages = Vec::new();
    messages.push(ProtocolMessage::Render(RenderMessage {
        id: None,
        surface: "main".into(),
        root: "app-shell-root".into(),
        nodes: shell_map,
        data: shell_data,
    }));
    messages.append(&mut content_messages);

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

    // Build form sub-tree: get root tuple for parent + descendants for flat map
    let (form_child, form_descendants) = Form::new()
        .id("login-form")
        .children(vec![email_input, password_input, submit_button])
        .build_tree();

    // Container only gets the heading and the form root as direct children
    let container_nodes = Container::new()
        .id("login-root")
        .children(vec![heading, form_child])
        .build_with_children();

    // Collect ALL flat nodes into the HashMap
    let mut nodes = HashMap::new();
    for (id, component) in container_nodes {
        nodes.insert(id, component);
    }
    for (id, component) in form_descendants {
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

    // Initialize database — connect directly and run CRM migrations only
    // (CRM migrations include all tables; skip marionette::init_db which runs
    // its own session migration that conflicts with the CRM migrator)
    let db = sea_orm::Database::connect("sqlite://crm.db?mode=rwc")
        .await
        .expect("failed to connect to database");

    // Run CRM-specific migrations (includes all tables the app needs)
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

    // Initialize Listmonk client from environment
    let listmonk_client: Option<Arc<listmonk::ListmonkClient>> =
        if let Some(client) = listmonk::ListmonkClient::from_env() {
            if client.validate_connection().await {
                tracing::info!("Listmonk connection validated");
            } else {
                tracing::warn!(
                    "Listmonk configured but unreachable -- sync features will return errors"
                );
            }
            Some(Arc::new(client))
        } else {
            tracing::info!("Listmonk not configured (LISTMONK_URL not set) -- sync features disabled");
            None
        };

    // Initialize global Listmonk client for handler access
    if let Some(ref client) = listmonk_client {
        handlers::listmonk::init_listmonk_client(Arc::clone(client));
    }

    let db = Arc::new(db);

    let action_router = ActionRouter::new()
        .action(
            "login",
            box_handler(handle_login_action),
            AuthRequirement::None,
        )
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
        )
        .action(
            "note_save",
            box_handler(handlers::note::handle_note_save),
            AuthRequirement::Authenticated,
        )
        .action(
            "contact_tag_save",
            box_handler(handlers::contact::handle_contact_tag_save),
            AuthRequirement::Authenticated,
        )
        .action(
            "contact_tag_remove",
            box_handler(handlers::contact::handle_contact_tag_remove),
            AuthRequirement::Authenticated,
        )
        .action(
            "interaction_form",
            box_handler(handlers::interaction::handle_interaction_form),
            AuthRequirement::Authenticated,
        )
        .action(
            "interaction_save",
            box_handler(handlers::interaction::handle_interaction_save),
            AuthRequirement::Authenticated,
        )
        .action(
            "listmonk_sync",
            box_handler(handlers::listmonk::handle_listmonk_sync),
            AuthRequirement::Authenticated,
        )
        .action(
            "listmonk_sync_all",
            box_handler(handlers::listmonk::handle_listmonk_sync_all),
            AuthRequirement::Authenticated,
        )
        .action(
            "listmonk_history_refresh",
            box_handler(handlers::listmonk::handle_listmonk_history_refresh),
            AuthRequirement::Authenticated,
        );

    let state = Arc::new(AppState {
        router: action_router,
        db,
        login_form: Some(build_login_form()),
        listmonk: listmonk_client.map(|c| c as Arc<dyn std::any::Any + Send + Sync>),
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
