---
status: awaiting_human_verify
trigger: "SDUI adjacency list rendering bugs - duplicate rendering, broken nav, broken forms"
created: 2026-03-23T00:00:00Z
updated: 2026-03-23T00:00:00Z
---

## Current Focus

hypothesis: build_with_children() returns flat vec of ALL nodes. When this vec is passed to another component's .children(), all descendants become direct children of the parent, causing duplicate rendering. NavItem hardcodes sendAction('navigate') ignoring the action prop.
test: Fix builder API, fix all handlers, fix NavItem
expecting: Correct adjacency lists with only direct children per node; nav dispatches correct actions
next_action: Implement build_child() method and fix all handler files

## Symptoms

expected: Each node's children array contains ONLY direct child IDs. Nav items dispatch their configured actions.
actual: Parent nodes list both children AND grandchildren. Nav always sends 'navigate' action regardless of action prop.
errors: Duplicate component rendering, broken sidebar navigation, broken form layouts
reproduction: Any page with nested components (forms inside containers)
started: Since initial implementation

## Eliminated

(none yet)

## Evidence

- timestamp: 2026-03-23T00:00:00Z
  checked: component_builder.rs build_with_children()
  found: Returns flat Vec including parent + all children. When this vec is spread into another component's .children(), ALL nodes become direct children of the parent.
  implication: Root cause of duplicate rendering confirmed.

- timestamp: 2026-03-23T00:00:00Z
  checked: NavItem.svelte handleClick
  found: handleClick hardcodes sendAction('navigate', { path: ... }) - ignores the action prop entirely
  implication: Root cause of broken sidebar navigation confirmed.

- timestamp: 2026-03-23T00:00:00Z
  checked: All handler files
  found: Multiple patterns of bug: (1) build_with_children() result spread into another .children() call, (2) manual indexing like form_nodes[0].clone() to work around the issue
  implication: All handlers need fixing

## Resolution

root_cause: Two bugs: (1) build_with_children() returns flat list that gets incorrectly spread into parent .children(), causing all descendants to be listed as direct children of grandparent nodes. (2) NavItem.svelte hardcodes sendAction('navigate') ignoring the action prop from the backend.
fix: Added build_tree() method to component builder that returns (root_tuple, descendants) -- root_tuple for passing to parent .child()/.children(), descendants for flat HashMap collection. Fixed all 6 handler files to use build_tree() instead of spreading build_with_children() into parent .children(). Fixed NavItem.svelte to dispatch the action prop when present.
verification: cargo build -p crm-demo --release succeeds. cargo test -p marionette-protocol -p marionette-macros passes all 15 tests.
files_changed: [crates/marionette-macros/src/component_builder.rs, crates/crm-demo/src/main.rs, crates/crm-demo/src/handlers/contact.rs, crates/crm-demo/src/handlers/company.rs, crates/crm-demo/src/handlers/user.rs, crates/crm-demo/src/handlers/audit.rs, crates/crm-demo/src/handlers/interaction.rs, frontend/src/lib/components/nav/NavItem.svelte]
