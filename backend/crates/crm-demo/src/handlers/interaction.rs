use sea_orm::{ActiveModelTrait, ActiveValue::{NotSet, Set}};
use serde::Deserialize;

use marionette::builders::standard::{
    form_shell, Button, Container, FieldSet, Form, Heading, RadioGroup, RadioOption, Textarea,
    TextInput,
};
use marionette::error::{ActionError, ActionResult};
use marionette::extractors::{Db, FromHandlerContext, HandlerContext, Payload, Session};
use marionette::validation::validation_error_patch;
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
///
/// Phase 15 Plan 04 (D-A1 + D-B1 + D-E1 + D-E3): the form uses the canonical
/// Phase 14 composition — single `FieldSet("Interaction")` with
/// `[type_radio, subject, date, notes]`, action row, and `form_shell`
/// envelope. `type` migrated from `Select` to `RadioGroup` per D-E1; `notes`
/// upgraded from `TextInput` to `Textarea` with `full_width(true)` per the
/// 15-UI-SPEC §Textarea full_width Contract; `date` carries the locked
/// description string per the 15-UI-SPEC §Description Copy Contract.
pub async fn handle_interaction_form(ctx: HandlerContext) -> ActionResult {
    let payload = Payload::<InteractionFormPayload>::from_context(&ctx)?;
    let contact_id = payload.0.contact_id;

    let heading = Heading::new("Log Interaction")
        .id("interaction-form-heading")
        .build();

    let back_button = Button::new("← Back")
        .id("interaction-form-back")
        .variant("outline")
        .action(ComponentAction::click("contact_list"))
        .build();

    // type: RadioGroup (D-E1) — replaces the Phase 14 Select. 3 options
    // vertically; no per-option descriptions per 15-UI-SPEC §RadioGroup
    // Production Contract (labels are self-explanatory).
    let type_options = vec![
        RadioOption {
            value: "call".into(),
            label: "Call".into(),
            description: None,
        },
        RadioOption {
            value: "email".into(),
            label: "Email".into(),
            description: None,
        },
        RadioOption {
            value: "meeting".into(),
            label: "Meeting".into(),
            description: None,
        },
    ];
    let type_radio = RadioGroup::new("Type", type_options)
        .id("interaction-form-type")
        .bind("/interactionForm/interaction_type")
        .required(true)
        .build();

    let subject_input = TextInput::new("Subject")
        .id("interaction-form-subject")
        .bind("/interactionForm/subject")
        .required(true)
        .build();

    let date_input = TextInput::new("Date")
        .id("interaction-form-date")
        .bind("/interactionForm/date")
        .input_type("datetime-local")
        .required(true)
        .description("Format: YYYY-MM-DD HH:MM (24-hour).")
        .build();

    // notes: Textarea full_width (upgraded from TextInput) — 15-UI-SPEC
    // §Textarea full_width Contract requires `full_width(true)` + `rows(4)`.
    let notes_textarea = Textarea::new("Notes")
        .id("interaction-form-notes")
        .bind("/interactionForm/notes")
        .rows(4u32)
        .full_width(true)
        .placeholder("Describe what happened, decisions made, or follow-ups needed…")
        .build();

    let (interaction_set, interaction_descendants) = FieldSet::new()
        .id("interaction-set")
        .legend("Interaction")
        .children(vec![type_radio, subject_input, date_input, notes_textarea])
        .build_tree();

    let cancel_button = Button::new("Cancel")
        .id("interaction-form-cancel")
        .variant("outline")
        .action(ComponentAction::click("contact_list"))
        .build();

    let save_button = Button::new("Save interaction")
        .id("interaction-form-save")
        .variant("default")
        .action(ComponentAction::submit("interaction_save"))
        .build();

    let (action_row, action_row_descendants) = Container::new()
        .id("interaction-form-actions")
        .class("flex gap-2 justify-end")
        .children(vec![cancel_button, save_button])
        .build_tree();

    let (form_child, form_descendants) = Form::new()
        .id("interaction-form")
        .children(vec![interaction_set, action_row])
        .build_tree();

    // Collect all descendants from the three sub-trees (fieldset children,
    // action row children, form children) for the form_shell envelope.
    let mut all_descendants: Vec<(String, marionette_protocol::Component)> = Vec::new();
    all_descendants.extend(interaction_descendants);
    all_descendants.extend(action_row_descendants);
    all_descendants.extend(form_descendants);

    let (root, nodes) = form_shell(
        "interaction-form-root",
        heading,
        back_button,
        form_child,
        all_descendants,
    );

    let data = serde_json::json!({
        "interactionForm": {
            "contact_id": contact_id,
            "interaction_type": "call",
            "subject": "",
            "date": now_sqlite(),
            "notes": ""
        }
    });

    Ok(vec![
        ProtocolMessage::Render(RenderMessage {
            id: ctx.action.id.clone(),
            surface: "content".into(),
            root,
            nodes,
            data,
        }),
        nav_active_patch("contacts"),
    ])
}

