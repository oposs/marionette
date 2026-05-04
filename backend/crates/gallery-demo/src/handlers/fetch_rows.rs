//! fetch-rows action handler (Phase 13 + Phase 17 compatible; Phase 18 extended).
//!
//! Source dispatch: `demo-rows` keeps the existing 5-row data-table leaf demo;
//! `catalog-synthetic-rows` serves paginated slices of `fixtures::synthetic_rows(500)`
//! to CAT-03 DataTable (Plan 18-06). Unknown sources produce BadPayload.

use marionette::error::{ActionError, ActionResult};
use marionette::extractors::HandlerContext;
use marionette_protocol::ProtocolMessage;
use marionette_protocol::data::PatchOperation;
use marionette_protocol::messages::PatchMessage;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct FetchRowsPayload {
    source: String,
    #[serde(default)]
    offset: u32,
    #[serde(default = "default_limit")]
    limit: u32,
}

fn default_limit() -> u32 {
    50
}

#[allow(clippy::unused_async, clippy::missing_panics_doc)]
pub async fn handle_demo_fetch_rows(ctx: HandlerContext) -> ActionResult {
    let payload: FetchRowsPayload = serde_json::from_value(
        ctx.action.payload.clone().unwrap_or_default(),
    )
    .map_err(|e| ActionError::BadPayload(format!("fetch-rows payload invalid: {e}")))?;

    let (path_prefix, rows): (&str, Vec<serde_json::Value>) = match payload.source.as_str() {
        "demo-rows" => ("/demo/data-table/rows", demo_rows_legacy()),
        "catalog-synthetic-rows" => {
            let all = crate::fixtures::synthetic_rows(500);
            let start = payload.offset as usize;
            let end = start
                .saturating_add(payload.limit as usize)
                .min(all.len());
            let slice = all.get(start..end).unwrap_or(&[]);
            let json_rows: Vec<serde_json::Value> = slice
                .iter()
                .map(|r| {
                    let mut v = serde_json::to_value(r).expect("Row serializes");
                    v["actions"] = serde_json::json!([
                        { "label": "Edit",      "action": { "type": "click", "name": "gallery-demo/noop" } },
                        { "label": "Delete",    "action": { "type": "click", "name": "gallery-demo/noop" } },
                        { "label": "Duplicate", "action": { "type": "click", "name": "gallery-demo/noop" } },
                    ]);
                    v
                })
                .collect();
            ("/demo/catalog-data-table/rows", json_rows)
        }
        "exer-03-synthetic" => {
            // Phase 19 Plan 19-01: 10_000 row pool for EXER-03 pathological scale
            // (19-RESEARCH.md §Pattern 4). Mirrors catalog-synthetic-rows' shape
            // (same Row struct + injected actions array) but against a 20x-larger
            // generator and at bind path /demo/exer-03/rows.
            let all = crate::fixtures::synthetic_rows(10_000);
            let start = payload.offset as usize;
            let end = start
                .saturating_add(payload.limit as usize)
                .min(all.len());
            let slice = all.get(start..end).unwrap_or(&[]);
            let json_rows: Vec<serde_json::Value> = slice
                .iter()
                .map(|r| {
                    let mut v = serde_json::to_value(r).expect("Row serializes");
                    v["actions"] = serde_json::json!([
                        { "label": "Edit",      "action": { "type": "click", "name": "gallery-demo/noop" } },
                        { "label": "Delete",    "action": { "type": "click", "name": "gallery-demo/noop" } },
                        { "label": "Duplicate", "action": { "type": "click", "name": "gallery-demo/noop" } },
                    ]);
                    v
                })
                .collect();
            ("/demo/exer-03/rows", json_rows)
        }
        other => {
            return Err(ActionError::BadPayload(format!(
                "unknown fetch-rows source: {other}"
            )));
        }
    };

    let ops: Vec<PatchOperation> = rows
        .into_iter()
        .filter_map(|row| {
            let id = row.get("id")?.as_u64()?;
            Some(PatchOperation::Set {
                path: format!("{path_prefix}/{id}"),
                value: row,
            })
        })
        .collect();

    Ok(vec![ProtocolMessage::Patch(PatchMessage {
        id: ctx.action.id.clone(),
        surface: "content".into(),
        patch: ops,
    })])
}

