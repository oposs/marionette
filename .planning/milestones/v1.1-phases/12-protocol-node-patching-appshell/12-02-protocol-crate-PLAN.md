---
phase: 12
plan: 02
type: execute
wave: 1
depends_on: [12-01]
files_modified:
  - backend/crates/marionette-protocol/src/data.rs
  - backend/crates/marionette-protocol/src/messages.rs
  - backend/crates/marionette-protocol/src/lib.rs
  - backend/crates/marionette/src/ws.rs
autonomous: true
requirements: [PATCH-01, PATCH-03]
nyquist_compliant: true
tags: [protocol, rust, serde, backend]
must_haves:
  truths:
    - "PatchOperation is a tagged enum serialized with the `op` discriminator"
    - "PatchOperation has exactly 6 variants: Set, SetNode, DeleteNode, SetChildren, InsertChild, RemoveChild"
    - "PatchMessage has a required `surface: Surface` field"
    - "HelloMessage.version emission site in ws.rs reports 1.1.0"
    - "All protocol crate tests pass, including new round-trip tests per variant"
  artifacts:
    - path: "backend/crates/marionette-protocol/src/data.rs"
      provides: "tagged PatchOperation enum"
      contains: "enum PatchOperation"
    - path: "backend/crates/marionette-protocol/src/messages.rs"
      provides: "PatchMessage with required surface field"
      contains: "pub surface: Surface"
    - path: "backend/crates/marionette/src/ws.rs"
      provides: "HelloMessage 1.1.0 emission"
      contains: "1.1.0"
  key_links:
    - from: "data.rs PatchOperation"
      to: "serde tag attribute"
      via: "serde(tag = op, rename_all = kebab-case)"
      pattern: "tag\\s*=\\s*\"op\""
    - from: "messages.rs PatchMessage"
      to: "Surface type"
      via: "pub surface: Surface"
      pattern: "pub\\s+surface:\\s*Surface"
---

<objective>
Rewrite the `marionette-protocol` crate so `PatchOperation` is a tagged enum with 6 variants, `PatchMessage` carries a required `surface: Surface` field, and the `HelloMessage` emission in `ws.rs` reports protocol version `"1.1.0"`. Matches locked decisions D-A1, D-A2, D-A3, D-A5, D-A7 verbatim.

Purpose: This is the structural root of Part A. Every downstream consumer (frontend types, YAML schemas, CRM handlers, docs) mirrors whatever lands here. No back-compat shims — the existing `{path, value}` shape is deleted, call sites migrate.

Output: Compiled protocol crate with new enum + new field + updated version, plus a comprehensive unit-test suite proving each variant round-trips losslessly.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/phases/12-protocol-node-patching-appshell/12-CONTEXT.md
@.planning/phases/12-protocol-node-patching-appshell/12-RESEARCH.md
@backend/crates/marionette-protocol/src/data.rs
@backend/crates/marionette-protocol/src/messages.rs
@backend/crates/marionette-protocol/src/common.rs
@backend/crates/marionette-protocol/src/component.rs
@backend/crates/marionette-protocol/src/lib.rs
@backend/crates/marionette/src/ws.rs

<interfaces>
Existing `ProtocolMessage` already uses the internally-tagged pattern (messages.rs:12-13):

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ProtocolMessage { ... }
```

`PatchOperation` follows the same idiom but with tag = "op" and `rename_all = "kebab-case"` (because variant names in wire format include hyphens like `set-node`).

Existing `Component` struct uses a plain `pub r#type: String` field (component.rs) — NOT a tagged enum — so there is no discriminator collision between `Component.type` and `PatchOperation.op`.

Existing `Surface` type alias (common.rs) is `pub type Surface = String;`.

The current `ws.rs` emits Hello at line 109:
```rust
ProtocolMessage::Hello(HelloMessage {
    version: "1.0.0".into(),
})
```