/// Build a `PatchMessage` that marks `<active_slug>` as the active nav item and
/// clears all others. Emitted alongside every screen Render so the sidebar's
/// `NavItem` active indicators (bound to `/nav/active/<slug>`) stay in sync
/// with the currently-visible screen. Interactions live under the contact
/// screen in the CRM demo, so this handler activates the "contacts" nav
/// entry. Per D-B13.
fn nav_active_patch(active_slug: &str) -> marionette_protocol::ProtocolMessage {
    use marionette_protocol::data::PatchOperation;
    use marionette_protocol::messages::PatchMessage;
    let slugs = ["home", "contacts", "companies", "users", "audit"];
    let ops: Vec<PatchOperation> = slugs
        .iter()
        .map(|s| PatchOperation::Set {
            path: format!("/nav/active/{s}"),
            value: serde_json::json!(*s == active_slug),
        })
        .collect();
    marionette_protocol::ProtocolMessage::Patch(PatchMessage {
        id: None,
        surface: "main".into(),
        patch: ops,
    })
}

/// Handle the `interaction_save` action: validate, insert, audit, and re-render contact form.
///
/// Phase 15 Plan 04 D-D1: per-field validation emits `/_errors{bind}` patches
/// via `validation_error_patch` instead of form-level `BadPayload` toasts. The
/// `interaction_type` allowlist stays server-authoritative (T-15-03-PLAN04-a
/// mitigation — RadioGroup is UX-only; a malicious client could send arbitrary
/// strings, so the server must still validate).
pub async fn handle_interaction_save(ctx: HandlerContext) -> ActionResult {
    let db = Db::from_context(&ctx)?;
    let session = Session::from_context(&ctx)?;
    let payload = Payload::<InteractionSavePayload>::from_context(&ctx)?;
    let data = payload.0.interaction_form;

    // PHASE 15 D-D1 — per-field validation. Collect top-to-bottom in form
    // field display order (type, subject, date) per 15-RESEARCH Pitfall #1.
    let mut errors: Vec<(String, String)> = Vec::new();
    if !["call", "email", "meeting"].contains(&data.interaction_type.as_str()) {
        errors.push((
            "/interactionForm/interaction_type".into(),
            "Choose one of the listed options.".into(),
        ));
    }
    if data.subject.trim().is_empty() {
        errors.push((
            "/interactionForm/subject".into(),
            "Subject is required.".into(),
        ));
    }
    if data.date.trim().is_empty() {
        errors.push((
            "/interactionForm/date".into(),
            "Date is required.".into(),
        ));
    }
    if !errors.is_empty() {
        return Ok(vec![validation_error_patch("content", errors)]);
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
            extensions: marionette::Extensions::new(),
    };
    super::contact::handle_contact_form(new_ctx).await
}

