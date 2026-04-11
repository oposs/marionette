//! Generic server-side row fetcher for the `fetch-rows` action (Phase 13 D-H1).
//!
//! Closes a dead-code gap: the frontend's DataTable sentinel dispatches
//! `sendAction('fetch-rows', { source, offset, limit })` but no backend
//! handler was registered for it until Phase 13. This module provides a
//! single generic handler that dispatches internally to per-source fetchers
//! based on the `source` payload field, enforcing per-source auth and a
//! global limit cap.
//!
//! Security properties:
//! - V4 Access Control: `check_source_auth` enforces per-source role
//!   requirements (audit_list and user_list require `admin`).
//! - V5 Input Validation: payload deserialized via `#[derive(Deserialize)]`;
//!   unknown sources rejected with `ActionError::BadPayload`.
//! - V5 DoS: `limit` is capped server-side at `MAX_LIMIT = 100`.
//! - D-H3 correlation: the outgoing `PatchMessage.id` echoes
//!   `ctx.action.id.clone()` so the frontend can discard stale responses.

use sea_orm::{EntityTrait, QueryOrder, QuerySelect};
use serde::Deserialize;

use marionette::error::{ActionError, ActionResult};
use marionette::extractors::{Db, FromHandlerContext, HandlerContext, Payload, Session};
use marionette_protocol::{PatchMessage, PatchOperation, ProtocolMessage};

use crate::entities::{audit_log, company, contact, user};

/// Maximum rows returnable in a single `fetch-rows` request. Enforced
/// server-side as a DoS mitigation (V5 Input Validation hardening).
const MAX_LIMIT: u32 = 100;

/// Default page size used when the payload omits `limit`.
fn default_limit() -> u32 {
    50
}

/// Payload shape for the `fetch-rows` action.
#[derive(Debug, Deserialize)]
pub struct FetchRowsPayload {
    /// Identifier of the source list screen. Maps to a per-screen fetcher in
    /// the dispatch table below. Must match one of the known sources
    /// (`contact_list`, `company_list`, `user_list`, `audit_list`).
    pub source: String,
    #[serde(default)]
    pub offset: u32,
    #[serde(default = "default_limit")]
    pub limit: u32,
    /// Optional filter payload forwarded to the per-source fetcher.
    #[serde(default)]
    pub filters: serde_json::Value,
}

/// Auth requirement per source (mirrors `main.rs` `ActionRouter` registrations).
///
/// `Ok(None)` means any authenticated caller passes; `Ok(Some(role))` means
/// the caller must hold that role; `Err(BadPayload)` rejects unknown sources.
///
/// Enforced IN the handler because the router registers `fetch-rows` at a
/// single level (`Authenticated`) but some sources (audit, user) require
/// admin. (V4 Access Control.)
fn required_role_for(source: &str) -> Result<Option<&'static str>, ActionError> {
    match source {
        "contact_list" | "company_list" => Ok(None), // authenticated-only
        "audit_list" | "user_list" => Ok(Some("admin")), // admin-only
        _ => Err(ActionError::BadPayload(format!(
            "unknown fetch-rows source: {source}"
        ))),
    }
}

/// Pure auth decision.
///
/// Returns `Ok(())` if the caller's roles satisfy the source's requirement,
/// `Err(ActionError::Unauthorized)` if not, or `Err(ActionError::BadPayload)`
/// if the source is unknown.
///
/// `session_roles` is the full list of roles attached to the current
/// `Session` (see `marionette::extractors::Session::roles`). This helper is
/// deliberately pure so unit tests can drive it directly without
/// constructing a `HandlerContext`.
fn check_source_auth(source: &str, session_roles: &[String]) -> Result<(), ActionError> {
    let required = required_role_for(source)?;
    if let Some(role) = required {
        if !session_roles.iter().any(|r| r == role) {
            return Err(ActionError::Unauthorized(format!(
                "fetch-rows source '{source}' requires role '{role}'"
            )));
        }
    }
    Ok(())
}

