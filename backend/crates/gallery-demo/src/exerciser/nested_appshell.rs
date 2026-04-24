//! EXER-01 — Nested AppShell exerciser (Plan 19-02).
//!
//! Per 19-CONTEXT.md §D-1: this demo ships a REAL nested AppShell invocation,
//! surfaces the G-02 shadcn Sidebar.Provider collision as live evidence, and
//! drafts a v1.3 framework extension via `.planning/seeds/v1.3-appshell-nestability.md`.
//! It does NOT attempt a v1.2 fix.
//!
//! Three Cards: Structural preview (real nested shell), Observation matrix (4
//! dimensions), v1.3 proposal (CTA toasts the seed path).
//!
//! IMPORTANT BUILDER CONTRACT — divergence from 19-PLAN.md:
//! - `AppShell::new().sidebar(...)` takes a SINGLE `(String, Component)` tuple
//!   per slot (not `Vec<Node>`). The plan's example was incorrect; we wrap
//!   each slot's children in a `Container` before passing. See navigate.rs for
//!   the canonical usage pattern.
//! - There is no `Badge` builder in `marionette::builders`. The observation
//!   matrix cells render the badge label as a `Text` node whose parent
//!   `Container` owns the status-chrome classes (left border stripe etc.).
//! - There is no `ComponentAction::click_with_payload`. Payload rides in the
//!   `extra` serde-flatten map via `action.extra.insert("payload", ...)` —
//!   the same pattern `home.rs` and `navigate.rs` use for gallery-show tiles.

use marionette::builders::{AppShell, Button, Container, Heading, NavItem, Text};
use marionette::gallery::Node;
use marionette_protocol::ComponentAction;

// UI-SPEC §Spacing Scale locks these class strings.
const OUTER_CLASS: &str = "flex flex-col gap-6 p-6";
const CARD_CLASS: &str =
    "rounded-lg border bg-card text-card-foreground shadow-sm p-6 flex flex-col gap-4";
const MATRIX_GRID_CLASS: &str = "grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-3";
const MATRIX_CELL_FAIL_CLASS: &str =
    "rounded-md border border-border bg-card p-3 flex flex-col gap-2 border-l-4 border-l-destructive";
const MATRIX_CELL_WARN_CLASS: &str =
    "rounded-md border border-border bg-card p-3 flex flex-col gap-2 border-l-4 border-l-muted-foreground";
const INNER_CONTENT_CLASS: &str = "p-8 bg-muted/30 rounded-md";
const BUTTON_ROW_CLASS: &str = "flex gap-2";
// Badge-look helper classes (no Badge builder exists — UI-SPEC §Copywriting
// Contract §EXER-01 bullets "Badge chrome" lock the visuals; we map them to
// Container+Text with these chrome classes. UI-SPEC line 186 FAIL:
// "Badge variant `destructive` with text `FAIL`"; line 187 WARN:
// "Badge variant `secondary` with text `WARN`".)
const BADGE_FAIL_CLASS: &str = "inline-flex items-center gap-1 rounded-md px-2 py-0.5 text-xs \
                                font-semibold bg-destructive/10 text-destructive";
const BADGE_WARN_CLASS: &str = "inline-flex items-center gap-1 rounded-md px-2 py-0.5 text-xs \
                                font-semibold bg-muted text-muted-foreground";

// ---------- Top-level demo fn ----------

