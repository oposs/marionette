# Phase 9: CRM Listmonk - Research

**Researched:** 2026-03-23
**Domain:** External service integration (Listmonk newsletter API)
**Confidence:** HIGH

## Summary

Phase 9 integrates the CRM with Listmonk, an open-source newsletter/mailing list manager, via its REST API. The integration involves two main capabilities: (1) syncing CRM contacts to Listmonk as subscribers with tag-to-list mapping, and (2) fetching mailing history (campaign views, link clicks) per subscriber for display in the CRM.

Listmonk provides a well-documented REST API with basic auth. The subscriber management endpoints support create, update, blocklist, and list membership operations. Campaign history per subscriber is available through the subscriber export endpoint (`GET /api/subscribers/{id}/export`), which returns campaign views and link clicks. The CRM will use `reqwest` (already a dev-dependency in the workspace) as the async HTTP client, connecting to Listmonk via environment-configured credentials.

The main architectural challenge is graceful degradation -- the CRM must work when Listmonk is unreachable. The sync status tracking table provides the foundation for error visibility, and the manual sync model (user-triggered, not automatic) simplifies error handling.

**Primary recommendation:** Build a `ListmonkClient` struct wrapping `reqwest::Client` with basic auth, add it as an `Option` on `AppState`, and implement sync/history handlers that return informative errors when the client is `None` (Listmonk not configured) or API calls fail.

<user_constraints>

## User Constraints (from CONTEXT.md)

