//! Gallery-demo action handlers.
//!
//! One file per handler family (per RESEARCH.md §Architecture Patterns recommended
//! project structure). `register_gallery_actions()` is the single entry point main.rs
//! calls to wire every handler into the ActionRouter.

use marionette::router::{ActionRouter, box_handler};
use marionette_protocol::common::AuthRequirement;

pub mod catalog_forms;
pub mod confirm;
pub mod exer01; // Phase 19 Plan 19-01 stub; Plan 19-02 implementation
pub mod exer02; // Phase 19 Plan 19-01 stub; Plan 19-03 implementation
pub mod exer03; // Phase 19 Plan 19-01 stub; Plan 19-04 implementation
pub mod fetch_rows;
pub mod modal;
pub mod navigate;
pub mod noop;
pub mod show;
pub mod toast;

/// Register every gallery-demo action handler on the given router, in a single
/// helper for readability. Action names match CONTEXT.md §D-C4 with the
/// frontend-hardcoded `close-modal` name included per RESEARCH.md §Pitfall 3.
/// (Legacy `dismiss-toast` removed in the sonner migration — toasts are now
/// dispatched as `type: "event"` and sonner owns dismissal. See docs/SDUI-CONCEPT.md
/// §"Where the Client Is Smart".)
#[must_use]
pub fn register_gallery_actions(router: ActionRouter) -> ActionRouter {
    // Force-link external demo crates (currently just gallery-smoke) so the
    // linkme-backed DEMOS slice is populated. Without this, integration tests
    // and the production binary would see an empty registry despite the
    // Cargo.toml dep on gallery-smoke. See `lib.rs::__force_link_gallery_smoke`.
    crate::ensure_demos_linked();
    router
        .action("navigate", box_handler(navigate::handle_navigate), AuthRequirement::None)
        .action("gallery-show", box_handler(show::handle_gallery_show), AuthRequirement::None)
        .action("gallery-demo/noop", box_handler(noop::handle_noop), AuthRequirement::None)
        .action("gallery-demo/modal-open", box_handler(modal::handle_modal_open), AuthRequirement::None)
        .action("close-modal", box_handler(modal::handle_modal_close), AuthRequirement::None)
        .action("gallery-demo/confirm-open", box_handler(confirm::handle_confirm_open), AuthRequirement::None)
        .action("gallery-demo/confirm-accept", box_handler(confirm::handle_confirm_accept), AuthRequirement::None)
        .action("gallery-demo/confirm-reject", box_handler(confirm::handle_confirm_reject), AuthRequirement::None)
        .action("gallery-demo/toast-fire", box_handler(toast::handle_toast_fire), AuthRequirement::None)
        .action("fetch-rows", box_handler(fetch_rows::handle_demo_fetch_rows), AuthRequirement::None)
        // --- CAT-02 blur-validate handlers (Phase 18 Plan 18-05) ---
        // Six validators demonstrate Phase 12 node-tree ops (set-node /
        // set-children / delete-node) rotated across every input type.
        .action(
            "gallery-demo/catalog-forms/validate-text-input",
            box_handler(catalog_forms::validate_text_input),
            AuthRequirement::None,
        )
        .action(
            "gallery-demo/catalog-forms/validate-select",
            box_handler(catalog_forms::validate_select),
            AuthRequirement::None,
        )
        .action(
            "gallery-demo/catalog-forms/validate-checkbox",
            box_handler(catalog_forms::validate_checkbox),
            AuthRequirement::None,
        )
        .action(
            "gallery-demo/catalog-forms/validate-switch",
            box_handler(catalog_forms::validate_switch),
            AuthRequirement::None,
        )
        .action(
            "gallery-demo/catalog-forms/validate-radio",
            box_handler(catalog_forms::validate_radio),
            AuthRequirement::None,
        )
        .action(
            "gallery-demo/catalog-forms/validate-textarea",
            box_handler(catalog_forms::validate_textarea),
            AuthRequirement::None,
        )
        // --- Phase 19 exerciser handlers (stubs in Plan 19-01; real impls in
        //     Plans 19-02/03/04). All 7 routes registered here so Wave 2 plans
        //     can drop implementations into place without touching this file
        //     again. gallery-demo/exer-02/tick is the explicit 19-01 -> 19-03
        //     handoff contract (router-dispatch reachability verified below). ---
        .action(
            "gallery-demo/exer-01/report",
            box_handler(exer01::handle_exer01_report),
            AuthRequirement::None,
        )
        .action(
            "gallery-demo/exer-01/open-seed",
            box_handler(exer01::handle_exer01_open_seed),
            AuthRequirement::None,
        )
        .action(
            "gallery-demo/exer-02/start",
            box_handler(exer02::handle_exer02_start),
            AuthRequirement::None,
        )
        .action(
            "gallery-demo/exer-02/pause",
            box_handler(exer02::handle_exer02_pause),
            AuthRequirement::None,
        )
        .action(
            "gallery-demo/exer-02/reset",
            box_handler(exer02::handle_exer02_reset),
            AuthRequirement::None,
        )
        .action(
            "gallery-demo/exer-02/tick",
            box_handler(exer02::handle_exer02_tick),
            AuthRequirement::None,
        )
        .action(
            "gallery-demo/exer-03/report-perf",
            box_handler(exer03::handle_exer03_report_perf),
            AuthRequirement::None,
        )
        .action(
            "gallery-demo/exer-03/remeasure",
            box_handler(exer03::handle_exer03_remeasure),
            AuthRequirement::None,
        )
}

