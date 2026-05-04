//! Type-keyed registry of app-defined services.
//!
//! Apps register their own state and clients in [`Extensions`] at bootstrap
//! and reach them from handlers via [`HandlerContext::extensions`]. The
//! registry holds at most one entry per concrete type (`TypeId` is the key)
//! and stores values as `Arc<dyn Any + Send + Sync>` so handlers receive a
//! cheap clone.
//!
//! Modeled on `axum::Extensions`. Lives on [`AppState`] and is cloned
//! (cheaply, via the inner `Arc<HashMap>`) into every [`HandlerContext`].
//!
//! ```ignore
//! // Bootstrap
//! let state = Arc::new(AppState {
//!     router: ActionRouter::new(),
//!     db,
//!     login_form: None,
//!     extensions: Extensions::new().with(MyStore::default()),
//! });
//!
//! // Handler
//! let store = ctx.extensions.get::<MyStore>().expect("store registered");
//! ```
//!
//! [`HandlerContext`]: crate::extractors::HandlerContext
//! [`AppState`]: crate::ws::AppState
//! [`HandlerContext::extensions`]: crate::extractors::HandlerContext::extensions

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

/// Map of app-defined extensions keyed by [`TypeId`]. See module docs.
#[derive(Clone, Default)]
pub struct Extensions {
    inner: Arc<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>,
}

impl Extensions {
    /// Construct an empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register `value` under its concrete type. Replaces any prior entry
    /// of the same type. Returns the receiver so registrations can chain
    /// at bootstrap (`Extensions::new().with(a).with(b)`).
    #[must_use]
    pub fn with<T: Any + Send + Sync>(self, value: T) -> Self {
        let mut next = (*self.inner).clone();
        next.insert(TypeId::of::<T>(), Arc::new(value));
        Self {
            inner: Arc::new(next),
        }
    }

    /// Borrow a registered extension by type. Returns `None` if nothing of
    /// that type was registered.
    #[must_use]
    pub fn get<T: Any + Send + Sync>(&self) -> Option<&T> {
        self.inner
            .get(&TypeId::of::<T>())
            .and_then(|arc| arc.downcast_ref::<T>())
    }

    /// Same as [`Self::get`] but returns the underlying [`Arc<T>`] so the
    /// caller can hold the value past the registry borrow.
    #[must_use]
    pub fn get_arc<T: Any + Send + Sync>(&self) -> Option<Arc<T>> {
        self.inner
            .get(&TypeId::of::<T>())
            .and_then(|arc| Arc::clone(arc).downcast::<T>().ok())
    }

    /// `true` when nothing has been registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Number of registered extensions.
    #[must_use]
    pub fn len(&self) -> usize {
        self.inner.len()
    }
}

impl std::fmt::Debug for Extensions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Extensions")
            .field("len", &self.inner.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq, Eq)]
    struct Counter(u32);

    #[derive(Debug, PartialEq, Eq)]
    struct Other(&'static str);

    #[test]
    fn empty_registry_returns_none() {
        let ext = Extensions::new();
        assert!(ext.is_empty());
        assert_eq!(ext.len(), 0);
        assert!(ext.get::<Counter>().is_none());
    }

    #[test]
    fn with_inserts_one_entry_per_type() {
        let ext = Extensions::new().with(Counter(7)).with(Other("hi"));
        assert_eq!(ext.len(), 2);
        assert_eq!(ext.get::<Counter>(), Some(&Counter(7)));
        assert_eq!(ext.get::<Other>(), Some(&Other("hi")));
    }

    #[test]
    fn with_replaces_existing_entry_of_same_type() {
        let ext = Extensions::new().with(Counter(1)).with(Counter(99));
        assert_eq!(ext.len(), 1);
        assert_eq!(ext.get::<Counter>(), Some(&Counter(99)));
    }

    #[test]
    fn get_arc_yields_shared_handle() {
        let ext = Extensions::new().with(Counter(3));
        let a = ext.get_arc::<Counter>().expect("registered");
        let b = ext.get_arc::<Counter>().expect("registered");
        assert!(Arc::ptr_eq(&a, &b));
    }

    #[test]
    fn clone_shares_inner_map() {
        let ext = Extensions::new().with(Counter(5));
        let cloned = ext.clone();
        assert!(Arc::ptr_eq(&ext.inner, &cloned.inner));
        assert_eq!(cloned.get::<Counter>(), Some(&Counter(5)));
    }

    #[test]
    fn with_after_clone_does_not_mutate_original() {
        let ext = Extensions::new().with(Counter(1));
        let extended = ext.clone().with(Other("added"));
        // Original keeps just Counter; extended sees both.
        assert_eq!(ext.len(), 1);
        assert_eq!(extended.len(), 2);
        assert!(ext.get::<Other>().is_none());
        assert_eq!(extended.get::<Other>(), Some(&Other("added")));
    }
}
