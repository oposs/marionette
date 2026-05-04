//! In-memory people store, registered into [`marionette::Extensions`] at
//! bootstrap and reached from handlers via `ctx.extensions.get::<PeopleStore>()`.
//!
//! "Restart is reset" — there is no persistence layer here. A real app would
//! store rows in SeaORM via `ctx.db`.

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Person {
    pub id: String,
    pub name: String,
    pub email: String,
    pub country: String,
}

/// Append-only in-memory list. Registered into `Extensions` as itself
/// (the framework wraps the value in `Arc` exactly once); handlers reach
/// it via `ctx.extensions.get::<PeopleStore>()` or `get_arc` for a shared
/// handle.
#[derive(Default)]
pub struct PeopleStore {
    rows: RwLock<Vec<Person>>,
}

impl PeopleStore {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn snapshot(&self) -> Vec<Person> {
        self.rows.read().await.clone()
    }

    pub async fn add(&self, person: Person) {
        self.rows.write().await.push(person);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn fresh_store_is_empty() {
        let store = PeopleStore::default();
        assert!(store.snapshot().await.is_empty());
    }

    #[tokio::test]
    async fn add_then_snapshot_yields_value() {
        let store = PeopleStore::default();
        let p = Person {
            id: "abc".into(),
            name: "Ada".into(),
            email: "ada@example.com".into(),
            country: "uk".into(),
        };
        store.add(p.clone()).await;
        assert_eq!(store.snapshot().await, vec![p]);
    }
}