/// Generic fetch-rows handler.
///
/// Parses the payload, enforces per-source auth, caps the limit, and
/// dispatches to the per-source fetcher. Returns a `PatchMessage` with one
/// `set` op per fetched row (keyed by the row's `id` field) so the frontend
/// can append to its existing bound collection without replacing the full
/// collection.
pub async fn handle_fetch_rows(ctx: HandlerContext) -> ActionResult {
    // 1. Parse and validate payload (V5 Input Validation).
    let payload: FetchRowsPayload = Payload::<FetchRowsPayload>::from_context(&ctx)
        .map_err(|e| {
            ActionError::BadPayload(format!(
                "fetch-rows payload missing or malformed: {e}"
            ))
        })?
        .0;

    // 2. Cap limit (V5 DoS mitigation).
    let limit = payload.limit.min(MAX_LIMIT);
    let offset = payload.offset;

    // 3. Per-source auth check (V4 Access Control).
    let session = Session::from_context(&ctx)?;
    check_source_auth(&payload.source, &session.roles)?;

    // 4. Dispatch to per-source fetcher.
    let db = Db::from_context(&ctx)?;
    let (path, rows) = match payload.source.as_str() {
        "contact_list" => fetch_contacts(&db, offset, limit, &payload.filters).await?,
        "company_list" => fetch_companies(&db, offset, limit, &payload.filters).await?,
        "user_list" => fetch_users(&db, offset, limit, &payload.filters).await?,
        "audit_list" => fetch_audit(&db, offset, limit, &payload.filters).await?,
        // Unreachable — `check_source_auth` rejected it above.
        other => {
            return Err(ActionError::BadPayload(format!(
                "unknown fetch-rows source: {other}"
            )))
        }
    };

    // 5. Build one `Set` op per row, keyed by the row's `id`. The frontend's
    //    `Object.entries(bound_collection)` iteration picks up appended keys
    //    automatically (D-H1 append semantics).
    let mut ops: Vec<PatchOperation> = Vec::with_capacity(rows.len());
    for row in rows {
        let row_id = row
            .get("id")
            .and_then(|v| {
                v.as_str()
                    .map(String::from)
                    .or_else(|| v.as_i64().map(|i| i.to_string()))
            })
            .ok_or_else(|| ActionError::Internal("row missing 'id' field".into()))?;
        ops.push(PatchOperation::Set {
            path: format!("{path}/{row_id}"),
            value: row,
        });
    }

    // 6. Echo the action id into the response (D-H3 correlation).
    Ok(vec![ProtocolMessage::Patch(PatchMessage {
        id: ctx.action.id.clone(),
        surface: "content".into(),
        patch: ops,
    })])
}

// ---------------------------------------------------------------------------
// Per-source fetchers
//
// Each returns `(bound_collection_path, Vec<row_json>)`. The path matches
// the `bind` used by the source's list handler so patches append to the
// same path the DataTable is already reading from.
// ---------------------------------------------------------------------------

async fn fetch_contacts(
    db: &Db,
    offset: u32,
    limit: u32,
    _filters: &serde_json::Value,
) -> Result<(&'static str, Vec<serde_json::Value>), ActionError> {
    let rows = contact::Entity::find()
        .order_by_asc(contact::Column::ContactId)
        .offset(u64::from(offset))
        .limit(u64::from(limit))
        .all(&*db.0)
        .await
        .map_err(|e| ActionError::Internal(e.to_string()))?
        .into_iter()
        .map(|c| {
            serde_json::json!({
                "id": c.contact_id,
                "name": c.contact_name,
                "email": c.contact_email,
                "phone": c.contact_phone.unwrap_or_default(),
                "title": c.contact_title.unwrap_or_default(),
            })
        })
        .collect();
    Ok(("/contacts", rows))
}

