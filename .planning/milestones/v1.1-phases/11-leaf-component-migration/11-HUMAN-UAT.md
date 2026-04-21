---
status: passed
phase: 11-leaf-component-migration
source: [11-VERIFICATION.md]
started: 2026-04-09T18:30:00Z
updated: 2026-04-10T08:30:00Z
---

## Current Test

[all tests complete]

## Tests

### 1. CRM Demo Visual Verification
expected: Start full stack (backend + frontend), open http://localhost:5173, verify: shadcn-styled form fields, table with muted header, Card-sectioned form screen, ArrowLeft back button, shadcn Dialog for ConfirmDialog with focus trap, toasts at bottom-right, no console errors
result: passed — verified 2026-04-10 via Claude-in-Chrome browser automation. Login flow works, DataTable renders with shadcn Table.Root + muted header, form inputs use shadcn Input + Label, Button uses shadcn default variant, SideNav uses ghost buttons, no console errors, zero flowbite references.

## Summary

total: 1
passed: 1
issues: 0
pending: 0
skipped: 0
blocked: 0

## Notes

Two pre-existing bugs observed during visual verification (NOT Phase 11 regressions — tracked separately):

1. **Sidebar missing icons** — crm-demo `backend/crates/crm-demo/src/main.rs` builds NavItems without `.icon(...)` calls. NavItem.svelte correctly supports `props.icon` via the Phase 11 icon registry; the demo backend simply doesn't supply icon names. Added to backlog.

2. **Invalid login password silently ignored** — Chained pre-existing bugs in error display flow: (a) `Form.svelte:21-23` only reads errors when `bind` is set; (b) crm-demo login form does not set `.bind(...)` on Form; (c) `init.ts:67` stores `ValidationError[]` at path typed as `string[]` in Form.svelte (would render `[object Object]` if it reached the template). Phase 11 did not modify any of these files. Added to backlog.

## Gaps
