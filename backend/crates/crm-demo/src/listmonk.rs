use serde::Serialize;
use std::time::Duration;

/// HTTP client for the Listmonk newsletter API.
///
/// Wraps `reqwest::Client` with basic auth credentials and provides
/// typed methods for all subscriber and list management operations.
pub struct ListmonkClient {
    pub(crate) client: reqwest::Client,
    pub(crate) base_url: String,
    pub(crate) user: String,
    pub(crate) password: String,
}

#[derive(Serialize)]
struct CreateSubscriberRequest {
    email: String,
    name: String,
    status: String,
    lists: Vec<i32>,
    preconfirm_subscriptions: bool,
}

impl ListmonkClient {
    /// Create a client from environment variables.
    ///
    /// Returns `None` if `LISTMONK_URL`, `LISTMONK_USER`, or
    /// `LISTMONK_PASSWORD` is not set.
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

    /// Validate the connection by fetching the lists endpoint.
    ///
    /// Returns `true` on a 2xx response within 5 seconds, `false` otherwise.
    pub async fn validate_connection(&self) -> bool {
        self.client
            .get(format!("{}/api/lists", self.base_url))
            .basic_auth(&self.user, Some(&self.password))
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }

    /// Find a subscriber by email address.
    ///
    /// Returns `(subscriber_id, status)` if found.
    pub async fn find_subscriber_by_email(
        &self,
        email: &str,
    ) -> Result<Option<(i32, String)>, String> {
        let escaped = email.replace('\'', "''");
        let query = format!("subscribers.email = '{escaped}'");
        let resp = self
            .client
            .get(format!("{}/api/subscribers", self.base_url))
            .basic_auth(&self.user, Some(&self.password))
            .query(&[("query", &query), ("per_page", &"1".to_string())])
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if !resp.status().is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("Listmonk subscriber query error: {text}"));
        }
        let result: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
        if let Some(first) = result["data"]["results"].as_array().and_then(|a| a.first()) {
            let id = first["id"]
                .as_i64()
                .map(|v| v as i32)
                .ok_or_else(|| "Missing subscriber ID".to_string())?;
            let status = first["status"]
                .as_str()
                .unwrap_or("unknown")
                .to_string();
            Ok(Some((id, status)))
        } else {
            Ok(None)
        }
    }

    /// Create a new subscriber with the given email, name, and list memberships.
    ///
    /// Returns the new subscriber ID.
    pub async fn create_subscriber(
        &self,
        email: &str,
        name: &str,
        list_ids: &[i32],
    ) -> Result<i32, String> {
        let body = CreateSubscriberRequest {
            email: email.to_owned(),
            name: name.to_owned(),
            status: "enabled".to_owned(),
            lists: list_ids.to_vec(),
            preconfirm_subscriptions: true,
        };
        let resp = self
            .client
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
        result["data"]["id"]
            .as_i64()
            .map(|id| id as i32)
            .ok_or_else(|| "Missing subscriber ID in response".into())
    }

    /// Update an existing subscriber's email and name.
    pub async fn update_subscriber(
        &self,
        subscriber_id: i32,
        email: &str,
        name: &str,
    ) -> Result<(), String> {
        let body = serde_json::json!({
            "email": email,
            "name": name,
            "status": "enabled",
        });
        let resp = self
            .client
            .put(format!(
                "{}/api/subscribers/{subscriber_id}",
                self.base_url
            ))
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

    /// Blocklist a subscriber (marks them as blocked without deleting).
    pub async fn blocklist_subscriber(&self, subscriber_id: i32) -> Result<(), String> {
        let resp = self
            .client
            .put(format!(
                "{}/api/subscribers/{subscriber_id}/blocklist",
                self.base_url
            ))
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

    /// Get a list by exact name, or create it if it does not exist.
    ///
    /// Returns the list ID.
    pub async fn get_or_create_list(&self, name: &str) -> Result<i32, String> {
        // Fetch existing lists
        let resp = self
            .client
            .get(format!("{}/api/lists", self.base_url))
            .basic_auth(&self.user, Some(&self.password))
            .query(&[("per_page", "100")])
            .send()
            .await
            .map_err(|e| e.to_string())?;
        let result: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
        if let Some(lists) = result["data"]["results"].as_array() {
            for list in lists {
                if list["name"].as_str() == Some(name) {
                    return list["id"]
                        .as_i64()
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
        let resp = self
            .client
            .post(format!("{}/api/lists", self.base_url))
            .basic_auth(&self.user, Some(&self.password))
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if !resp.status().is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("Listmonk create list error: {text}"));
        }
        let result: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
        result["data"]["id"]
            .as_i64()
            .map(|id| id as i32)
            .ok_or_else(|| "Missing list ID in create response".into())
    }

    /// Modify subscriber list memberships.
    ///
    /// `action` should be `"add"`, `"remove"`, or `"unsubscribe"`.
    pub async fn set_subscriber_lists(
        &self,
        subscriber_ids: &[i32],
        list_ids: &[i32],
        action: &str,
    ) -> Result<(), String> {
        let body = serde_json::json!({
            "ids": subscriber_ids,
            "action": action,
            "target_list_ids": list_ids,
            "status": "confirmed",
        });
        let resp = self
            .client
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

    /// Fetch the full subscriber export (campaign views, link clicks, etc.).
    pub async fn get_subscriber_export(
        &self,
        subscriber_id: i32,
    ) -> Result<serde_json::Value, String> {
        let resp = self
            .client
            .get(format!(
                "{}/api/subscribers/{subscriber_id}/export",
                self.base_url
            ))
            .basic_auth(&self.user, Some(&self.password))
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if !resp.status().is_success() {
            return Err(format!("Export failed: {}", resp.status()));
        }
        resp.json().await.map_err(|e| e.to_string())
    }
}