#[cfg(feature = "gallery")]
#[marionette_macros::gallery_demo(key = "exer-01", name = "Exerciser: Nested AppShell")]
#[must_use]
pub fn gallery_demo() -> Vec<Node> {
    // --- Title + intro (UI-SPEC §Copywriting lines 151-152 verbatim) ---
    let title = Heading::new("Nested AppShell")
        .id("exer-01-title")
        .level(1)
        .build();
    let intro = Text::new(
        "Outer AppShell hosts an inner AppShell in its content slot. This \
         screen documents the broken-nesting state we know about (shadcn \
         SidebarProvider context collision, mobile-sheet behaviour, keyboard \
         shortcut scoping, --sidebar-* token inheritance) and drafts a v1.3 \
         framework-extension proposal — it does not attempt a v1.2 fix.",
    )
    .id("exer-01-intro")
    .build();

    // --- Build three Cards via helper fns. Each returns (card_root, descendants). ---
    let (card1_root, card1_desc) = build_structural_preview_card();
    let (card2_root, card2_desc) = build_observation_matrix_card();
    let (card3_root, card3_desc) = build_v13_proposal_card();

    // --- Outer root flatten (feedback.rs pattern, lines 49-81) ---
    let (outer_root, outer_direct) = Container::new()
        .id("exer-01-root")
        .class(OUTER_CLASS)
        .children(vec![title, intro, card1_root, card2_root, card3_root])
        .build_tree();

    let mut out: Vec<Node> = Vec::with_capacity(
        1 + outer_direct.len() + card1_desc.len() + card2_desc.len() + card3_desc.len(),
    );
    out.push(outer_root);
    out.extend(outer_direct);
    out.extend(card1_desc);
    out.extend(card2_desc);
    out.extend(card3_desc);
    out
}

// ---------- Card 1: Structural preview (real nested AppShell) ----------

#[cfg(feature = "gallery")]
fn build_structural_preview_card() -> (Node, Vec<Node>) {
    // Card heading + legend
    let heading = Heading::new("Structural preview")
        .id("exer-01-card-1-h2")
        .level(2)
        .build();
    let legend = Text::new(
        "Below: an actual AppShell::new() invocation rendered inside the outer \
         gallery AppShell's content slot. The visual breakage IS the point — \
         see the observation matrix for captured findings.",
    )
    .id("exer-01-card-1-legend")
    .build();

    // --- Inner AppShell slot roots ---
    // Sidebar: 3 NavItems, wrapped in a Container (AppShell slot = 1 Node).
    let inner_nav_1 = NavItem::new("Dashboard", "/demo/exer-01/inner-nav/dashboard")
        .id("exer-01-inner-nav-1")
        .action(ComponentAction::click("gallery-demo/noop"))
        .build();
    let inner_nav_2 = NavItem::new("Reports", "/demo/exer-01/inner-nav/reports")
        .id("exer-01-inner-nav-2")
        .action(ComponentAction::click("gallery-demo/noop"))
        .build();
    let inner_nav_3 = NavItem::new("Settings", "/demo/exer-01/inner-nav/settings")
        .id("exer-01-inner-nav-3")
        .action(ComponentAction::click("gallery-demo/noop"))
        .build();
    let (inner_sidebar_root, inner_sidebar_desc) = Container::new()
        .id("exer-01-inner-sidebar")
        .children(vec![inner_nav_1, inner_nav_2, inner_nav_3])
        .build_tree();

    // Header slot: single Heading inside a Container so AppShell gets a single
    // slot Node. (The header slot accepts `(String, Component)`; wrap for
    // consistency with sidebar and to give the header stable chrome.)
    let inner_header_h = Heading::new("Inner shell header")
        .id("exer-01-inner-header-h")
        .level(3)
        .build();
    let (inner_header_root, inner_header_desc) = Container::new()
        .id("exer-01-inner-header")
        .children(vec![inner_header_h])
        .build_tree();

    // Footer slot: single Text wrapped.
    let inner_footer_t = Text::new("Inner shell footer")
        .id("exer-01-inner-footer-t")
        .build();
    let (inner_footer_root, inner_footer_desc) = Container::new()
        .id("exer-01-inner-footer")
        .children(vec![inner_footer_t])
        .build_tree();

    // Main slot: single Text wrapped.
    let inner_main_t = Text::new(
        "Inner main content. This area sits inside the outer gallery's main \
         slot; the observation matrix below documents what breaks under this \
         nesting.",
    )
    .id("exer-01-inner-main-t")
    .build();
    let (inner_main_root, inner_main_desc) = Container::new()
        .id("exer-01-inner-main")
        .children(vec![inner_main_t])
        .build_tree();

    // Invoke the REAL AppShell. Contrast: the Phase 17 static-preview workaround
    // in backend/crates/marionette/src/builders/app_shell.rs::gallery_demo uses
    // Container + 5 labeled slot-boxes — NOT a real AppShell. EXER-01 must do
    // the opposite: force the collision by invoking AppShell::new() literally.
    let mut inner_descendants: Vec<Node> = Vec::new();
    inner_descendants.extend(inner_sidebar_desc);
    inner_descendants.extend(inner_header_desc);
    inner_descendants.extend(inner_footer_desc);
    inner_descendants.extend(inner_main_desc);
    let inner_shell_tree: Vec<Node> = AppShell::new()
        .id("exer-01-inner-shell")
        .sidebar(inner_sidebar_root)
        .header(inner_header_root)
        .footer(inner_footer_root)
        .main(inner_main_root)
        .with_descendants(inner_descendants)
        .build_with_children();
    // inner_shell_tree[0] is the inner AppShell root; [1..] are descendants.

    // Wrap the inner_shell_tree in an INNER_CONTENT_CLASS container so the
    // inner shell has a visible padded area (UI-SPEC §Per-Screen Anatomy
    // line 360 — `p-8 bg-muted/30`).
    let (inner_wrap_root, inner_wrap_desc) = Container::new()
        .id("exer-01-inner-wrap")
        .class(INNER_CONTENT_CLASS)
        .children(inner_shell_tree)
        .build_tree();

    // Card = outer container with CARD_CLASS wrapping [heading, legend, inner_wrap_root]
    let (card_root, card_direct) = Container::new()
        .id("exer-01-card-1-structural-preview")
        .class(CARD_CLASS)
        .children(vec![heading, legend, inner_wrap_root])
        .build_tree();

    let mut desc = Vec::with_capacity(card_direct.len() + inner_wrap_desc.len());
    desc.extend(card_direct);
    desc.extend(inner_wrap_desc);
    (card_root, desc)
}

