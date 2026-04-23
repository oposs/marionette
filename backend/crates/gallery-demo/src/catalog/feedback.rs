//! CAT-04 — Feedback catalog screen.
//!
//! Two Cards side-by-side: (Card 1) trigger surfaces for toast / modal /
//! confirm-dialog + (Card 2) placeholder states for empty / loading / error.
//! All three triggers reuse existing Phase 17 handlers (`gallery-demo/toast-fire`,
//! `gallery-demo/modal-open`, `gallery-demo/confirm-open`) — Plan 18-07 does
//! NOT introduce new handlers (D-2-C). The placeholder mini-Cards are purely
//! static visuals composed from Container + Heading + Text + Spinner + ErrorDisplay
//! builders. Per D-2-C, the W-06 ErrorDisplay `message`-field dead-state is
//! NOT fixed here (deferred to a later polish plan).
//!
//! RED-phase stub — implementation lands in the GREEN commit.

use marionette::gallery::Node;

#[cfg(feature = "gallery")]
#[marionette_macros::gallery_demo(key = "catalog-feedback", name = "Catalog: Feedback")]
#[must_use]
pub fn gallery_demo() -> Vec<Node> {
    // Stub: returns a single empty-Container root so registration doesn't
    // emit an empty Vec (gallery-show handler rejects empty trees). Real
    // implementation lands in the GREEN commit.
    use marionette::builders::Container;
    Container::new()
        .id("catalog-feedback-root-stub")
        .build_with_children()
}

#[cfg(all(test, feature = "gallery"))]
mod tests {
    use super::*;
    use marionette::gallery::registered_demos;

    fn find(v: &[Node], id: &str) -> Option<serde_json::Value> {
        v.iter()
            .find(|(i, _)| i == id)
            .map(|(_, c)| serde_json::to_value(c).expect("serialize"))
    }

    #[test]
    fn root_id() {
        let v = gallery_demo();
        assert_eq!(v[0].0, "catalog-feedback-root");
    }

    #[test]
    fn three_trigger_buttons_with_locked_actions() {
        let v = gallery_demo();
        let toast = find(&v, "catalog-feedback-toast-trigger").expect("toast trigger");
        assert_eq!(toast["type"], "button");
        assert_eq!(toast["props"]["action"]["name"], "gallery-demo/toast-fire");
        let modal = find(&v, "catalog-feedback-modal-trigger").expect("modal trigger");
        assert_eq!(modal["props"]["action"]["name"], "gallery-demo/modal-open");
        let confirm = find(&v, "catalog-feedback-confirm-trigger").expect("confirm trigger");
        assert_eq!(confirm["props"]["action"]["name"], "gallery-demo/confirm-open");
    }

    #[test]
    fn empty_placeholder_has_border_dashed_class() {
        let v = gallery_demo();
        let empty = find(&v, "catalog-feedback-empty").expect("empty placeholder");
        let class = empty["props"]["class"].as_str().unwrap_or("");
        assert!(
            class.contains("border-dashed"),
            "empty class missing border-dashed: {class}"
        );
        assert!(class.contains("text-muted-foreground"));
    }

    #[test]
    fn loading_placeholder_has_spinner() {
        let v = gallery_demo();
        let spinner = find(&v, "catalog-feedback-loading-spinner").expect("spinner");
        assert_eq!(spinner["type"], "spinner");
    }

    #[test]
    fn error_display_bound_to_seeded_path() {
        let v = gallery_demo();
        let err = find(&v, "catalog-feedback-error").expect("error display");
        assert_eq!(err["type"], "error-display");
        assert_eq!(err["props"]["bind"], "/demo/catalog-feedback/errors");
    }

    #[test]
    fn registered_demos_includes_catalog_feedback() {
        let e = registered_demos()
            .find(|e| e.key == "catalog-feedback")
            .expect("registered");
        assert_eq!(e.display_name, "Catalog: Feedback");
    }
}
