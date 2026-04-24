//! EXER-01 Nested AppShell nestability probe (Plan 19-02).
//!
//! Runs at mount of the inner AppShell and reports 4 observation dimensions
//! to the backend via `gallery-demo/exer-01/report`.
//!
//! Source: 19-RESEARCH.md §Code Examples Example 1. Depends on outer
//! Sidebar.Provider exposing its state handle via `window.__mrnExer01OuterSidebar`
//! — see `frontend/src/lib/components/ui/sidebar/sidebar-provider.svelte`
//! (first-mount guard ensures the inner provider does NOT clobber the outer).

import { getContext } from 'svelte';
import { sendAction } from '$lib/transport/dispatcher';

const SIDEBAR_KEY = Symbol.for('scn-sidebar');
const SIDEBAR_KEYBOARD_SHORTCUT = 'b';

interface MatrixEntry {
	state: 'PASS' | 'FAIL' | 'WARN';
	details: string;
}

/**
 * Capture 4 nestability observations and send them to the backend.
 *
 * Runs once per inner-AppShell mount. The dimensions are:
 * 1. Provider context identity — inner vs outer `Sidebar.Provider` handle.
 * 2. --sidebar-width CSS custom-property inheritance on the inner wrap.
 * 3. Keyboard shortcut scoping — Ctrl+B toggles which provider?
 * 4. Mobile sheet — present-viewport probe only; breakage is documented.
 */
export async function probeNestability(): Promise<void> {
	// --- Dimension 1: Provider context identity ---
	const innerState = getContext(SIDEBAR_KEY);
	const outerState = (window as unknown as { __mrnExer01OuterSidebar?: unknown })
		.__mrnExer01OuterSidebar;
	const sameIdentity =
		innerState !== undefined && outerState !== undefined && innerState === outerState;

	// --- Dimension 2: --sidebar-width inheritance (inner wrap element's computed style) ---
	const innerRootEl = document.querySelector('#exer-01-inner-wrap');
	const innerWidth = innerRootEl
		? getComputedStyle(innerRootEl as Element)
				.getPropertyValue('--sidebar-width')
				.trim()
		: '';

	// --- Dimension 3: Keyboard shortcut — synthetic dispatch + observe both states ---
	const readOpen = (s: unknown): boolean | null =>
		s && typeof s === 'object' && 'open' in s
			? (s as { open: boolean }).open
			: null;
	const beforeOuter = readOpen(outerState);
	const beforeInner = readOpen(innerState);
	window.dispatchEvent(
		new KeyboardEvent('keydown', {
			key: SIDEBAR_KEYBOARD_SHORTCUT,
			ctrlKey: true,
			bubbles: true
		})
	);
	// Give Svelte a microtask to propagate.
	await Promise.resolve();
	await new Promise((r) => setTimeout(r, 0));
	const afterOuter = readOpen(outerState);
	const afterInner = readOpen(innerState);
	const outerFlipped = beforeOuter !== null && beforeOuter !== afterOuter;
	const innerFlipped = beforeInner !== null && beforeInner !== afterInner;

	// --- Dimension 4: Mobile sheet ---
	const isMobile = window.innerWidth < 768;

	// --- Report — copy locked details verbatim from UI-SPEC §EXER-01 matrix copy ---
	const report: Record<string, MatrixEntry> = {
		'provider-context': {
			state: sameIdentity ? 'WARN' : 'FAIL',
			details: sameIdentity
				? 'Inner and outer providers share identity — unexpected. shadcn <Sidebar.Provider> is global-symbol-keyed.'
				: 'shadcn <Sidebar.Provider> is not scoped: the inner provider shadows outer via Symbol.for("scn-sidebar") (different state objects confirmed).'
		},
		'mobile-sheet': {
			state: isMobile ? 'FAIL' : 'WARN',
			details: isMobile
				? 'Mobile viewport active — inner Sheet layered on outer Sheet; dismiss cascades.'
				: 'Resize viewport below 768px to observe live. UI-SPEC documents the known collision at mobile widths.'
		},
		'keyboard-shortcuts': {
			state: outerFlipped && innerFlipped ? 'FAIL' : 'WARN',
			details:
				outerFlipped && innerFlipped
					? 'Ctrl+B toggled both shells (last-registered wins; non-deterministic).'
					: `Synthetic Ctrl+B observed outerFlipped=${outerFlipped}, innerFlipped=${innerFlipped}. UI-SPEC locks the "both flip" outcome as the known bug.`
		},
		'sidebar-tokens': {
			state: 'WARN',
			details: `Inner --sidebar-width computed="${innerWidth || '(not set)'}" — CSS custom-property inheritance cascades naturally; scoped tokens would need :where([data-surface="<key>"]).`
		}
	};

	sendAction('gallery-demo/exer-01/report', report);
}

// Auto-arm: watch for the inner-wrap to appear, then probe once.
//
// Alternatives considered (rejected):
//   1. ProbeMount.svelte component + backend registry type "exer-01-probe-mount" — requires
//      SDUI-tree coordination and a throwaway registered component type; too invasive for a
//      one-shot probe.
//   2. Inner-AppShell onMount hook — would require framework-crate changes or a bespoke
//      fork of AppShell.svelte just for this probe.
//   3. +layout.svelte route detection — adds routing coupling to a plain observation module.
//
// MutationObserver on document.body is self-contained: it fires when the SDUI patch pipeline
// inserts the inner-wrap node, which is guaranteed to happen exactly once per /#exer-01 mount.
if (typeof window !== 'undefined' && typeof document !== 'undefined') {
	let armed = false;
	const obs = new MutationObserver(() => {
		if (armed) return;
		// Guard against callbacks firing after the host document/window is torn
		// down (e.g., under vitest jsdom where module-level observers outlive the
		// test's `document` reference).
		if (typeof document === 'undefined') return;
		if (document.querySelector('#exer-01-inner-wrap')) {
			armed = true;
			setTimeout(() => {
				void probeNestability();
			}, 100);
			obs.disconnect();
		}
	});
	obs.observe(document.body, { childList: true, subtree: true });
}