// ---------- Card 2: Observation matrix (4 dimension cells) ----------

#[cfg(feature = "gallery")]
fn build_observation_matrix_card() -> (Node, Vec<Node>) {
    let heading = Heading::new("Observation matrix")
        .id("exer-01-card-2-h2")
        .level(2)
        .build();

    // Four dimension cells. Copy verbatim from UI-SPEC §EXER-01 Observation matrix copy
    // (lines 174-183). State defaults are authored here; the frontend probe
    // (observe.svelte.ts) overwrites them via /demo/exer-01/matrix/{dim} on mount.
    let cell_provider = build_matrix_cell(
        "exer-01-matrix-provider-context",
        "Provider context",
        MATRIX_CELL_FAIL_CLASS,
        BADGE_FAIL_CLASS,
        "triangle-alert",
        "FAIL",
        "/demo/exer-01/matrix/provider-context/details",
    );
    let cell_mobile = build_matrix_cell(
        "exer-01-matrix-mobile-sheet",
        "Mobile sheet",
        MATRIX_CELL_FAIL_CLASS,
        BADGE_FAIL_CLASS,
        "triangle-alert",
        "FAIL",
        "/demo/exer-01/matrix/mobile-sheet/details",
    );
    let cell_kbd = build_matrix_cell(
        "exer-01-matrix-keyboard-shortcuts",
        "Keyboard shortcuts",
        MATRIX_CELL_FAIL_CLASS,
        BADGE_FAIL_CLASS,
        "triangle-alert",
        "FAIL",
        "/demo/exer-01/matrix/keyboard-shortcuts/details",
    );
    let cell_tokens = build_matrix_cell(
        "exer-01-matrix-sidebar-tokens",
        "--sidebar-* tokens",
        MATRIX_CELL_WARN_CLASS,
        BADGE_WARN_CLASS,
        "triangle-alert", // WARN also uses triangle-alert per UI-SPEC icon catalog
        "WARN",
        "/demo/exer-01/matrix/sidebar-tokens/details",
    );

    let (grid_root, grid_desc) = Container::new()
        .id("exer-01-matrix-grid")
        .class(MATRIX_GRID_CLASS)
        .children(vec![
            cell_provider.0,
            cell_mobile.0,
            cell_kbd.0,
            cell_tokens.0,
        ])
        .build_tree();

    // Flatten descendants of each cell (each cell is a Container tree).
    let mut cell_descendants: Vec<Node> = Vec::new();
    cell_descendants.extend(cell_provider.1);
    cell_descendants.extend(cell_mobile.1);
    cell_descendants.extend(cell_kbd.1);
    cell_descendants.extend(cell_tokens.1);

    let (card_root, card_direct) = Container::new()
        .id("exer-01-card-2-observation-matrix")
        .class(CARD_CLASS)
        .children(vec![heading, grid_root])
        .build_tree();

    let mut desc =
        Vec::with_capacity(card_direct.len() + grid_desc.len() + cell_descendants.len());
    desc.extend(card_direct);
    desc.extend(grid_desc);
    desc.extend(cell_descendants);
    (card_root, desc)
}

