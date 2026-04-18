use sea_orm::{ActiveModelTrait, ActiveValue::{NotSet, Set}};
use serde::Deserialize;

use marionette::error::{ActionError, ActionResult};
use marionette::extractors::{Db, FromHandlerContext, HandlerContext, Payload, Session};
use marionette_protocol::messages::ActionMessage;

use crate::entities::note;

/// Inner form data for a note (create only -- notes are append-only).
#[derive(Deserialize)]
struct NoteFormData {
    contact_id: Option<i32>,
    company_id: Option<i32>,
    text: String,
}

/// Payload wrapper: the frontend sends all surface data, with form fields
/// nested under the form's bind prefix (e.g. `noteForm`).
#[derive(Deserialize)]
struct NoteSavePayload {
    #[serde(rename = "noteForm")]
    note_form: NoteFormData,
}

/// Handle the `note_save` action: create a new note attached to a contact or company.
pub async fn handle_note_save(ctx: HandlerContext) -> ActionResult {
    let db = Db::from_context(&ctx)?;
    let session = Session::from_context(&ctx)?;
    let payload = Payload::<NoteSavePayload>::from_context(&ctx)?;
    let data = payload.0.note_form;

    // Validate text is not empty
    if data.text.trim().is_empty() {
        return Err(ActionError::BadPayload("Note text is required".into()));
    }

    // Validate exactly one of contact_id or company_id is provided
    match (data.contact_id, data.company_id) {
        (Some(_), Some(_)) => {
            return Err(ActionError::BadPayload(
                "Note must be attached to either a contact or a company, not both".into(),
            ));
        }
        (None, None) => {
            return Err(ActionError::BadPayload(
                "Note must be attached to a contact or a company".into(),
            ));
        }
        _ => {}
    }

    let caller_id: i32 = session
        .user_id
        .as_ref()
        .and_then(|id| id.parse().ok())
        .unwrap_or(0);

    // Insert the note
    let new_note = note::ActiveModel {
        note_id: NotSet,
        note_contact: Set(data.contact_id),
        note_company: Set(data.company_id),
        note_text: Set(data.text.clone()),
        note_user: Set(caller_id),
        note_created_at: NotSet, // DB default datetime('now')
    };
    let inserted = new_note
        .insert(&*db.0)
        .await
        .map_err(|e| ActionError::Internal(e.to_string()))?;

    // Record audit entry
    let audit_changes = serde_json::json!({
        "text": &data.text,
        "contact_id": data.contact_id,
        "company_id": data.company_id,
    });
    crate::audit::record_audit(
        &*db.0,
        caller_id,
        "note",
        inserted.note_id,
        "create",
        audit_changes,
    )
    .await?;

    // Re-render the parent form by delegating to the appropriate handler
    if let Some(cid) = data.contact_id {
        let new_ctx = HandlerContext {
            action: ActionMessage {
                id: ctx.action.id.clone(),
                name: "contact_edit".into(),
                source: None,
                payload: Some(serde_json::json!({ "contact_id": cid })),
                optimistic: None,
            },
            db: ctx.db.clone(),
            session: ctx.session.clone(),
        };
        super::contact::handle_contact_form(new_ctx).await
    } else if let Some(cid) = data.company_id {
        let new_ctx = HandlerContext {
            action: ActionMessage {
                id: ctx.action.id.clone(),
                name: "company_edit".into(),
                source: None,
                payload: Some(serde_json::json!({ "company_id": cid })),
                optimistic: None,
            },
            db: ctx.db.clone(),
            session: ctx.session.clone(),
        };
        super::company::handle_company_form(new_ctx).await
    } else {
        // Should never reach here due to validation above
        Err(ActionError::Internal("Unexpected state".into()))
    }
}

#[cfg(test)]
mod tests {
    //! Phase 15 Plan 04 Task 2 — RED/GREEN gate for note.rs validation rewiring.
    //!
    //! Source-grep assertion over the handler module source: the empty-body
    //! branch MUST use `validation_error_patch` at the `/noteForm/text` bind
    //! path rather than `ActionError::BadPayload`.

    const THIS_FILE: &str = include_str!("note.rs");

    #[test]
    fn note_save_uses_validation_error_patch_for_empty_body() {
        // D-D1: per-field /_errors/noteForm/text patch instead of form-level BadPayload.
        assert!(
            THIS_FILE.contains("validation_error_patch("),
            "expected validation_error_patch(...) in handle_note_save"
        );
        assert!(
            THIS_FILE.contains("/noteForm/text"),
            "expected /noteForm/text bind path (matches TextInput.bind(...) in contact.rs)"
        );
        // Old empty-text BadPayload branch must be gone.
        assert!(
            !THIS_FILE.contains("BadPayload(\"Note text is required\""),
            "old BadPayload(\"Note text is required\") must be replaced"
        );
    }
}
