use std::sync::{Arc, OnceLock};

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter};
use serde::Deserialize;

use marionette::error::{ActionError, ActionResult};
use marionette::extractors::{Db, FromHandlerContext, HandlerContext};
use crate::entities::{contact, contact_tag, listmonk_sync, tag};
use crate::listmonk::ListmonkClient;

/// Global holder for the Listmonk client, initialized in main.rs.
static LISTMONK_CLIENT: OnceLock<Arc<ListmonkClient>> = OnceLock::new();

/// Initialize the global Listmonk client. Called from main.rs after creating the client.
pub fn init_listmonk_client(client: Arc<ListmonkClient>) {
    let _ = LISTMONK_CLIENT.set(client);
}

/// Get a reference to the global Listmonk client, if configured.
pub fn get_listmonk_client() -> Option<&'static Arc<ListmonkClient>> {
    LISTMONK_CLIENT.get()
}

/// Sync a single contact to Listmonk as a subscriber.
///
/// Creates or updates the subscriber, maps tags to lists, and records
/// the sync status in the `listmonk_sync` table.
pub async fn sync_one_contact(
    client: &ListmonkClient,
    db: &sea_orm::DatabaseConnection,
    contact: &contact::Model,
    tags: &[String],
) -> Result<i32, String> {
    use sea_orm::ActiveValue::{NotSet, Set};

    // 1. For each tag name, get or create a Listmonk list
    let mut list_ids = Vec::new();
    for tag_name in tags {
        let list_id = client.get_or_create_list(tag_name).await?;
        list_ids.push(list_id);
    }

    // 2. Check if subscriber exists
    let name = &contact.contact_name;
    let email = &contact.contact_email;
    let existing = client.find_subscriber_by_email(email).await?;

    let subscriber_id = match existing {
        Some((id, _status)) => {
            // 3. Existing subscriber: update name/email and set lists
            client.update_subscriber(id, email, name).await?;
            if !list_ids.is_empty() {
                client
                    .set_subscriber_lists(&[id], &list_ids, "add")
                    .await?;
            }
            id
        }
        None => {
            // 4. New subscriber: create with list memberships
            client.create_subscriber(email, name, &list_ids).await?
        }
    };

    // 5. Record success in listmonk_sync table (upsert: delete old + insert new)
    listmonk_sync::Entity::delete_many()
        .filter(listmonk_sync::Column::ListmonkSyncContact.eq(contact.contact_id))
        .exec(db)
        .await
        .map_err(|e| e.to_string())?;

    let now = {
        let t = time::OffsetDateTime::now_utc();
        format!(
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
            t.year(),
            t.month() as u8,
            t.day(),
            t.hour(),
            t.minute(),
            t.second()
        )
    };

    let sync_record = listmonk_sync::ActiveModel {
        listmonk_sync_id: NotSet,
        listmonk_sync_contact: Set(contact.contact_id),
        listmonk_sync_status: Set("success".to_owned()),
        listmonk_sync_error: Set(None),
        listmonk_sync_subscriber_id: Set(Some(subscriber_id)),
        listmonk_sync_at: Set(now),
    };
    sync_record.insert(db).await.map_err(|e| e.to_string())?;

    Ok(subscriber_id)
}

/// Record a sync error in the `listmonk_sync` table.
async fn record_sync_error(
    db: &sea_orm::DatabaseConnection,
    contact_id: i32,
    error: &str,
) -> Result<(), String> {
    use sea_orm::ActiveValue::{NotSet, Set};

    // Upsert: delete old + insert new
    listmonk_sync::Entity::delete_many()
        .filter(listmonk_sync::Column::ListmonkSyncContact.eq(contact_id))
        .exec(db)
        .await
        .map_err(|e| e.to_string())?;

    let now = {
        let t = time::OffsetDateTime::now_utc();
        format!(
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
            t.year(),
            t.month() as u8,
            t.day(),
            t.hour(),
            t.minute(),
            t.second()
        )
    };

    let sync_record = listmonk_sync::ActiveModel {
        listmonk_sync_id: NotSet,
        listmonk_sync_contact: Set(contact_id),
        listmonk_sync_status: Set("error".to_owned()),
        listmonk_sync_error: Set(Some(error.to_owned())),
        listmonk_sync_subscriber_id: Set(None),
        listmonk_sync_at: Set(now),
    };
    sync_record.insert(db).await.map_err(|e| e.to_string())?;
    Ok(())
}

#[derive(Deserialize)]
struct SyncPayload {
    contact_id: i32,
}