/// Build one observation-matrix cell: dimension heading + badge-like chip +
/// findings Text. Findings Text is bound to a data path so the frontend
/// probe can overwrite it at mount.
#[cfg(feature = "gallery")]
fn build_matrix_cell(
    id: &str,
    dimension_name: &str,
    cell_class: &str,
    badge_class: &str,
    icon_name: &str,
    badge_text: &str,
    findings_bind: &str,
) -> (Node, Vec<Node>) {
    let dim_heading = Heading::new(dimension_name)
        .id(format!("{id}-heading"))
        .level(3)
        .build();
    // Badge chrome: Container with icon + Text label. See BADGE_*_CLASS constants.
    let badge_label = Text::new(badge_text)
        .id(format!("{id}-badge-label"))
        .build();
    let (badge_root, badge_desc) = Container::new()
        .id(format!("{id}-badge"))
        .class(badge_class)
        .icon(icon_name)
        .children(vec![badge_label])
        .build_tree();

    let findings = Text::new("") // overwritten at probe time / pre-seeded from show.rs
        .id(format!("{id}-findings"))
        .bind(findings_bind)
        .build();

    let (cell_root, cell_direct) = Container::new()
        .id(id)
        .class(cell_class)
        .children(vec![dim_heading, badge_root, findings])
        .build_tree();

    let mut desc = Vec::with_capacity(cell_direct.len() + badge_desc.len());
    desc.extend(cell_direct);
    desc.extend(badge_desc);
    (cell_root, desc)
}

// ---------- Card 3: v1.3 proposal ----------

#[cfg(feature = "gallery")]
fn build_v13_proposal_card() -> (Node, Vec<Node>) {
    let heading = Heading::new("v1.3 proposal")
        .id("exer-01-card-3-h2")
        .level(2)
        .build();
    let body = Text::new(
        "The v1.3 seed at .planning/seeds/v1.3-appshell-nestability.md proposes \
         a scoped-surface-name framework extension: each AppShell gets a unique \
         surface key, SidebarProvider scopes its context by that key, and CSS \
         tokens resolve via :where([data-surface=\"<key>\"]). The drafted scope \
         is enough to spin up a v1.3 phase without restarting research.",
    )
    .id("exer-01-card-3-body")
    .build();

    // ComponentAction::click_with_payload does NOT exist — use the extra map
    // (flattened serde) the same way home.rs::handle_home and
    // navigate.rs::handle_navigate attach payloads to gallery-show actions.
    let mut action = ComponentAction::click("gallery-demo/exer-01/open-seed");
    action.extra.insert(
        "payload".into(),
        serde_json::json!({ "path": ".planning/seeds/v1.3-appshell-nestability.md" }),
    );
    let cta = Button::new("Open seed draft")
        .id("exer-01-open-seed")
        .icon("arrow-left")
        .action(action)
        .build();

    let (cta_row_root, cta_row_desc) = Container::new()
        .id("exer-01-card-3-ctas")
        .class(BUTTON_ROW_CLASS)
        .children(vec![cta])
        .build_tree();

    let (card_root, card_direct) = Container::new()
        .id("exer-01-card-3-v13-proposal")
        .class(CARD_CLASS)
        .children(vec![heading, body, cta_row_root])
        .build_tree();

    let mut desc = Vec::with_capacity(card_direct.len() + cta_row_desc.len());
    desc.extend(card_direct);
    desc.extend(cta_row_desc);
    (card_root, desc)
}