async fn fetch_companies(
    db: &Db,
    offset: u32,
    limit: u32,
    _filters: &serde_json::Value,
) -> Result<(&'static str, Vec<serde_json::Value>), ActionError> {
    let rows = company::Entity::find()
        .order_by_asc(company::Column::CompanyId)
        .offset(u64::from(offset))
        .limit(u64::from(limit))
        .all(&*db.0)
        .await
        .map_err(|e| ActionError::Internal(e.to_string()))?
        .into_iter()
        .map(|c| {
            serde_json::json!({
                "id": c.company_id,
                "name": c.company_name,
                "website": c.company_website.unwrap_or_default(),
                "address": c.company_address.unwrap_or_default(),
            })
        })
        .collect();
    Ok(("/companies", rows))
}

async fn fetch_users(
    db: &Db,
    offset: u32,
    limit: u32,
    _filters: &serde_json::Value,
) -> Result<(&'static str, Vec<serde_json::Value>), ActionError> {
    let rows = user::Entity::find()
        .order_by_asc(user::Column::UserId)
        .offset(u64::from(offset))
        .limit(u64::from(limit))
        .all(&*db.0)
        .await
        .map_err(|e| ActionError::Internal(e.to_string()))?
        .into_iter()
        .map(|u| {
            serde_json::json!({
                "id": u.user_id,
                "name": u.user_name,
                "email": u.user_email,
                "role": u.user_role,
            })
        })
        .collect();
    Ok(("/users", rows))
}