/// Handle the `listmonk_sync` action: sync a single contact to Listmonk.
pub async fn handle_listmonk_sync(ctx: HandlerContext) -> ActionResult {
    let db = Db::from_context(&ctx)?;
    let payload = marionette::extractors::Payload::<SyncPayload>::from_context(&ctx)?;
    let cid = payload.0.contact_id;

    let client = get_listmonk_client()
        .ok_or_else(|| ActionError::Internal("Listmonk is not configured".into()))?;

    // Load contact
    let found = contact::Entity::find_by_id(cid)
        .one(&*db.0)
        .await
        .map_err(|e| ActionError::Internal(e.to_string()))?
        .ok_or_else(|| ActionError::Internal("Contact not found".into()))?;

    // Load contact tags
    let ct_rows = contact_tag::Entity::find()
        .filter(contact_tag::Column::ContactTagContact.eq(cid))
        .all(&*db.0)
        .await
        .map_err(|e| ActionError::Internal(e.to_string()))?;
    let all_tags = tag::Entity::find()
        .all(&*db.0)
        .await
        .map_err(|e| ActionError::Internal(e.to_string()))?;
    let tag_name_map: std::collections::HashMap<i32, String> =
        all_tags.into_iter().map(|t| (t.tag_id, t.tag_name)).collect();
    let tag_names: Vec<String> = ct_rows
        .iter()
        .filter_map(|ct| tag_name_map.get(&ct.contact_tag_tag).cloned())
        .collect();

    // Sync
    match sync_one_contact(client, &*db.0, &found, &tag_names).await {
        Ok(_subscriber_id) => {
            tracing::info!(contact_id = cid, "Contact synced to Listmonk");
        }
        Err(e) => {
            tracing::warn!(contact_id = cid, error = %e, "Failed to sync contact to Listmonk");
            let _ = record_sync_error(&*db.0, cid, &e).await;
        }
    }

    // Re-render the contact form
    let mut form_action = ctx.action.clone();
    form_action.payload = Some(serde_json::json!({ "contact_id": cid }));
    let form_ctx = HandlerContext {
        action: form_action,
        db: ctx.db.clone(),
        session: ctx.session.clone(),
    };
    super::contact::handle_contact_form(form_ctx).await
}

