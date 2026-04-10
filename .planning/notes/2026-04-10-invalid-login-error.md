---
date: "2026-04-10 08:35"
promoted: false
---

Pre-existing bug: invalid password login shows no error message. Three chained bugs: (a) Form.svelte:21-23 only reads errors when `bind` is set; (b) crm-demo login form does not call .bind(...) on Form; (c) init.ts:67 stores ValidationError[] but Form.svelte types it as string[] and would render [object Object]. Backend correctly returns ErrorMessage with "Invalid email or password" — it just never reaches the UI. None of these files were touched in Phase 11. Fix: either add form-level error reading (when bind empty, read /_errors flat) and render error.message, OR have backend put errors under a bound path.