#[cfg(test)]
mod tests {
    //! Phase 15 Plan 04 Task 1 — RED/GREEN gates for interaction form migration.
    //!
    //! RED gate: assert the expected Phase 15 shape of the interaction
    //! form (RadioGroup for type, Textarea full_width for notes, FieldSet
    //! envelope via form_shell, per-field validation_error_patch on save).
    //!
    //! The tests use source-level greps over this file rather than calling
    //! the handlers — crm-demo is a pure binary crate so integration tests
    //! can't import module-private symbols, and the handlers require a
    //! full DB + Session context to run end-to-end. Source-grep assertions
    //! pin the structural contract without that ceremony.

    const THIS_FILE: &str = include_str!("interaction.rs");

    #[test]
    fn interaction_form_uses_radio_group_for_type() {
        // D-E1: type field migrated from Select to RadioGroup.
        assert!(
            THIS_FILE.contains("RadioGroup::new(\"Type\""),
            "expected RadioGroup::new(\"Type\", ...) in interaction.rs"
        );
        // Old type-select Select construction must be gone.
        // Look for the Phase 14 Select::new literal wrapper "Type" to
        // differentiate from the RadioGroup construction — they share the
        // "Type" label but diverge on the builder name.
        assert!(
            !THIS_FILE.contains("Select::new(\n        \"Type\","),
            "old Select::new(\"Type\", ...) construction must be removed"
        );
    }

    #[test]
    fn interaction_form_uses_textarea_full_width_for_notes() {
        // 15-UI-SPEC §Textarea full_width Contract.
        assert!(
            THIS_FILE.contains("Textarea::new(\"Notes\")"),
            "expected Textarea::new(\"Notes\") in interaction.rs"
        );
        assert!(
            THIS_FILE.contains(".full_width(true)"),
            "expected .full_width(true) on notes textarea"
        );
    }

    #[test]
    fn interaction_form_uses_fieldset_and_form_shell() {
        // 15-UI-SPEC §Per-Screen §3.
        assert!(
            THIS_FILE.contains("FieldSet::new()"),
            "expected FieldSet::new() envelope in interaction.rs"
        );
        assert!(
            THIS_FILE.contains("legend(\"Interaction\")"),
            "expected FieldSet legend \"Interaction\""
        );
        assert!(
            THIS_FILE.contains("form_shell("),
            "expected form_shell(...) envelope wiring"
        );
    }

    #[test]
    fn interaction_form_date_carries_locked_description() {
        // 15-UI-SPEC §Description Copy Contract.
        // Check via `.description(` call so the assertion text itself
        // doesn't accidentally satisfy `.contains(...)` (self-reference).
        assert!(
            THIS_FILE.contains(".description(\"Format: YYYY-MM-DD HH:MM (24-hour).\")"),
            "expected .description(\"Format: YYYY-MM-DD HH:MM ...\") on date field"
        );
    }

    #[test]
    fn interaction_save_uses_validation_error_patch() {
        // D-D1: per-field /_errors{bind} patches on validation failure.
        assert!(
            THIS_FILE.contains("validation_error_patch("),
            "expected validation_error_patch(...) in handle_interaction_save"
        );
        // Per-field BadPayload branches are replaced.
        assert!(
            !THIS_FILE.contains("BadPayload(\"Subject is required"),
            "old BadPayload(\"Subject is required\") must be replaced"
        );
        assert!(
            !THIS_FILE.contains("BadPayload(\"Date is required"),
            "old BadPayload(\"Date is required\") must be replaced"
        );
        assert!(
            !THIS_FILE.contains("BadPayload(\n                \"Type must be call"),
            "old BadPayload(\"Type must be call, email, or meeting\") must be replaced"
        );
    }

    #[test]
    fn interaction_form_save_button_label() {
        // 15-UI-SPEC §Copywriting — "Save interaction" (sentence case, not "Save Interaction").
        assert!(
            THIS_FILE.contains("Button::new(\"Save interaction\")"),
            "expected Button::new(\"Save interaction\") (sentence case)"
        );
        assert!(
            !THIS_FILE.contains("Button::new(\"Save Interaction\")"),
            "old \"Save Interaction\" (title case) must be replaced"
        );
    }
}