/// Handle the `listmonk_sync_all` action: bulk sync all contacts with emails.
pub async fn handle_listmonk_sync_all(ctx: HandlerContext) -> ActionResult {
    let db = Db::from_context(&ctx)?;

    let client = get_listmonk_client()
        .ok_or_else(|| ActionError::Internal("Listmonk is not configured".into()))?;

    // Load all contacts with non-empty email
    let contacts = contact::Entity::find()
        .all(&*db.0)
        .await
        .map_err(|e| ActionError::Internal(e.to_string()))?;

    // Load all tags and contact_tag mappings
    let all_tags = tag::Entity::find()
        .all(&*db.0)
        .await
        .map_err(|e| ActionError::Internal(e.to_string()))?;
    let tag_name_map: std::collections::HashMap<i32, String> =
        all_tags.into_iter().map(|t| (t.tag_id, t.tag_name)).collect();
    let all_ct = contact_tag::Entity::find()
        .all(&*db.0)
        .await
        .map_err(|e| ActionError::Internal(e.to_string()))?;

    // Build contact_id -> tag names map
    let mut contact_tags: std::collections::HashMap<i32, Vec<String>> =
        std::collections::HashMap::new();
    for ct in &all_ct {
        if let Some(name) = tag_name_map.get(&ct.contact_tag_tag) {
            contact_tags
                .entry(ct.contact_tag_contact)
                .or_default()
                .push(name.clone());
        }
    }

    let mut success_count = 0u32;
    let mut error_count = 0u32;

    for c in &contacts {
        if c.contact_email.trim().is_empty() {
            continue;
        }
        let tags = contact_tags
            .get(&c.contact_id)
            .cloned()
            .unwrap_or_default();
        match sync_one_contact(client, &*db.0, c, &tags).await {
            Ok(_) => success_count += 1,
            Err(e) => {
                tracing::warn!(contact_id = c.contact_id, error = %e, "Bulk sync error");
                let _ = record_sync_error(&*db.0, c.contact_id, &e).await;
                error_count += 1;
            }
        }
    }

    tracing::info!(
        "Bulk Listmonk sync complete: {success_count} synced, {error_count} errors"
    );

    // Re-render the contact list
    super::contact::handle_contact_list(HandlerContext {
        action: ctx.action.clone(),
        db: ctx.db.clone(),
        session: ctx.session.clone(),
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult};
    use wiremock::matchers::{method, path_regex};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn mock_client(server: &MockServer) -> ListmonkClient {
        ListmonkClient {
            client: reqwest::Client::new(),
            base_url: server.uri(),
            user: "test".into(),
            password: "test".into(),
        }
    }

    fn sample_contact(id: i32) -> contact::Model {
        contact::Model {
            contact_id: id,
            contact_name: "Jane Doe".into(),
            contact_email: "jane@example.com".into(),
            contact_phone: None,
            contact_title: None,
            contact_company: None,
            contact_created_at: "2026-01-01 00:00:00".into(),
            contact_updated_at: "2026-01-01 00:00:00".into(),
        }
    }

    #[tokio::test]
    async fn test_sync_new_contact_creates_subscriber() {
        let server = MockServer::start().await;

        // Mock find_subscriber (returns empty)
        Mock::given(method("GET"))
            .and(path_regex("api/subscribers$"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"data": {"results": []}})),
            )
            .mount(&server)
            .await;

        // Mock get lists (for tag mapping)
        Mock::given(method("GET"))
            .and(path_regex("api/lists"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"data": {"results": []}})),
            )
            .mount(&server)
            .await;

        // Mock create list
        Mock::given(method("POST"))
            .and(path_regex("api/lists"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"data": {"id": 1}})),
            )
            .mount(&server)
            .await;

        // Mock create subscriber
        Mock::given(method("POST"))
            .and(path_regex("api/subscribers$"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"data": {"id": 42}})),
            )
            .mount(&server)
            .await;

        let client = mock_client(&server);
        let contact = sample_contact(1);

        // Use MockDatabase for the sync record insert
        let db = MockDatabase::new(DatabaseBackend::Sqlite)
            // delete_many exec result
            .append_exec_results([MockExecResult {
                last_insert_id: 0,
                rows_affected: 0,
            }])
            // insert exec result
            .append_exec_results([MockExecResult {
                last_insert_id: 1,
                rows_affected: 1,
            }])
            // query result for the inserted model (sea-orm reads back after insert)
            .append_query_results([[listmonk_sync::Model {
                listmonk_sync_id: 1,
                listmonk_sync_contact: 1,
                listmonk_sync_status: "success".into(),
                listmonk_sync_error: None,
                listmonk_sync_subscriber_id: Some(42),
                listmonk_sync_at: "2026-01-01 00:00:00".into(),
            }]])
            .into_connection();

        let result = sync_one_contact(&client, &db, &contact, &["VIP".to_string()]).await;
        assert!(result.is_ok(), "sync should succeed: {:?}", result);
        assert_eq!(result.unwrap(), 42, "should return subscriber_id 42");
    }

    #[tokio::test]
    async fn test_sync_existing_contact_updates_subscriber() {
        let server = MockServer::start().await;

        // Mock find_subscriber (returns existing)
        Mock::given(method("GET"))
            .and(path_regex("api/subscribers$"))
            .respond_with(ResponseTemplate::new(200).set_body_json(
                serde_json::json!({"data": {"results": [{"id": 10, "status": "enabled"}]}}),
            ))
            .mount(&server)
            .await;

        // Mock update_subscriber
        Mock::given(method("PUT"))
            .and(path_regex("api/subscribers/10$"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
            .mount(&server)
            .await;

        // Mock set_subscriber_lists
        Mock::given(method("PUT"))
            .and(path_regex("api/subscribers/lists"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
            .mount(&server)
            .await;

        // Mock get lists
        Mock::given(method("GET"))
            .and(path_regex("api/lists"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(
                    serde_json::json!({"data": {"results": [{"id": 5, "name": "VIP"}]}}),
                ),
            )
            .mount(&server)
            .await;

        let client = mock_client(&server);
        let contact = sample_contact(2);

        // MockDatabase for sync record upsert
        let db = MockDatabase::new(DatabaseBackend::Sqlite)
            .append_exec_results([MockExecResult {
                last_insert_id: 0,
                rows_affected: 1,
            }])
            .append_exec_results([MockExecResult {
                last_insert_id: 2,
                rows_affected: 1,
            }])
            .append_query_results([[listmonk_sync::Model {
                listmonk_sync_id: 2,
                listmonk_sync_contact: 2,
                listmonk_sync_status: "success".into(),
                listmonk_sync_error: None,
                listmonk_sync_subscriber_id: Some(10),
                listmonk_sync_at: "2026-01-01 00:00:00".into(),
            }]])
            .into_connection();

        let result = sync_one_contact(&client, &db, &contact, &["VIP".to_string()]).await;
        assert!(result.is_ok(), "sync should succeed: {:?}", result);
        assert_eq!(result.unwrap(), 10, "should return existing subscriber_id");
    }

    #[tokio::test]
    async fn test_sync_records_error_on_api_failure() {
        let server = MockServer::start().await;

        // Mock find_subscriber returns 500
        Mock::given(method("GET"))
            .and(path_regex("api/subscribers"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
            .mount(&server)
            .await;

        let client = mock_client(&server);
        let contact = sample_contact(3);

        let result = sync_one_contact(
            &client,
            // We don't need a DB here because sync_one_contact will fail
            // before the DB write. But record_sync_error needs a DB --
            // we test that the error is returned properly.
            &MockDatabase::new(DatabaseBackend::Sqlite).into_connection(),
            &contact,
            &[],
        )
        .await;

        assert!(result.is_err(), "sync should fail on API error");
        let err = result.unwrap_err();
        assert!(
            err.contains("error") || err.contains("Error"),
            "error message should indicate failure: {err}"
        );
    }
}
