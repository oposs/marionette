import { render } from 'vitest-browser-svelte';
import { expect, test, beforeEach } from 'vitest';
import { tick } from 'svelte';
import { page } from 'vitest/browser';
import Surface from '$lib/components/core/Surface.svelte';
import {
	setSurfaceTree,
	clearSurfaceTree,
} from '$lib/store/surfaces.svelte';
import { setFullState, resetStore } from '$lib/store/data.svelte';
import { registerDefaults } from '$lib/registry/defaults';

const SURFACE = 'appshell-test';

beforeEach(async () => {
	// Force desktop viewport so shadcn Sidebar's mobile-Sheet path is avoided.
	// (At <768px the sidebar uses Sheet.Root which is closed by default — its
	// children are not rendered, breaking slot-content assertions.) The chosen
	// 1280×800 matches a typical desktop test viewport.
	await page.viewport(1280, 800);
	resetStore(SURFACE);
	clearSurfaceTree(SURFACE);
	registerDefaults();
});

test('AppShell renders all slot contents via NodeRenderer', async () => {
	setFullState(SURFACE, {});
	setSurfaceTree(SURFACE, 'shell-root', {
		'shell-root': {
			type: 'app-shell',
			props: {
				sidebarNodeId: 'side-1',
				headerNodeId: 'head-1',
				footerNodeId: 'foot-1',
				mainNodeId: 'main-1',
			},
		},
		'side-1': { type: 'heading', props: { text: 'SidebarContent' } },
		'head-1': { type: 'heading', props: { text: 'HeaderContent' } },
		'foot-1': { type: 'heading', props: { text: 'FooterContent' } },
		'main-1': { type: 'heading', props: { text: 'MainContent' } },
	});

	const screen = render(Surface, { props: { name: SURFACE } });
	await tick();
	await tick();

	// Header / main / footer slots live inside Sidebar.Inset — their text is
	// directly in the main document tree and visible in baseElement.textContent.
	expect(screen.baseElement.textContent).toContain('HeaderContent');
	expect(screen.baseElement.textContent).toContain('FooterContent');
	expect(screen.baseElement.textContent).toContain('MainContent');

	// The sidebar slot's content lives inside Sidebar.Root → Sidebar.Content,
	// which on desktop is wrapped in a fixed-position container whose outer
	// <div> has `hidden md:block` (tailwind responsive visibility). In
	// chromium's default 1280×720 playwright viewport the md breakpoint is
	// active so the container is rendered, but we query the sidebar-inner
	// region directly to prove the NodeRenderer attached the sidebar slot
	// node — independent of responsive visibility classes.
	// The sidebar slot's content lives inside Sidebar.Root → Sidebar.Content.
	// On desktop (viewport ≥768px, forced in beforeEach) it's in a fixed
	// container identified by data-sidebar="sidebar" / data-slot="sidebar-inner".
	const sidebarInner = screen.baseElement.querySelector(
		'[data-slot="sidebar-inner"], [data-sidebar="sidebar"]'
	);
	expect(sidebarInner).not.toBeNull();
	expect(sidebarInner?.textContent).toContain('SidebarContent');
});

test('AppShell with missing slots renders without crashing', async () => {
	setFullState(SURFACE, {});
	setSurfaceTree(SURFACE, 'shell-root', {
		'shell-root': {
			type: 'app-shell',
			props: {
				mainNodeId: 'main-1',
			},
		},
		'main-1': { type: 'heading', props: { text: 'LoneMain' } },
	});

	const screen = render(Surface, { props: { name: SURFACE } });
	await tick();

	expect(screen.baseElement.textContent).toContain('LoneMain');
});

test('AppShell header includes the Sidebar.Trigger (mobile hamburger)', async () => {
	setFullState(SURFACE, {});
	setSurfaceTree(SURFACE, 'shell-root', {
		'shell-root': {
			type: 'app-shell',
			props: { mainNodeId: 'main-1' },
		},
		'main-1': { type: 'heading', props: { text: 'X' } },
	});

	const screen = render(Surface, { props: { name: SURFACE } });
	await tick();

	// Sidebar.Trigger renders as a button with data-sidebar="trigger" (verified
	// against frontend/src/lib/components/ui/sidebar/sidebar-trigger.svelte).
	const triggerCandidates = screen.baseElement.querySelectorAll(
		'button[data-sidebar="trigger"], button[aria-label*="sidebar" i], button[aria-controls*="sidebar" i]'
	);
	expect(triggerCandidates.length).toBeGreaterThanOrEqual(1);
});
