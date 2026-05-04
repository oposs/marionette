//! People CRUD handlers.
//!
//! The `#[action]` macro emits a `pub const APP_ADD_PERSON: &str` next to the
//! handler — the action name `"app/add-person"` becomes the const ident
//! `APP_ADD_PERSON` after the macro replaces non-identifier characters with
//! `_` and uppercases. Registration sites and `Button::action(submit(...))`
//! references both reach for the same symbol — no string-literal action
//! names anywhere in the app.

use marionette::error::{ActionError, ActionResult};
use marionette::extractors::HandlerContext;
use marionette_macros::action;
use marionette_protocol::ProtocolMessage;
use marionette_protocol::data::PatchOperation;
use marionette_protocol::messages::{EventMessage, PatchMessage};
use serde::Deserialize;
use uuid::Uuid;

use crate::state::{PeopleStore, Person};

/// Form payload sent by the People page submit. Field names match the
/// TextInput / Select bind paths under `/form`.
#[derive(Debug, Default, Deserialize)]
struct AddPersonPayload {
    #[serde(default)]
    name: String,
    #[serde(default)]
    email: String,
    #[serde(default)]
    country: String,
}

/// Validate that the three form fields are non-empty (after trim).
fn validate(p: &AddPersonPayload) -> Result<(), &'static str> {
    if p.name.trim().is_empty() {
        return Err("Name is required.");
    }
    if p.email.trim().is_empty() {
        return Err("Email is required.");
    }
    if p.country.trim().is_empty() {
        return Err("Country is required.");
    }
    Ok(())
}

/// Toast helper. Toasts ride on `ProtocolMessage::Event` with name `"toast"`
/// (svelte-sonner consumes them at the layout root).
fn toast(message: &str, severity: &str) -> ProtocolMessage {
    ProtocolMessage::Event(EventMessage {
        id: None,
        surface: None,
        name: "toast".into(),
        hint: Some(serde_json::json!({
            "message": message,
            "severity": severity,
        })),
    })
}

#[action(name = "app/add-person")]
pub async fn handle_add_person(ctx: HandlerContext) -> ActionResult {
    let store = ctx
        .extensions
        .get_arc::<PeopleStore>()
        .ok_or_else(|| ActionError::Internal("PeopleStore not registered".into()))?;

    let payload: AddPersonPayload = ctx
        .action
        .payload
        .clone()
        .map(serde_json::from_value)
        .transpose()
        .map_err(|e| ActionError::BadPayload(e.to_string()))?
        .unwrap_or_default();

    if let Err(message) = validate(&payload) {
        return Ok(vec![toast(message, "error")]);
    }

    let person = Person {
        id: Uuid::new_v4().to_string(),
        name: payload.name.trim().to_string(),
        email: payload.email.trim().to_string(),
        country: payload.country.trim().to_string(),
    };
    store.add(person).await;
    let snapshot = store.snapshot().await;

    Ok(vec![
        ProtocolMessage::Patch(PatchMessage {
            id: ctx.action.id.clone(),
            surface: "content".into(),
            patch: vec![
                PatchOperation::Set {
                    path: "/people".into(),
                    value: serde_json::to_value(&snapshot)
                        .unwrap_or(serde_json::Value::Array(vec![])),
                },
                PatchOperation::Set {
                    path: "/form/name".into(),
                    value: serde_json::Value::String(String::new()),
                },
                PatchOperation::Set {
                    path: "/form/email".into(),
                    value: serde_json::Value::String(String::new()),
                },
                PatchOperation::Set {
                    path: "/form/country".into(),
                    value: serde_json::Value::String(String::new()),
                },
            ],
        }),
        toast("Person added.", "success"),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::Arc;

    use marionette::extensions::Extensions;
    use marionette::extractors::Session;
    use marionette_protocol::ActionMessage;
    use sea_orm::{DatabaseBackend, MockDatabase};

    fn ctx_with(payload: serde_json::Value, extensions: Extensions) -> HandlerContext {
        HandlerContext {
            action: ActionMessage {
                id: Some("t1".into()),
                name: "app/add-person".into(),
                source: None,
                payload: Some(payload),
                optimistic: None,
            },
            db: Arc::new(MockDatabase::new(DatabaseBackend::Sqlite).into_connection()),
            session: Session {
                user_id: None,
                roles: vec![],
            },
            extensions,
        }
    }

    #[tokio::test]
    async fn missing_name_returns_toast_only() {
        let exts = Extensions::new().with(PeopleStore::new());
        let store = exts.get_arc::<PeopleStore>().unwrap();
        let msgs = handle_add_person(ctx_with(
            serde_json::json!({ "name": "", "email": "x@y", "country": "ch" }),
            exts,
        ))
        .await
        .unwrap();
        assert_eq!(msgs.len(), 1);
        assert!(matches!(msgs[0], ProtocolMessage::Event(_)));
        assert!(store.snapshot().await.is_empty());
    }

    #[tokio::test]
    async fn valid_input_appends_and_clears_form() {
        let exts = Extensions::new().with(PeopleStore::new());
        let store = exts.get_arc::<PeopleStore>().unwrap();
        let msgs = handle_add_person(ctx_with(
            serde_json::json!({ "name": "Ada", "email": "ada@x", "country": "uk" }),
            exts,
        ))
        .await
        .unwrap();
        // Patch + success toast.
        assert_eq!(msgs.len(), 2);
        let saved = store.snapshot().await;
        assert_eq!(saved.len(), 1);
        assert_eq!(saved[0].name, "Ada");
    }
}