// Keep the existing 5-row set available for the `data-table` leaf demo that
// Phase 17 wired to `"demo-rows"`. Do NOT touch seed_table_rows in show.rs
// (D-4-C locks it untouched); this function duplicates the shape locally.
fn demo_rows_legacy() -> Vec<serde_json::Value> {
    vec![
        serde_json::json!({"id": 1, "name": "Alice Baker", "email": "alice@example.com", "created": "2026-01-05"}),
        serde_json::json!({"id": 2, "name": "Bob Chen",    "email": "bob@example.com",   "created": "2026-01-08"}),
        serde_json::json!({"id": 3, "name": "Carol Davis", "email": "carol@example.com", "created": "2026-01-12"}),
        serde_json::json!({"id": 4, "name": "Dan Evans",   "email": "dan@example.com",   "created": "2026-01-15"}),
        serde_json::json!({"id": 5, "name": "Eva Frost",   "email": "eva@example.com",   "created": "2026-01-20"}),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::Arc;

    use marionette::extractors::Session;
    use marionette_protocol::ActionMessage;
    use sea_orm::{DatabaseBackend, MockDatabase};

    fn mock_db() -> Arc<sea_orm::DatabaseConnection> {
        Arc::new(MockDatabase::new(DatabaseBackend::Sqlite).into_connection())
    }

    fn anonymous_session() -> Session {
        Session {
            user_id: None,
            roles: vec![],
        }
    }

    fn make_ctx(payload: serde_json::Value) -> HandlerContext {
        HandlerContext {
            action: ActionMessage {
                id: Some("t1".into()),
                name: "fetch-rows".into(),
                source: None,
                payload: Some(payload),
                optimistic: None,
            },
            db: mock_db(),
            session: anonymous_session(),
            extensions: marionette::Extensions::new(),
        }
    }

    #[tokio::test]
    async fn catalog_rows_first_page_50_ids_1_through_50() {
        let ctx = make_ctx(serde_json::json!({
            "source": "catalog-synthetic-rows",
            "offset": 0,
            "limit": 50
        }));
        let result = handle_demo_fetch_rows(ctx).await.expect("ok");
        let ProtocolMessage::Patch(msg) = &result[0] else {
            panic!("expected Patch message");
        };
        assert_eq!(msg.patch.len(), 50);
        let PatchOperation::Set { path, .. } = &msg.patch[0] else {
            panic!("expected Set op at index 0");
        };
        assert_eq!(path, "/demo/catalog-data-table/rows/1");
        let PatchOperation::Set { path, .. } = &msg.patch[49] else {
            panic!("expected Set op at index 49");
        };
        assert_eq!(path, "/demo/catalog-data-table/rows/50");
    }

    #[tokio::test]
    async fn catalog_rows_last_page_offset_450() {
        let ctx = make_ctx(serde_json::json!({
            "source": "catalog-synthetic-rows",
            "offset": 450,
            "limit": 50
        }));
        let result = handle_demo_fetch_rows(ctx).await.expect("ok");
        let ProtocolMessage::Patch(msg) = &result[0] else {
            panic!("expected Patch message");
        };
        assert_eq!(msg.patch.len(), 50);
        let PatchOperation::Set { path, .. } = &msg.patch[49] else {
            panic!("expected Set op at index 49");
        };
        assert_eq!(path, "/demo/catalog-data-table/rows/500");
    }

    #[tokio::test]
    async fn catalog_rows_past_end_returns_empty() {
        let ctx = make_ctx(serde_json::json!({
            "source": "catalog-synthetic-rows",
            "offset": 500,
            "limit": 50
        }));
        let result = handle_demo_fetch_rows(ctx).await.expect("ok");
        let ProtocolMessage::Patch(msg) = &result[0] else {
            panic!("expected Patch message");
        };
        assert!(
            msg.patch.is_empty(),
            "past-end slice must be empty, no panic"
        );
    }

    #[tokio::test]
    async fn catalog_rows_include_actions_array() {
        let ctx = make_ctx(serde_json::json!({
            "source": "catalog-synthetic-rows",
            "offset": 0,
            "limit": 1
        }));
        let result = handle_demo_fetch_rows(ctx).await.expect("ok");
        let ProtocolMessage::Patch(msg) = &result[0] else {
            panic!("expected Patch message");
        };
        let PatchOperation::Set { value, .. } = &msg.patch[0] else {
            panic!("expected Set op at index 0");
        };
        let actions = value
            .get("actions")
            .expect("actions present")
            .as_array()
            .expect("array");
        assert_eq!(actions.len(), 3);
        assert_eq!(actions[0]["label"], "Edit");
        assert_eq!(actions[1]["label"], "Delete");
        assert_eq!(actions[2]["label"], "Duplicate");
    }

    #[tokio::test]
    async fn legacy_demo_rows_unchanged() {
        let ctx = make_ctx(serde_json::json!({ "source": "demo-rows" }));
        let result = handle_demo_fetch_rows(ctx).await.expect("ok");
        let ProtocolMessage::Patch(msg) = &result[0] else {
            panic!("expected Patch message");
        };
        assert_eq!(msg.patch.len(), 5);
        let PatchOperation::Set { path, .. } = &msg.patch[0] else {
            panic!("expected Set op at index 0");
        };
        assert_eq!(path, "/demo/data-table/rows/1");
    }

    #[tokio::test]
    async fn exer03_rows_first_page_50_ids_1_through_50() {
        // Phase 19 Plan 19-01: verify the exer-03-synthetic arm returns a
        // 50-row page starting at id=1 under /demo/exer-03/rows.
        let ctx = make_ctx(serde_json::json!({
            "source": "exer-03-synthetic",
            "offset": 0,
            "limit": 50
        }));
        let result = handle_demo_fetch_rows(ctx).await.expect("ok");
        let ProtocolMessage::Patch(msg) = &result[0] else {
            panic!("expected Patch message");
        };
        assert_eq!(msg.patch.len(), 50);
        let PatchOperation::Set { path, .. } = &msg.patch[0] else {
            panic!("expected Set op at index 0");
        };
        assert_eq!(path, "/demo/exer-03/rows/1");
        let PatchOperation::Set { path, .. } = &msg.patch[49] else {
            panic!("expected Set op at index 49");
        };
        assert_eq!(path, "/demo/exer-03/rows/50");
    }

    #[tokio::test]
    async fn exer03_rows_last_page_offset_9950() {
        // Phase 19 Plan 19-01: verify the 10_000-row cap — offset 9950 returns
        // exactly the final 50 rows ending at id=10_000, with no panic.
        let ctx = make_ctx(serde_json::json!({
            "source": "exer-03-synthetic",
            "offset": 9950,
            "limit": 50
        }));
        let result = handle_demo_fetch_rows(ctx).await.expect("ok");
        let ProtocolMessage::Patch(msg) = &result[0] else {
            panic!("expected Patch message");
        };
        assert_eq!(msg.patch.len(), 50);
        let PatchOperation::Set { path, .. } = &msg.patch[49] else {
            panic!("expected Set op at index 49");
        };
        assert_eq!(path, "/demo/exer-03/rows/10000");
    }

    #[tokio::test]
    async fn unknown_source_returns_bad_payload() {
        let ctx = make_ctx(serde_json::json!({ "source": "bogus" }));
        let err = handle_demo_fetch_rows(ctx).await.expect_err("should error");
        match err {
            ActionError::BadPayload(msg) => {
                assert!(
                    msg.contains("bogus"),
                    "error message should mention the unknown source, got: {msg}"
                );
            }
            other => panic!("expected BadPayload, got {other:?}"),
        }
    }
}
