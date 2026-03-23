use std::collections::HashMap;

use sea_orm::{ActiveModelTrait, ActiveValue::{NotSet, Set}};
use serde::Deserialize;

use marionette::builders::standard::{
    Button, Container, Form, Heading, Select, SelectOption, TextInput,
};
use marionette::error::{ActionError, ActionResult};
use marionette::extractors::{Db, FromHandlerContext, HandlerContext, Payload, Session};
use marionette_protocol::messages::ActionMessage;
use marionette_protocol::{ComponentAction, ProtocolMessage, RenderMessage};

use crate::entities::interaction;

/// Format current UTC time as SQLite datetime string.
fn now_sqlite() -> String {
    let now = time::OffsetDateTime::now_utc();
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        now.year(),
        now.month() as u8,
        now.day(),
        now.hour(),
        now.minute(),
        now.second()
    )
}

/// Payload for opening the interaction form.
#[derive(Deserialize)]
struct InteractionFormPayload {
    contact_id: i32,
}

/// Inner form data for saving an interaction.
#[derive(Deserialize)]
struct InteractionFormData {
    contact_id: i32,
    interaction_type: String,
    subject: String,
    date: String,
    notes: Option<String>,
}

/// Payload wrapper: the frontend sends all surface data, with form fields
/// nested under the form's bind prefix (e.g. `interactionForm`).
#[derive(Deserialize)]
struct InteractionSavePayload {
    #[serde(rename = "interactionForm")]
    interaction_form: InteractionFormData,
}

/// Handle the `interaction_form` action: render a form for logging an interaction.
pub async fn handle_interaction_form(ctx: HandlerContext) -> ActionResult {
    let payload = Payload::<InteractionFormPayload>::from_context(&ctx)?;
    let contact_id = payload.0.contact_id;

    let heading = Heading::new("Log Interaction")
        .id("interaction-form-heading")
        .build();

    let type_select = Select::new(
        "Type",
        vec![
            SelectOption {
                value: "call".into(),
                label: "Call".into(),
            },
            SelectOption {
                value: "email".into(),
                label: "Email".into(),
            },
            SelectOption {
                value: "meeting".into(),
                label: "Meeting".into(),
            },
        ],
    )
    .id("interaction-type")
    .bind("/interactionForm/interaction_type")
    .build();

    let subject_input = TextInput::new("Subject")
        .id("interaction-subject")
        .bind("/interactionForm/subject")
        .build();

    let date_input = TextInput::new("Date (YYYY-MM-DD HH:MM)")
        .id("interaction-date")
        .bind("/interactionForm/date")
        .build();

    let notes_input = TextInput::new("Notes")
        .id("interaction-notes")
        .bind("/interactionForm/notes")
        .build();

    let save_button = Button::new("Save Interaction")
        .id("interaction-save-btn")
        .action(ComponentAction::submit("interaction_save"))
        .build();

    let cancel_button = Button::new("Cancel")
        .id("interaction-cancel")
        .action(ComponentAction::click("contact_list"))
        .build();

    let (form_child, form_descendants) = Form::new()
        .id("interaction-form")
        .children(vec![
            type_select,
            subject_input,
            date_input,
            notes_input,
            save_button,
            cancel_button,
        ])
        .build_tree();

    let all_nodes = vec![heading, form_child];

    let container_nodes = Container::new()
        .id("interaction-form-root")
        .children(all_nodes)
        .build_with_children();

    let mut nodes = HashMap::new();
    for (id, component) in container_nodes {
        nodes.insert(id, component);
    }
    for (id, component) in form_descendants {
        nodes.insert(id, component);
    }

    let data = serde_json::json!({
        "interactionForm": {
            "contact_id": contact_id,
            "interaction_type": "call",
            "subject": "",
            "date": now_sqlite(),
            "notes": ""
        }
    });

    Ok(vec![ProtocolMessage::Render(RenderMessage {
        id: ctx.action.id.clone(),
        surface: "main".into(),
        root: "interaction-form-root".into(),
        nodes,
        data,
    })])
}

/// Handle the `interaction_save` action: validate, insert, audit, and re-render contact form.
pub async fn handle_interaction_save(ctx: HandlerContext) -> ActionResult {
    let db = Db::from_context(&ctx)?;
    let session = Session::from_context(&ctx)?;
    let payload = Payload::<InteractionSavePayload>::from_context(&ctx)?;
    let data = payload.0.interaction_form;

    // Validate required fields
    if data.subject.trim().is_empty() {
        return Err(ActionError::BadPayload("Subject is required".into()));
    }
    if data.date.trim().is_empty() {
        return Err(ActionError::BadPayload("Date is required".into()));
    }
    if !["call", "email", "meeting"].contains(&data.interaction_type.as_str()) {
        return Err(ActionError::BadPayload(
            "Type must be call, email, or meeting".into(),
        ));
    }

    let caller_id: i32 = session
        .user_id
        .as_ref()
        .and_then(|id| id.parse().ok())
        .unwrap_or(0);

    let new_interaction = interaction::ActiveModel {
        interaction_id: NotSet,
        interaction_contact: Set(data.contact_id),
        interaction_type: Set(data.interaction_type.clone()),
        interaction_subject: Set(data.subject.clone()),
        interaction_notes: Set(data.notes.clone()),
        interaction_user: Set(caller_id),
        interaction_date: Set(data.date.clone()),
        interaction_created_at: NotSet,
    };
    let inserted = new_interaction
        .insert(&*db.0)
        .await
        .map_err(|e| ActionError::Internal(e.to_string()))?;

    // Record audit entry
    let audit_changes = serde_json::json!({
        "contact_id": data.contact_id,
        "type": &data.interaction_type,
        "subject": &data.subject,
        "date": &data.date,
        "notes": &data.notes,
    });
    crate::audit::record_audit(
        &*db.0,
        caller_id,
        "interaction",
        inserted.interaction_id,
        "create",
        audit_changes,
    )
    .await?;

    // Re-render the contact form by delegating to the contact handler
    let new_ctx = HandlerContext {
        action: ActionMessage {
            id: ctx.action.id.clone(),
            name: "contact_edit".into(),
            source: None,
            payload: Some(serde_json::json!({ "contact_id": data.contact_id })),
            optimistic: None,
        },
        db: ctx.db.clone(),
        session: ctx.session.clone(),
    };
    super::contact::handle_contact_form(new_ctx).await
}
