# trybuild UI fixtures

The `.stderr` files in this directory are regenerated via:

    cd backend
    TRYBUILD=overwrite cargo test -p gallery-smoke --test ui_errors

Review the `git diff` on `*.stderr` afterwards — a new rustc stable may change
diagnostic formatting, but the key-rule substrings (see the table below) MUST
still appear in each regenerated file. If they don't, Plan 02's
`marionette-macros/src/gallery_demo.rs` error messages have drifted and need
correction.

## Fixture rule map

| File | Rule violated | Required substring in .stderr |
|------|---------------|-------------------------------|
| `fail_not_pub.rs` | `pub fn` visibility | `requires \`pub fn\` visibility` |
| `fail_wrong_signature.rs` | zero arguments | `fn must be \`fn() -> Node\` with zero arguments` |
| `fail_wrong_return.rs` | returns `Node` | `fn must return \`Node\`` |
| `fail_applied_to_struct.rs` | item kind | `can only be applied to \`pub fn\` items` |

Per RESEARCH.md §5 "`.stderr` rustc-version sensitivity", a pinned toolchain
is NOT used; we rely on the committed `.stderr` matching CI's stable rustc.