// ---------- Tests ----------

#[cfg(all(test, feature = "gallery"))]
mod tests {
    use super::*;
    use marionette::gallery::registered_demos;

    #[test]
    fn root_id_is_exer_01_root() {
        let v = gallery_demo();
        assert_eq!(v[0].0, "exer-01-root");
    }

    #[test]
    fn outer_class_is_locked_string() {
        let v = gallery_demo();
        let val = serde_json::to_value(&v[0].1).expect("serialize");
        assert_eq!(val["props"]["class"], OUTER_CLASS);
    }

    #[test]
    fn tree_contains_exactly_one_app_shell() {
        // Pitfall 1 regression guard: any revert to Phase 17's static-preview
        // workaround would drop this count to 0.
        let v = gallery_demo();
        let count = v
            .iter()
            .filter(|(_, c)| {
                let s = serde_json::to_value(c).expect("serialize");
                s["type"] == "app-shell"
            })
            .count();
        assert_eq!(
            count, 1,
            "EXER-01 must invoke AppShell::new() exactly once (inner shell)"
        );
    }

    #[test]
    fn observation_matrix_has_four_dimensions() {
        let v = gallery_demo();
        let required = [
            "exer-01-matrix-provider-context",
            "exer-01-matrix-mobile-sheet",
            "exer-01-matrix-keyboard-shortcuts",
            "exer-01-matrix-sidebar-tokens",
        ];
        for id in required {
            assert!(
                v.iter().any(|(nid, _)| nid == id),
                "missing node id: {id}"
            );
        }
    }

    #[test]
    fn v13_proposal_cta_action_is_open_seed() {
        let v = gallery_demo();
        let (_, cta) = v
            .iter()
            .find(|(id, _)| id == "exer-01-open-seed")
            .expect("cta node");
        let json = serde_json::to_value(cta).expect("serialize");
        // Action is a sibling of type (Component::action), not nested under
        // props — confirmed in catalog/feedback.rs tests.
        assert_eq!(json["action"]["name"], "gallery-demo/exer-01/open-seed");
        assert_eq!(json["action"]["type"], "click");
        assert_eq!(
            json["action"]["payload"]["path"],
            ".planning/seeds/v1.3-appshell-nestability.md"
        );
    }

    #[test]
    fn registered_demos_includes_exer_01() {
        let e = registered_demos()
            .find(|e| e.key == "exer-01")
            .expect("exer-01 must be registered via linkme");
        assert_eq!(e.display_name, "Exerciser: Nested AppShell");
    }

    #[test]
    fn matrix_findings_texts_are_bound() {
        // Contract between backend demo and /demo/exer-01/matrix/* seed /
        // probe report: each cell's findings Text MUST carry the matching
        // bind path. Frontend reads from these paths; show.rs seeds them;
        // handlers/exer01.rs::handle_exer01_report overwrites them.
        let v = gallery_demo();
        let expected = [
            (
                "exer-01-matrix-provider-context-findings",
                "/demo/exer-01/matrix/provider-context/details",
            ),
            (
                "exer-01-matrix-mobile-sheet-findings",
                "/demo/exer-01/matrix/mobile-sheet/details",
            ),
            (
                "exer-01-matrix-keyboard-shortcuts-findings",
                "/demo/exer-01/matrix/keyboard-shortcuts/details",
            ),
            (
                "exer-01-matrix-sidebar-tokens-findings",
                "/demo/exer-01/matrix/sidebar-tokens/details",
            ),
        ];
        for (id, bind_path) in expected {
            let (_, c) = v
                .iter()
                .find(|(nid, _)| nid == id)
                .unwrap_or_else(|| panic!("missing findings node {id}"));
            let j = serde_json::to_value(c).expect("serialize");
            assert_eq!(j["bind"], bind_path, "bad bind for {id}: {j}");
        }
    }
}
