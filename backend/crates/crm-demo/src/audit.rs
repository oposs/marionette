use sea_orm::{ActiveValue::NotSet, ActiveValue::Set, DatabaseConnection, EntityTrait};

use crate::entities::audit_log;
use marionette::error::ActionError;

/// Record an audit log entry. Called after successful entity mutations.
///
/// `changes` should be a JSON object like `{"field": {"old": X, "new": Y}}` for updates,
/// the full new entity for creates, or the deleted entity for deletes.
pub async fn record_audit(
    db: &DatabaseConnection,
    user_id: i32,
    table: &str,
    record_id: i32,
    action: &str, // "create", "update", "delete"
    changes: serde_json::Value,
) -> Result<(), ActionError> {
    let entry = audit_log::ActiveModel {
        audit_log_id: NotSet,
        audit_log_user: Set(user_id),
        audit_log_table: Set(table.to_owned()),
        audit_log_record_id: Set(record_id),
        audit_log_action: Set(action.to_owned()),
        audit_log_changes: Set(changes.to_string()),
        audit_log_timestamp: NotSet, // DEFAULT datetime('now')
    };
    audit_log::Entity::insert(entry)
        .exec(db)
        .await
        .map_err(|e| ActionError::Internal(e.to_string()))?;
    Ok(())
}

/// Compute a JSON diff between old and new values for audit logging.
/// Returns `{"field": {"old": old_val, "new": new_val}}` for each changed field.
pub fn compute_changes(
    old: &serde_json::Value,
    new: &serde_json::Value,
) -> serde_json::Value {
    let mut diff = serde_json::Map::new();
    if let (Some(old_obj), Some(new_obj)) = (old.as_object(), new.as_object()) {
        for (key, new_val) in new_obj {
            if let Some(old_val) = old_obj.get(key) {
                if old_val != new_val {
                    diff.insert(
                        key.clone(),
                        serde_json::json!({
                            "old": old_val,
                            "new": new_val
                        }),
                    );
                }
            } else {
                diff.insert(
                    key.clone(),
                    serde_json::json!({
                        "old": null,
                        "new": new_val
                    }),
                );
            }
        }
    }
    serde_json::Value::Object(diff)
}