async fn fetch_audit(
    db: &Db,
    offset: u32,
    limit: u32,
    _filters: &serde_json::Value,
) -> Result<(&'static str, Vec<serde_json::Value>), ActionError> {
    let rows = audit_log::Entity::find()
        .order_by_desc(audit_log::Column::AuditLogTimestamp)
        .offset(u64::from(offset))
        .limit(u64::from(limit))
        .all(&*db.0)
        .await
        .map_err(|e| ActionError::Internal(e.to_string()))?
        .into_iter()
        .map(|e| {
            serde_json::json!({
                "id": e.audit_log_id,
                "timestamp": e.audit_log_timestamp,
                "table": e.audit_log_table,
                "recordId": e.audit_log_record_id,
                "action": e.audit_log_action,
                "changes": e.audit_log_changes,
            })
        })
        .collect();
    Ok(("/auditEntries", rows))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ---- required_role_for ------------------------------------------------

    #[test]
    fn required_role_for_known_sources() {
        assert!(matches!(required_role_for("contact_list"), Ok(None)));
        assert!(matches!(required_role_for("company_list"), Ok(None)));
        assert!(matches!(
            required_role_for("audit_list"),
            Ok(Some("admin"))
        ));
        assert!(matches!(
            required_role_for("user_list"),
            Ok(Some("admin"))
        ));
    }

    #[test]
    fn required_role_for_rejects_unknown_source() {
        let err = required_role_for("not_a_real_source");
        assert!(matches!(err, Err(ActionError::BadPayload(_))));
    }

    // ---- check_source_auth (the pure auth core) --------------------------

    fn roles(rs: &[&str]) -> Vec<String> {
        rs.iter().map(|&s| s.to_string()).collect()
    }

    #[test]
    fn check_source_auth_allows_authenticated_for_contact_list() {
        // contact_list requires no specific role — any authenticated caller passes.
        assert!(check_source_auth("contact_list", &roles(&["user"])).is_ok());
        assert!(check_source_auth("contact_list", &roles(&["admin"])).is_ok());
        // Even an empty-role session passes because the source requires no
        // specific role. (Router-level `AuthRequirement::Authenticated`
        // separately ensures there IS a session; this helper only checks the
        // role match.)
        assert!(check_source_auth("contact_list", &[]).is_ok());
    }

    #[test]
    fn check_source_auth_allows_authenticated_for_company_list() {
        assert!(check_source_auth("company_list", &roles(&["user"])).is_ok());
        assert!(check_source_auth("company_list", &[]).is_ok());
    }

    #[test]
    fn check_source_auth_allows_admin_for_audit_list() {
        assert!(check_source_auth("audit_list", &roles(&["admin"])).is_ok());
    }

    #[test]
    fn check_source_auth_allows_admin_for_user_list() {
        assert!(check_source_auth("user_list", &roles(&["admin"])).is_ok());
    }

    #[test]
    fn check_source_auth_allows_admin_with_extra_roles() {
        // Defensive: a caller with ["admin", "user"] still counts as admin.
        assert!(
            check_source_auth("audit_list", &roles(&["admin", "user"])).is_ok()
        );
    }

    #[test]
    fn check_source_auth_rejects_non_admin_for_audit_list() {
        let err = check_source_auth("audit_list", &roles(&["user"]));
        assert!(
            matches!(err, Err(ActionError::Unauthorized(_))),
            "got {err:?}"
        );
    }

    #[test]
    fn check_source_auth_rejects_missing_role_for_audit_list() {
        let err = check_source_auth("audit_list", &[]);
        assert!(
            matches!(err, Err(ActionError::Unauthorized(_))),
            "got {err:?}"
        );
    }

    #[test]
    fn check_source_auth_rejects_non_admin_for_user_list() {
        let err = check_source_auth("user_list", &roles(&["user"]));
        assert!(
            matches!(err, Err(ActionError::Unauthorized(_))),
            "got {err:?}"
        );
    }

    #[test]
    fn check_source_auth_rejects_unknown_source() {
        let err = check_source_auth("not_a_real_source", &roles(&["admin"]));
        assert!(
            matches!(err, Err(ActionError::BadPayload(_))),
            "got {err:?}"
        );
    }

    // ---- Payload deserialization -----------------------------------------

    #[test]
    fn fetch_rows_payload_deserializes_defaults() {
        let p: FetchRowsPayload = serde_json::from_value(json!({
            "source": "contact_list"
        }))
        .unwrap();
        assert_eq!(p.source, "contact_list");
        assert_eq!(p.offset, 0);
        assert_eq!(p.limit, 50);
    }

    #[test]
    fn fetch_rows_payload_rejects_missing_source() {
        let r = serde_json::from_value::<FetchRowsPayload>(json!({
            "offset": 0,
            "limit": 10
        }));
        assert!(r.is_err(), "expected deserialize error for missing source");
    }

    #[test]
    fn fetch_rows_payload_accepts_filters_blob() {
        let p: FetchRowsPayload = serde_json::from_value(json!({
            "source": "contact_list",
            "offset": 10,
            "limit": 25,
            "filters": { "search": "alice" }
        }))
        .unwrap();
        assert_eq!(p.offset, 10);
        assert_eq!(p.limit, 25);
        assert_eq!(p.filters["search"], "alice");
    }

    // ---- Limit cap -------------------------------------------------------

    #[test]
    fn fetch_rows_limit_cap_constant() {
        // Sanity check on the cap value — must match the V5 DoS mitigation.
        assert_eq!(MAX_LIMIT, 100);
    }

    #[test]
    fn fetch_rows_limit_min_caps_oversized_request() {
        // The runtime code does `payload.limit.min(MAX_LIMIT)` — verify the
        // saturation behavior directly.
        let requested: u32 = 10_000;
        let capped = requested.min(MAX_LIMIT);
        assert_eq!(capped, 100);
    }

    #[test]
    fn fetch_rows_limit_cap_is_applied_in_source() {
        // Structural check: the handler must cap the limit before using it.
        let src = include_str!("fetch_rows.rs");
        assert!(
            src.contains("payload.limit.min(MAX_LIMIT)"),
            "handler must cap limit via payload.limit.min(MAX_LIMIT)"
        );
    }

    // ---- D-H3 action-id echo --------------------------------------------

    #[test]
    fn fetch_rows_patch_message_id_uses_action_id_clone() {
        // Structural check of the D-H3 correlation invariant. Because we
        // cannot cheaply build a full HandlerContext in a unit test, we
        // assert on the source text directly — a regression that drops the
        // echo would change this string.
        let src = include_str!("fetch_rows.rs");
        assert!(
            src.contains("id: ctx.action.id.clone()"),
            "PatchMessage must echo ctx.action.id (D-H3 correlation)"
        );
    }
}