`clippy::pedantic` is enabled crate-wide — expect `clippy::large_enum_variant` warning on `SetNode { component: Component }`; resolve with `#[allow(clippy::large_enum_variant)]` at the enum (per CONTEXT Claude's Discretion + RESEARCH Finding 3).
</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Rewrite PatchOperation as tagged enum with 6 variants + per-variant round-trip tests</name>
  <read_first>
    - backend/crates/marionette-protocol/src/data.rs
    - backend/crates/marionette-protocol/src/component.rs
    - backend/crates/marionette-protocol/src/messages.rs (line 13 ProtocolMessage serde pattern)
    - .planning/phases/12-protocol-node-patching-appshell/12-RESEARCH.md Pattern 3
  </read_first>
  <behavior>
    - `PatchOperation::Set { path, value }` serializes to `{"op": "set", "path": "/x", "value": 42}` and deserializes back
    - `PatchOperation::SetNode { id, component }` serializes to `{"op": "set-node", "id": "n-1", "component": {"type": "text-input", ...}}` and deserializes back
    - `PatchOperation::DeleteNode { id }` serializes to `{"op": "delete-node", "id": "n-1"}`
    - `PatchOperation::SetChildren { id, children }` serializes to `{"op": "set-children", "id": "n-1", "children": ["a","b"]}`
    - `PatchOperation::InsertChild { parent, index, child_id }` serializes to `{"op": "insert-child", "parent": "n-1", "index": 0, "childId": "n-2"}` (camelCase on childId only)
    - `PatchOperation::RemoveChild { parent, child_id }` serializes to `{"op": "remove-child", "parent": "n-1", "childId": "n-2"}`
    - Unknown `op` values return a serde deserialization error (strict tagged enum)
    - Each variant round-trips through `serde_json::to_value` then `serde_json::from_value` and compares equal with PartialEq
  </behavior>
  <action>
REPLACE the entire contents of `backend/crates/marionette-protocol/src/data.rs` with the following. Do not preserve the old `PatchOperation` struct — delete it cleanly per D-A7.

```rust
use serde::{Deserialize, Serialize};

use crate::component::Component;

/// A single patch operation applied to a surface.
///
/// Operations inside a `PatchMessage.patch` array are applied in declared order, all-or-nothing.
/// Data operations (`Set`) and node-tree operations can be mixed freely in one batch.
/// Serialized with a `"op"` discriminator tag using kebab-case variant names.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "op", rename_all = "kebab-case")]
#[allow(clippy::large_enum_variant)]
pub enum PatchOperation {
    /// Data op — set a value at a JSON Pointer path in the surface's data store.
    Set {
        path: String,
        value: serde_json::Value,
    },
    /// Node op — replace (or create) the component at this node ID.
    SetNode {
        id: String,
        component: Component,
    },
    /// Node op — delete the node with this ID from the surface's adjacency list.
    DeleteNode { id: String },
    /// Node op — replace the children array of the given node.
    SetChildren {
        id: String,
        children: Vec<String>,
    },
    /// Node op — insert an existing child ID into a parent's children array at `index`.
    InsertChild {
        parent: String,
        index: usize,
        #[serde(rename = "childId")]
        child_id: String,
    },
    /// Node op — remove a child ID from a parent's children array.
    RemoveChild {
        parent: String,
        #[serde(rename = "childId")]
        child_id: String,
    },
}

/// A validation error returned by the server.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ValidationError {
    /// Data path the error relates to (optional for global errors).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,

    /// Human-readable error message.
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::Component;
    use serde_json::json;

    fn sample_component() -> Component {
        Component {
            r#type: "text-input".into(),
            props: Some(json!({"label": "Name"})),
            children: None,
            bind: Some("/user/name".into()),
            action: None,
            visible: None,
        }
    }

    #[test]
    fn patch_op_set_round_trip() {
        let op = PatchOperation::Set {
            path: "/users/u-123/name".into(),
            value: json!("Alice"),
        };
        let v = serde_json::to_value(&op).unwrap();
        assert_eq!(v["op"], "set");
        assert_eq!(v["path"], "/users/u-123/name");
        assert_eq!(v["value"], "Alice");
        let back: PatchOperation = serde_json::from_value(v).unwrap();
        assert_eq!(back, op);
    }

    #[test]
    fn patch_op_set_node_round_trip() {
        let op = PatchOperation::SetNode {
            id: "field-a".into(),
            component: sample_component(),
        };
        let v = serde_json::to_value(&op).unwrap();
        assert_eq!(v["op"], "set-node");
        assert_eq!(v["id"], "field-a");
        assert_eq!(v["component"]["type"], "text-input");
        let back: PatchOperation = serde_json::from_value(v).unwrap();
        assert_eq!(back, op);
    }

    #[test]
    fn patch_op_delete_node_round_trip() {
        let op = PatchOperation::DeleteNode { id: "field-b".into() };
        let v = serde_json::to_value(&op).unwrap();
        assert_eq!(v, json!({"op": "delete-node", "id": "field-b"}));
        let back: PatchOperation = serde_json::from_value(v).unwrap();
        assert_eq!(back, op);
    }

    #[test]
    fn patch_op_set_children_round_trip() {
        let op = PatchOperation::SetChildren {
            id: "form-1".into(),
            children: vec!["a".into(), "b".into(), "c".into()],
        };
        let v = serde_json::to_value(&op).unwrap();
        assert_eq!(v["op"], "set-children");
        assert_eq!(v["id"], "form-1");
        assert_eq!(v["children"], json!(["a", "b", "c"]));
        let back: PatchOperation = serde_json::from_value(v).unwrap();
        assert_eq!(back, op);
    }

    #[test]
    fn patch_op_insert_child_round_trip() {
        let op = PatchOperation::InsertChild {
            parent: "form-1".into(),
            index: 2,
            child_id: "new-field".into(),
        };
        let v = serde_json::to_value(&op).unwrap();
        assert_eq!(
            v,
            json!({"op": "insert-child", "parent": "form-1", "index": 2, "childId": "new-field"})
        );
        let back: PatchOperation = serde_json::from_value(v).unwrap();
        assert_eq!(back, op);
    }

    #[test]
    fn patch_op_remove_child_round_trip() {
        let op = PatchOperation::RemoveChild {
            parent: "form-1".into(),
            child_id: "old-field".into(),
        };
        let v = serde_json::to_value(&op).unwrap();
        assert_eq!(
            v,
            json!({"op": "remove-child", "parent": "form-1", "childId": "old-field"})
        );
        let back: PatchOperation = serde_json::from_value(v).unwrap();
        assert_eq!(back, op);
    }

    #[test]
    fn patch_op_unknown_discriminator_rejected() {
        let v = json!({"op": "swap-root", "id": "x"});
        let result: Result<PatchOperation, _> = serde_json::from_value(v);
        assert!(result.is_err(), "unknown op must be rejected by tagged enum");
    }

    #[test]
    fn validation_error_round_trip() {
        let err = ValidationError {
            path: Some("/email".into()),
            message: "Invalid email format".into(),
        };
        let v = serde_json::to_value(&err).unwrap();
        assert_eq!(v, json!({"path": "/email", "message": "Invalid email format"}));
        let back: ValidationError = serde_json::from_value(v).unwrap();
        assert_eq!(back, err);
    }

    #[test]
    fn validation_error_without_path() {
        let err = ValidationError {
            path: None,
            message: "Server error".into(),
        };
        let v = serde_json::to_value(&err).unwrap();
        assert_eq!(v, json!({"message": "Server error"}));
        let back: ValidationError = serde_json::from_value(v).unwrap();
        assert_eq!(back, err);
    }
}
```

After writing: run `cd backend && cargo build -p marionette-protocol` first, then `cd backend && cargo build --workspace`. EXPECT compile errors at every call site that constructs the old `PatchOperation { path, value }` shape. Rust call sites include `ws.rs` optimistic patches, handlers in `crm-demo/src/handlers/contact.rs`, `company.rs`, etc. For each Rust compile error, migrate the call site to the new `PatchOperation::Set { path, value }` variant. The migration is mechanical: `PatchOperation { path: p, value: v }` → `PatchOperation::Set { path: p, value: v }`. Do NOT introduce new helper constructors — call sites use the enum variant directly.

If `lib.rs` re-exports `PatchOperation` explicitly (not via `pub use data::*`), confirm the re-export still compiles (the enum name is unchanged).
  </action>
  <verify>
    <automated>cd backend &amp;&amp; cargo test -p marionette-protocol data::tests 2&gt;&amp;1 | tail -20</automated>
  </verify>
  <acceptance_criteria>
    - `grep -q 'enum PatchOperation' backend/crates/marionette-protocol/src/data.rs` succeeds
    - `grep -q 'tag = "op"' backend/crates/marionette-protocol/src/data.rs` succeeds
    - `grep -q 'rename_all = "kebab-case"' backend/crates/marionette-protocol/src/data.rs` succeeds
    - `cd backend && cargo test -p marionette-protocol data::tests` exits 0 with 9 tests passing (6 variant round-trips + unknown rejection + 2 validation error tests)
    - `cd backend && cargo build --workspace` exits 0 (all Rust call sites migrated to `PatchOperation::Set`)
    - `cd backend && cargo clippy -p marionette-protocol -- -D warnings` exits 0
  </acceptance_criteria>
  <done>PatchOperation rewritten as tagged enum with 6 variants. All 9+ unit tests pass. Full workspace builds cleanly with clippy pedantic. No references to the old struct shape remain.</done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: Add required surface field to PatchMessage + bump HelloMessage version to 1.1.0</name>
  <read_first>
    - backend/crates/marionette-protocol/src/messages.rs
    - backend/crates/marionette/src/ws.rs (line 109 Hello emission)
    - backend/crates/marionette-protocol/src/common.rs (Surface type alias)
  </read_first>
  <behavior>
    - `PatchMessage { id: None, surface: "main".into(), patch: vec![] }` serializes to `{"type": "patch", "surface": "main", "patch": []}`
    - Deserializing a patch message WITHOUT a `surface` field fails (required)
    - Deserializing with `"surface": "modal"` routes to a PatchMessage whose surface is `"modal"`
    - `ws.rs`'s `HelloMessage` emission uses `"1.1.0"` as the version string
  </behavior>
  <action>
1. In `backend/crates/marionette-protocol/src/messages.rs`, modify `PatchMessage` (currently at lines 57-65). Add a required `pub surface: Surface` field. The imports already include `Surface` via `use crate::common::{MessageId, Surface};`. New struct:

```rust
/// Incremental update via patch operations applied to a single surface.
///
/// A `PatchMessage` targets exactly one surface and carries a batch of
/// `PatchOperation` entries that are applied in declared order, all-or-nothing.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PatchMessage {
    /// Correlation ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<MessageId>,

    /// Target surface name. Required — one message targets exactly one surface.
    pub surface: Surface,

    /// Array of patch operations to apply, in declared order.
    pub patch: Vec<PatchOperation>,
}
```

2. Update the existing `patch_round_trip` test (currently messages.rs:178-195) to pass the new `surface` field and construct via the new variant:

```rust
#[test]
fn patch_round_trip() {
    let msg = ProtocolMessage::Patch(PatchMessage {
        id: None,
        surface: "main".into(),
        patch: vec![PatchOperation::Set {
            path: "/user/name".into(),
            value: json!("Bob"),
        }],
    });

    let json = serde_json::to_value(&msg).unwrap();
    assert_eq!(json["type"], "patch");
    assert_eq!(json["surface"], "main");
    assert_eq!(json["patch"][0]["op"], "set");
    assert_eq!(json["patch"][0]["path"], "/user/name");
    assert_eq!(json["patch"][0]["value"], "Bob");

    let deserialized: ProtocolMessage = serde_json::from_value(json).unwrap();
    assert_eq!(deserialized, msg);
}
```

3. Add two new tests to the `mod tests` block in `messages.rs`:

```rust
#[test]
fn patch_message_surface_required() {
    let v = json!({"type": "patch", "patch": []});
    let result: Result<ProtocolMessage, _> = serde_json::from_value(v);
    assert!(result.is_err(), "PatchMessage without surface must be rejected");
}

#[test]
fn patch_message_targets_non_main_surface() {
    let msg = ProtocolMessage::Patch(PatchMessage {
        id: Some("msg-1".into()),
        surface: "modal".into(),
        patch: vec![PatchOperation::DeleteNode { id: "old-modal".into() }],
    });
    let v = serde_json::to_value(&msg).unwrap();
    assert_eq!(v["surface"], "modal");
    assert_eq!(v["patch"][0]["op"], "delete-node");
    let back: ProtocolMessage = serde_json::from_value(v).unwrap();
    assert_eq!(back, msg);
}
```

4. Update `optional_fields_omitted` test (messages.rs ~line 268) that constructs a `PatchMessage` — add `surface: "main".into()` so it compiles.

5. Update the `action_round_trip` test (~line 197) which uses `OptimisticUpdate { patch: vec![PatchOperation { path, value }] }`. Change to `PatchOperation::Set { path, value }`.

6. In `backend/crates/marionette/src/ws.rs` around line 109, change `version: "1.0.0".into(),` to `version: "1.1.0".into(),`. This is the only emission site in runtime Rust code — confirm with `grep -rn '"1.0.0"' backend/crates/ --include='*.rs'`.

7. If step 6's grep returns any Rust file OTHER than a test fixture or doc comment that references `"1.0.0"` as a version, update it to `"1.1.0"`. Do NOT touch `spec/PROTOCOL.md` or `.planning/**` markdown version references — those live in Plan 03.

8. Run `cd backend && cargo test --workspace` — all tests must pass. Any handler test that constructs `OptimisticUpdate { patch: vec![PatchOperation { path, value }] }` migrates mechanically to `PatchOperation::Set { path, value }`. The compiler error messages guide this.
  </action>
  <verify>
    <automated>cd backend &amp;&amp; cargo test -p marionette-protocol messages::tests::patch_round_trip messages::tests::patch_message_surface_required messages::tests::patch_message_targets_non_main_surface &amp;&amp; grep -q '"1.1.0"' backend/crates/marionette/src/ws.rs &amp;&amp; cargo test --workspace 2&gt;&amp;1 | tail -5</automated>
  </verify>
  <acceptance_criteria>
    - `grep -q 'pub surface: Surface' backend/crates/marionette-protocol/src/messages.rs` succeeds
    - `grep -q '"1.1.0"' backend/crates/marionette/src/ws.rs` succeeds
    - `grep -rn '"1.0.0"' backend/crates/ --include='*.rs'` returns zero non-comment lines (only doc strings or comments may reference the old version)
    - `cd backend && cargo test -p marionette-protocol messages::tests::patch_message_surface_required` exits 0
    - `cd backend && cargo test -p marionette-protocol messages::tests::patch_message_targets_non_main_surface` exits 0
    - `cd backend && cargo test --workspace` exits 0 (entire workspace green)
    - `cd backend && cargo clippy --workspace -- -D warnings` exits 0
  </acceptance_criteria>
  <done>PatchMessage has required `surface: Surface` field. HelloMessage reports 1.1.0. All workspace tests pass. Handler call sites that constructed PatchMessage with the old shape now supply `surface` explicitly.</done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| client→server WebSocket | Clients send ActionMessage/EventMessage; server sends Render/Patch/Hello. This plan changes the server→client Patch shape and the Hello version string. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-12-02 | Tampering | `PatchOperation::SetNode { component: Component }` on wire: a malformed or oversized Component payload could consume memory during deserialization | accept | Patches originate server-side in this architecture; frontend-to-server direction never sends PatchMessage. Clippy `large_enum_variant` allow is documented. v1.1 trusts the server as authoritative; size limits are a v2 concern. |
| T-12-03 | Denial of Service | Strict tagged enum rejects unknown `op` values — a buggy server emitting a future op crashes the client deserializer | accept | Covered by `HelloMessage.version` negotiation in §Stale Client Handling. The `patch_op_unknown_discriminator_rejected` test proves the strict behavior is intentional. Documented in Plan 03 PROTOCOL.md updates. |
| T-12-04 | Information Disclosure | `PatchMessage.surface` field enables targeting arbitrary surfaces — could a malicious server push Patch into a surface the user is not viewing? | mitigate | The server is authoritative, so the concept of "malicious server" is moot for the normal threat model. The frontend applies patches per `msg.surface` without additional auth gating because the whole session runs under the user's auth. Frontend acceptance test `patch_message_targets_non_main_surface` proves routing works. |
</threat_model>

<verification>
- `cd backend && cargo test -p marionette-protocol` exits 0 with new variant tests passing
- `cd backend && cargo test --workspace` exits 0 (entire Rust workspace green)
- `cd backend && cargo clippy --workspace -- -D warnings` exits 0
- `grep -q 'enum PatchOperation' backend/crates/marionette-protocol/src/data.rs`
- `grep -q 'pub surface: Surface' backend/crates/marionette-protocol/src/messages.rs`
- `grep -q '"1.1.0"' backend/crates/marionette/src/ws.rs`
- `grep -rn 'PatchOperation {' backend/crates/ --include='*.rs'` returns zero hits (old struct syntax) — only `PatchOperation::Variant { ... }` uses remain
</verification>

<success_criteria>
- `PatchOperation` is a tagged enum with 6 variants matching D-A2 exactly
- Each variant has an inline round-trip test asserting wire-level JSON shape
- `PatchMessage.surface` is required and enforced by a deserialization-failure test
- `HelloMessage.version` emission reports `"1.1.0"` in `ws.rs`
- Full `cargo test --workspace` is green
- `cargo clippy --workspace -- -D warnings` is green
- No `PatchOperation { path, value }` struct-construction syntax remains in backend crates
</success_criteria>

<output>
After completion, create `.planning/phases/12-protocol-node-patching-appshell/12-02-SUMMARY.md` recording:
- Count of Rust call sites migrated from `PatchOperation { path, value }` to `PatchOperation::Set { path, value }` (per crate)
- Any clippy warnings that required `#[allow]` beyond `large_enum_variant`
- Confirmed test counts: protocol crate test count before and after
</output>