#[cfg(test)]
mod router_tests {
    //! Phase 19 exerciser route reachability tests.
    //!
    //! Originally written in Plan 19-01 as the 19-01 -> 19-03 handoff guard
    //! when all 7 handlers were `Ok(vec![])` stubs (assertion: "dispatch
    //! returns an empty Vec"). Updated in Plan 19-03 when the exer-02
    //! handlers shipped real PatchMessage-emitting bodies: the assertion
    //! now checks "dispatch does NOT return a NotFound error" (the
    //! underlying reachability property), which survives both the stub era
    //! and the real-implementation era.
    //!
    //! ActionRouter::dispatch returns `Vec<ProtocolMessage>` (not a Result).
    //! Unknown routes produce a single `ProtocolMessage::Error` whose
    //! message contains "Action not found"; successful dispatches return
    //! whatever the handler builds (empty Vec for Plan 19-01 stubs, a
    //! single PatchMessage for Plan 19-03 real handlers).
    use super::*;

    use std::sync::Arc;

    use marionette::extractors::{HandlerContext, Session};
    use marionette::router::ActionRouter;
    use marionette_protocol::ActionMessage;
    use marionette_protocol::ProtocolMessage;
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

    fn ctx_for(action_name: &str) -> HandlerContext {
        HandlerContext {
            action: ActionMessage {
                id: Some("t1".into()),
                name: action_name.into(),
                source: None,
                payload: None,
                optimistic: None,
            },
            db: mock_db(),
            session: anonymous_session(),
        }
    }

    #[tokio::test]
    async fn exer_02_tick_route_is_reachable() {
        // Plan 19-01 stub returned Ok(vec![]); Plan 19-03 real handler returns
        // a single PatchMessage. Assert the underlying reachability property
        // that survives both eras: dispatch produces no NotFound error.
        let router = register_gallery_actions(ActionRouter::new());
        let result = router.dispatch(ctx_for("gallery-demo/exer-02/tick")).await;

        for msg in &result {
            if let ProtocolMessage::Error(err) = msg {
                for e in &err.errors {
                    assert!(
                        !e.message.contains("not found"),
                        "gallery-demo/exer-02/tick must be registered; got: {}",
                        e.message
                    );
                }
            }
        }
    }

    #[tokio::test]
    async fn exer_02_tick_route_is_not_a_not_found_error() {
        // Belt-and-suspenders: explicitly assert no NotFound-shaped error.
        // If the route were unregistered, dispatch would return a single
        // ProtocolMessage::Error whose message contains "Action not found"
        // (see marionette::router::tests::router_returns_not_found_for_unknown).
        let router = register_gallery_actions(ActionRouter::new());
        let result = router.dispatch(ctx_for("gallery-demo/exer-02/tick")).await;
        for msg in &result {
            if let ProtocolMessage::Error(err) = msg {
                for e in &err.errors {
                    assert!(
                        !e.message.contains("not found"),
                        "exer-02/tick should not produce a NotFound error; got: {}",
                        e.message
                    );
                }
            }
        }
    }

    #[tokio::test]
    async fn all_seven_phase19_exerciser_routes_are_reachable() {
        // Each of the 7 registered routes must NOT produce a NotFound error
        // when dispatched. Broader reachability guard for Wave 2 plans.
        //
        // Plan 19-01 stubs returned Ok(vec![]); Plan 19-03 real handlers for
        // exer-02 return PatchMessages. Future plans may fill in exer-01/03.
        // The invariant surviving all eras is "the route is registered".
        let dispatcher = register_gallery_actions(ActionRouter::new());
        let route_names = [
            "gallery-demo/exer-01/report",
            "gallery-demo/exer-02/start",
            "gallery-demo/exer-02/pause",
            "gallery-demo/exer-02/reset",
            "gallery-demo/exer-02/tick",
            "gallery-demo/exer-03/report-perf",
            "gallery-demo/exer-03/remeasure",
        ];
        for name in route_names {
            let result = dispatcher.dispatch(ctx_for(name)).await;
            for msg in &result {
                if let ProtocolMessage::Error(err) = msg {
                    for e in &err.errors {
                        assert!(
                            !e.message.contains("not found"),
                            "route {name} must be registered; got: {}",
                            e.message
                        );
                    }
                }
            }
        }
    }
}