### Locked Decisions
- Configuration via environment variables: `LISTMONK_URL`, `LISTMONK_USER`, `LISTMONK_PASSWORD`
- Basic auth against Listmonk's REST API
- Connection validated on startup -- log warning if Listmonk is unreachable but don't prevent CRM from starting
- HTTP client: `reqwest` crate (async, already compatible with tokio)
- Manual sync per contact -- "Sync to Listmonk" button on contact detail view
- Bulk sync -- "Sync All" button on contact list
- Sync creates/updates Listmonk subscriber using contact email as identifier
- Contact name maps to Listmonk subscriber name
- Tags map to Listmonk subscriber lists (each CRM tag = a Listmonk list)
- On contact email change, update the Listmonk subscriber
- On contact delete, mark Listmonk subscriber as "blocklisted" (not deleted)
- `listmonk_sync` table: `listmonk_sync_contact` (FK), `listmonk_sync_status` (success/error), `listmonk_sync_error` (nullable text), `listmonk_sync_subscriber_id` (Listmonk's ID), `listmonk_sync_at` (timestamp)
- Sync status shown as badge on contact list and detail view
- Fetch campaign send history from Listmonk API per subscriber ID
- Display as read-only timeline on contact detail (below interactions)
- Cached locally to avoid repeated API calls -- refresh on demand

### Claude's Discretion
- Exact Listmonk API endpoint paths and payload formats
- Retry logic for failed API calls
- Cache duration for mailing history
- How to handle Listmonk being down (graceful degradation UI)
- Whether to add a Listmonk settings/status page in admin
- Bulk sync progress reporting

### Deferred Ideas (OUT OF SCOPE)
None

</user_constraints>

<phase_requirements>

## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| CRM-15 | User can sync contacts to Listmonk subscriber lists | Listmonk subscriber API (POST/PUT /api/subscribers, PUT /api/subscribers/lists), reqwest client, listmonk_sync table, tag-to-list mapping |
| CRM-16 | User can view mailing history per contact from Listmonk | GET /api/subscribers/{id}/export returns campaign_views and link_clicks; local cache table |

</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| reqwest | 0.12 | Async HTTP client for Listmonk API | Already a dev-dependency in workspace, async/tokio-native, supports basic auth natively |
| sea-orm | 1.1 | ORM for listmonk_sync table + cache | Already in workspace, established entity patterns |
| serde_json | 1 | JSON serialization for API payloads/responses | Already in workspace |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| tracing | 0.1 | Log sync operations, connection warnings | Already in workspace, use for startup validation and sync error logging |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| reqwest | hyper directly | reqwest provides higher-level API with built-in basic auth, connection pooling -- no reason to go lower-level |

**Installation:**
```bash
# reqwest is already a dev-dependency; move to [dependencies] for crm-demo
# Add to crm-demo Cargo.toml:
reqwest = { version = "0.12", features = ["rustls-tls", "json"], default-features = false }
```

**Version verification:** reqwest 0.12 is already in workspace dev-dependencies with `rustls-tls` feature. Need to add `json` feature for `.json()` request builder.

## Architecture Patterns

### Recommended Project Structure
```
backend/crates/crm-demo/src/
  listmonk.rs              # ListmonkClient struct + API methods
  entities/
    listmonk_sync.rs       # SeaORM entity for sync status
    listmonk_cache.rs      # SeaORM entity for cached mailing history
    mod.rs                 # Add new entities
  handlers/
    listmonk.rs            # Sync and history action handlers
    contact.rs             # Extended: sync button, sync status badge, mailing history
    mod.rs                 # Add listmonk handler module
  migration/
    m20260323_000009_*.rs  # listmonk_sync table
    m20260323_000010_*.rs  # listmonk_cache table
    mod.rs                 # Register new migrations
  main.rs                  # Add ListmonkClient to AppState, register sync actions
```

### Pattern 1: ListmonkClient as Optional AppState Extension
**What:** Wrap the Listmonk HTTP client in an `Option<ListmonkClient>` on a new struct or extend `AppState`.
**When to use:** Always -- the CRM must start without Listmonk.
**Example:**
```rust
// listmonk.rs
pub struct ListmonkClient {
    client: reqwest::Client,
    base_url: String,
    // basic auth is set on the client via default headers or per-request
    user: String,
    password: String,
}

impl ListmonkClient {
    pub fn from_env() -> Option<Self> {
        let url = std::env::var("LISTMONK_URL").ok()?;
        let user = std::env::var("LISTMONK_USER").ok()?;
        let password = std::env::var("LISTMONK_PASSWORD").ok()?;
        Some(Self {
            client: reqwest::Client::new(),
            base_url: url.trim_end_matches('/').to_owned(),
            user,
            password,
        })
    }

    pub async fn validate_connection(&self) -> bool {
        self.client
            .get(format!("{}/api/lists", self.base_url))
            .basic_auth(&self.user, Some(&self.password))
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }
}
```

**Note:** `AppState` currently has fields `router`, `db`, `login_form`. Adding a `listmonk: Option<ListmonkClient>` field is the cleanest approach. However, `AppState` lives in the marionette crate (`marionette::ws::AppState`). Two options:
1. Add the field to marionette's `AppState` (leaks CRM-specific concerns into the toolkit)
2. Use a separate struct or trait object pattern

**Recommendation:** Since `AppState` is in the marionette library crate, the cleanest approach is to add a generic `extensions` field (e.g., `pub extra: Option<Arc<dyn Any + Send + Sync>>`) or simply add the `listmonk` field directly since the toolkit is primarily for this CRM demo. Given the project's pragmatic approach (demo-scale), adding the field directly is acceptable.

### Pattern 2: Sync Handler with Error Capture
**What:** Each sync operation writes status to `listmonk_sync` regardless of outcome.
**When to use:** All sync operations (single and bulk).
**Example:**
```rust
async fn sync_contact_to_listmonk(
    client: &ListmonkClient,
    db: &DatabaseConnection,
    contact: &contact::Model,
    tags: &[String],
) -> Result<i32, String> {
    // 1. Ensure Listmonk lists exist for each tag
    // 2. Create or update subscriber
    // 3. Set list memberships
    // 4. Record success in listmonk_sync
    // Returns Listmonk subscriber_id on success
}
```

### Pattern 3: Subscriber Lookup by Email
**What:** Listmonk identifies subscribers by email. Use `GET /api/subscribers?query=subscribers.email='x'` to find existing subscribers before create/update.
**When to use:** Every sync operation to determine create vs update.
**Example:**
```rust
pub async fn find_subscriber_by_email(&self, email: &str) -> Result<Option<Subscriber>, reqwest::Error> {
    let query = format!("subscribers.email = '{}'", email.replace('\'', "''"));
    let resp = self.client
        .get(format!("{}/api/subscribers", self.base_url))
        .basic_auth(&self.user, Some(&self.password))
        .query(&[("query", &query), ("per_page", &"1".to_string())])
        .send()
        .await?
        .json::<SubscriberListResponse>()
        .await?;
    Ok(resp.data.results.into_iter().next())
}
```

### Pattern 4: Tag-to-List Mapping
**What:** Each CRM tag maps to a Listmonk list. On sync, ensure lists exist and assign subscriber to matching lists.
**When to use:** During contact sync.
**Steps:**
1. Fetch all Listmonk lists (`GET /api/lists`)
2. For each CRM tag, find matching list by name or create it (`POST /api/lists`)
3. Use `PUT /api/subscribers/lists` to set the subscriber's list memberships

### Pattern 5: Mailing History Cache
**What:** Cache the subscriber export data locally to avoid repeated API calls.
**When to use:** When displaying mailing history on contact detail.
**Recommendation:** Cache for 15 minutes. Store as JSON blob in a `listmonk_cache` table with `contact_id`, `data` (JSON), `cached_at` (timestamp). On display, check age; if stale, re-fetch. Add "Refresh" button for manual refresh.

### Anti-Patterns to Avoid
- **Automatic sync on every contact save:** Creates tight coupling and confusing UX when Listmonk is down. Use manual sync only.
- **Storing Listmonk credentials in database:** Environment variables are the right choice -- no credential management UI needed.
- **Deleting Listmonk subscribers on contact delete:** Blocklisting preserves mailing history as decided.
- **Parsing HTML from Listmonk responses:** All Listmonk API responses are JSON.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| HTTP client | Custom HTTP layer | reqwest with basic_auth | Connection pooling, TLS, timeouts, redirects |
| JSON API response parsing | Manual string parsing | serde derive structs | Type safety, error handling |
| Retry logic | Custom retry loop | Simple retry with backoff (2-3 attempts) | Keep it simple -- manual sync means user can retry |
| SQL escaping for Listmonk queries | String interpolation | Parameterized subscriber query format | Listmonk uses its own query syntax, but still escape single quotes |

**Key insight:** The Listmonk API is straightforward REST -- the complexity is in error handling and graceful degradation, not in the API calls themselves.

## Common Pitfalls

### Pitfall 1: Listmonk Subscriber Email Uniqueness
**What goes wrong:** Creating a subscriber with an existing email returns a conflict error, not an update.
**Why it happens:** Listmonk enforces unique emails per subscriber.
**How to avoid:** Always query by email first (`GET /api/subscribers?query=subscribers.email='...'`), then decide create (POST) vs update (PUT).
**Warning signs:** 409 Conflict responses from POST /api/subscribers.

### Pitfall 2: List Membership API Semantics
**What goes wrong:** Using the wrong `action` value in `PUT /api/subscribers/lists`.
**Why it happens:** The API supports `add`, `remove`, and `unsubscribe` -- `remove` deletes the subscription record, `unsubscribe` marks it as unsubscribed (different semantics).
**How to avoid:** Use `add` for syncing tags, `unsubscribe` for removed tags (preserves history).
**Warning signs:** Subscribers losing list history unexpectedly.

### Pitfall 3: AppState Modification
**What goes wrong:** Adding a field to `AppState` in the marionette crate changes the library's public API.
**Why it happens:** `AppState` is defined in `marionette::ws` module.
**How to avoid:** Either accept the coupling (demo project) or use a type-erased extension field.
**Warning signs:** Compile errors in other crates that construct `AppState`.

### Pitfall 4: SQL Injection in Listmonk Query Syntax
**What goes wrong:** Unsanitized email addresses passed to Listmonk's subscriber query.
**Why it happens:** Listmonk uses SQL-like query expressions (`subscribers.email = '...'`).
**How to avoid:** Escape single quotes in email addresses before building the query string.
**Warning signs:** Query errors or unexpected results with emails containing special characters.

### Pitfall 5: Blocking Startup on Listmonk Connection
**What goes wrong:** CRM fails to start when Listmonk is down.
**Why it happens:** Validation check blocks if DNS resolution or TCP connect hangs.
**How to avoid:** Use a timeout (5 seconds) on the validation request. Log warning on failure, continue startup.
**Warning signs:** CRM startup hanging or failing in environments without Listmonk.

### Pitfall 6: Mailing History Export Size
**What goes wrong:** The subscriber export endpoint returns all campaign views and link clicks, which could be large for active subscribers.
**Why it happens:** No pagination on the export endpoint.
**How to avoid:** Cache the data and limit what's displayed (e.g., last 50 entries). The cache prevents repeated large fetches.
**Warning signs:** Slow API responses for highly engaged subscribers.

## Code Examples

### Listmonk API: Create Subscriber
```rust
// Source: https://listmonk.app/docs/apis/subscribers/
// POST /api/subscribers
#[derive(Serialize)]
struct CreateSubscriberRequest {
    email: String,
    name: String,
    status: String,     // "enabled"
    lists: Vec<i32>,    // Listmonk list IDs
    preconfirm_subscriptions: bool,
}

pub async fn create_subscriber(&self, email: &str, name: &str, list_ids: &[i32]) -> Result<i32, String> {
    let body = CreateSubscriberRequest {
        email: email.to_owned(),
        name: name.to_owned(),
        status: "enabled".to_owned(),
        lists: list_ids.to_vec(),
        preconfirm_subscriptions: true,
    };
    let resp = self.client
        .post(format!("{}/api/subscribers", self.base_url))
        .basic_auth(&self.user, Some(&self.password))
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Listmonk API error: {text}"));
    }
    let result: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    result["data"]["id"].as_i64()
        .map(|id| id as i32)
        .ok_or_else(|| "Missing subscriber ID in response".into())
}
```

### Listmonk API: Update Subscriber
```rust
// Source: https://listmonk.app/docs/apis/subscribers/
// PUT /api/subscribers/{id}
pub async fn update_subscriber(&self, subscriber_id: i32, email: &str, name: &str) -> Result<(), String> {
    let body = serde_json::json!({
        "email": email,
        "name": name,
        "status": "enabled",
    });
    let resp = self.client
        .put(format!("{}/api/subscribers/{subscriber_id}", self.base_url))
        .basic_auth(&self.user, Some(&self.password))
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Listmonk update error: {text}"));
    }
    Ok(())
}
```

### Listmonk API: Blocklist Subscriber
```rust
// Source: https://listmonk.app/docs/apis/subscribers/
// PUT /api/subscribers/{id}/blocklist
pub async fn blocklist_subscriber(&self, subscriber_id: i32) -> Result<(), String> {
    let resp = self.client
        .put(format!("{}/api/subscribers/{subscriber_id}/blocklist", self.base_url))
        .basic_auth(&self.user, Some(&self.password))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Listmonk blocklist error: {text}"));
    }
    Ok(())
}
```

### Listmonk API: Set Subscriber List Memberships
```rust
// Source: https://listmonk.app/docs/apis/subscribers/
// PUT /api/subscribers/lists
pub async fn set_subscriber_lists(
    &self,
    subscriber_ids: &[i32],
    list_ids: &[i32],
    action: &str,  // "add", "remove", or "unsubscribe"
) -> Result<(), String> {
    let body = serde_json::json!({
        "ids": subscriber_ids,
        "action": action,
        "target_list_ids": list_ids,
        "status": "confirmed",
    });
    let resp = self.client
        .put(format!("{}/api/subscribers/lists", self.base_url))
        .basic_auth(&self.user, Some(&self.password))
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Listmonk list membership error: {text}"));
    }
    Ok(())
}
```

### Listmonk API: Get or Create List
```rust
// Source: https://listmonk.app/docs/apis/lists/
pub async fn get_or_create_list(&self, name: &str) -> Result<i32, String> {
    // Check if list exists
    let resp = self.client
        .get(format!("{}/api/lists", self.base_url))
        .basic_auth(&self.user, Some(&self.password))
        .query(&[("query", name), ("per_page", "100")])
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let result: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    if let Some(lists) = result["data"]["results"].as_array() {
        for list in lists {
            if list["name"].as_str() == Some(name) {
                return list["id"].as_i64()
                    .map(|id| id as i32)
                    .ok_or_else(|| "Missing list ID".into());
            }
        }
    }
    // Create new list
    let body = serde_json::json!({
        "name": name,
        "type": "private",
        "optin": "single",
    });
    let resp = self.client
        .post(format!("{}/api/lists", self.base_url))
        .basic_auth(&self.user, Some(&self.password))
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let result: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    result["data"]["id"].as_i64()
        .map(|id| id as i32)
        .ok_or_else(|| "Missing list ID in create response".into())
}
```

### Listmonk API: Fetch Subscriber Export (Mailing History)
```rust
// Source: https://listmonk.app/docs/apis/subscribers/
// GET /api/subscribers/{id}/export
pub async fn get_subscriber_export(&self, subscriber_id: i32) -> Result<serde_json::Value, String> {
    let resp = self.client
        .get(format!("{}/api/subscribers/{subscriber_id}/export", self.base_url))
        .basic_auth(&self.user, Some(&self.password))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("Export failed: {}", resp.status()));
    }
    resp.json().await.map_err(|e| e.to_string())
}
```

### Sync Status Database Table
```sql
-- Following TOOLING.md conventions
CREATE TABLE listmonk_sync (
    listmonk_sync_id INTEGER PRIMARY KEY AUTOINCREMENT,
    listmonk_sync_contact INTEGER NOT NULL REFERENCES contact(contact_id) ON DELETE CASCADE,
    listmonk_sync_status TEXT NOT NULL CHECK (listmonk_sync_status IN ('success', 'error')),
    listmonk_sync_error TEXT,
    listmonk_sync_subscriber_id INTEGER,
    listmonk_sync_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(listmonk_sync_contact)
);
```

### Mailing History Cache Table
```sql
CREATE TABLE listmonk_cache (
    listmonk_cache_id INTEGER PRIMARY KEY AUTOINCREMENT,
    listmonk_cache_contact INTEGER NOT NULL REFERENCES contact(contact_id) ON DELETE CASCADE,
    listmonk_cache_data TEXT NOT NULL CHECK (json_valid(listmonk_cache_data)),
    listmonk_cache_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(listmonk_cache_contact)
);
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Listmonk v2.x API | Listmonk v4.x API (same REST interface) | 2024 | API paths unchanged, added user roles/permissions |
| No subscriber export | GET /api/subscribers/{id}/export | v3.0+ | Enables mailing history per subscriber |

**Deprecated/outdated:**
- None relevant -- Listmonk's REST API has been stable since v2.

## Open Questions

1. **AppState extension mechanism**
   - What we know: `AppState` is in the marionette crate with 3 fields
   - What's unclear: Best way to add Listmonk client without changing the library crate's public API
   - Recommendation: Add `listmonk` field directly to `AppState` -- this is a demo project, pragmatism over purity. Alternatively, add a generic `pub extra: Option<Arc<dyn std::any::Any + Send + Sync>>` field.

2. **Subscriber export response schema**
   - What we know: Returns campaign_views and link_clicks arrays
   - What's unclear: Exact field names/types in those arrays (campaign name, timestamps)
   - Recommendation: Parse as `serde_json::Value` initially, display what's available. The cache stores raw JSON so schema changes don't require migration.

3. **Bulk sync progress reporting**
   - What we know: "Sync All" button syncs all contacts with emails
   - What's unclear: How to report progress in SDUI model (no streaming responses)
   - Recommendation: Process all contacts server-side, return a summary render with success/failure counts. For large contact lists, this could take time -- show a loading state before starting.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | cargo test (Rust) |
| Config file | backend/Cargo.toml workspace |
| Quick run command | `cd backend && cargo test -p crm-demo --lib` |
| Full suite command | `cd backend && cargo test` |

### Phase Requirements to Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| CRM-15 | Sync contact creates/updates Listmonk subscriber | unit | `cd backend && cargo test -p crm-demo listmonk -x` | No -- Wave 0 |
| CRM-15 | Sync status recorded in DB | unit | `cd backend && cargo test -p crm-demo listmonk_sync -x` | No -- Wave 0 |
| CRM-15 | Tag-to-list mapping creates lists | unit | `cd backend && cargo test -p crm-demo listmonk -x` | No -- Wave 0 |
| CRM-15 | Blocklist on contact delete | unit | `cd backend && cargo test -p crm-demo listmonk -x` | No -- Wave 0 |
| CRM-16 | Mailing history fetched and cached | unit | `cd backend && cargo test -p crm-demo listmonk_cache -x` | No -- Wave 0 |
| CRM-16 | Cached history displayed in timeline | manual-only | Visual check on contact detail | N/A |

### Sampling Rate
- **Per task commit:** `cd backend && cargo test -p crm-demo --lib`
- **Per wave merge:** `cd backend && cargo test`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `backend/crates/crm-demo/src/listmonk.rs` -- ListmonkClient unit tests (mock HTTP responses)
- [ ] `listmonk_sync` entity and migration
- [ ] `listmonk_cache` entity and migration
- [ ] reqwest added to crm-demo dependencies

## Sources

### Primary (HIGH confidence)
- [Listmonk Subscribers API](https://listmonk.app/docs/apis/subscribers/) -- All subscriber endpoints, create/update/blocklist/export
- [Listmonk Lists API](https://listmonk.app/docs/apis/lists/) -- List CRUD endpoints
- [Listmonk API Introduction](https://listmonk.app/docs/apis/apis/) -- Authentication (basic auth), response format
- [Listmonk Campaigns API](https://listmonk.app/docs/apis/campaigns/) -- Campaign endpoints and analytics

### Secondary (MEDIUM confidence)
- [DeepWiki Listmonk API Reference](https://deepwiki.com/knadh/listmonk/10-api-reference) -- Subscriber export schema details (campaign_views, link_clicks)

### Tertiary (LOW confidence)
- Subscriber export exact field schema -- inferred from documentation descriptions, not verified against actual API response. Recommend parsing as `serde_json::Value` and adapting at runtime.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - reqwest already in project, Listmonk API well-documented
- Architecture: HIGH - follows established CRM patterns (new entity, handler, migration), well-defined integration points
- Pitfalls: HIGH - API semantics verified from official docs, AppState extension is the only uncertain area
- Mailing history schema: MEDIUM - export endpoint documented but exact response fields not verified with live API

**Research date:** 2026-03-23
**Valid until:** 2026-04-23 (Listmonk API is stable, 30-day window appropriate)
