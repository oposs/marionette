---
date: "2026-04-10 08:35"
promoted: false
---

Pre-existing bug: crm-demo sidebar has no icons. NavItem.svelte correctly supports props.icon via Phase 11 icon registry, but backend/crates/crm-demo/src/main.rs lines 136-165 never call .icon(...) on NavItems. Fix: add home/users/building2/list/shield lucide icon names to each NavItem::new(...).icon(...) call. Pure demo-config gap, no component changes needed.
