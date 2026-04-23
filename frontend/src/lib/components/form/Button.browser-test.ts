import { render } from 'vitest-browser-svelte';
import { expect, test, vi, beforeEach } from 'vitest';
import Button from './Button.svelte';

// Mock sendAction
vi.mock('$lib/transport/dispatcher', () => ({
	sendAction: vi.fn(),
}));

import { sendAction } from '$lib/transport/dispatcher';

beforeEach(() => {
	vi.clearAllMocks();
});

test('renders with label text', async () => {
	const screen = await render(Button, {
		props: { props: { label: 'Save' }, surface: 'test' },
	});

	await expect.element(screen.getByText('Save')).toBeVisible();
});

test('dispatches action on click', async () => {
	const screen = await render(Button, {
		props: {
			props: { label: 'Submit' },
			action: { type: 'submit', name: 'save-contact' },
			surface: 'test',
		},
	});

	await screen.getByText('Submit').click();

	expect(sendAction).toHaveBeenCalledWith(
		'save-contact',
		expect.any(Object),
		undefined,
		undefined,
	);
});

test('renders disabled state', async () => {
	const screen = await render(Button, {
		props: { props: { label: 'Save', disabled: true }, surface: 'test' },
	});

	const button = screen.getByRole('button');
	await expect.element(button).toBeDisabled();
});

test('renders icon when props.icon set', async () => {
	const screen = await render(Button, {
		props: { props: { label: 'Add', icon: 'plus' }, surface: 'test' },
	});

	await expect.element(screen.getByText('Add')).toBeVisible();
	// Icon should render as SVG inside button
	const button = screen.getByRole('button').element() as HTMLButtonElement;
	expect(button.querySelector('svg')).toBeTruthy();
});

// -----------------------------------------------------------------------------
// Phase 18 Plan 01 (Gap 1) — variant/size/loading/icon/aria_label pass-through.
//
// Plan 18-01 Task 2 rewires Button.svelte to read variant/size directly from
// props instead of deriving variant from color/outline (the Phase 11 legacy).
// The leaf button gallery_demo already passes variant="destructive" since
// Phase 17; prior to this plan that string was IGNORED on the frontend, so the
// "Destructive" demo rendered as a plain default button. These tests lock in
// the new contract: the frontend honors the backend-authoritative variant/size
// strings verbatim (pre-deployment posture, no back-compat color/outline).
// -----------------------------------------------------------------------------

test('passes variant=destructive through to shadcn Button', async () => {
	const screen = await render(Button, {
		props: { props: { label: 'Delete', variant: 'destructive' }, surface: 'test' },
	});
	const button = screen.getByRole('button');
	await expect.element(button).toBeVisible();
	// shadcn destructive variant applies `bg-destructive/10` as the distinctive
	// surface fill. NOTE: the plain /destructive/ regex is too loose because
	// the base button class list already includes `ring-destructive/20` for
	// aria-invalid styling; we must assert the variant-specific class.
	const el = button.element() as HTMLButtonElement;
	expect(el.className).toMatch(/bg-destructive\/10/);
});

test('passes variant=outline through to shadcn Button', async () => {
	const screen = await render(Button, {
		props: { props: { label: 'Cancel', variant: 'outline' }, surface: 'test' },
	});
	const el = screen.getByRole('button').element() as HTMLButtonElement;
	// shadcn outline variant applies border-border class
	expect(el.className).toMatch(/border-border|bg-background/);
});

test('passes size=sm through to shadcn Button', async () => {
	const screen = await render(Button, {
		props: { props: { label: 'Small', size: 'sm' }, surface: 'test' },
	});
	const el = screen.getByRole('button').element() as HTMLButtonElement;
	// shadcn applies h-7 for size=sm per ui/button/button.svelte variants
	expect(el.className).toMatch(/h-7/);
});

test('passes size=lg through to shadcn Button', async () => {
	const screen = await render(Button, {
		props: { props: { label: 'Large', size: 'lg' }, surface: 'test' },
	});
	const el = screen.getByRole('button').element() as HTMLButtonElement;
	expect(el.className).toMatch(/h-9/);
});

test('defaults to variant=default and size=default without props', async () => {
	const screen = await render(Button, {
		props: { props: { label: 'Plain' }, surface: 'test' },
	});
	const el = screen.getByRole('button').element() as HTMLButtonElement;
	// default variant => bg-primary; default size => h-8
	expect(el.className).toMatch(/bg-primary/);
	expect(el.className).toMatch(/h-8/);
});

test('renders Loader2 spinner when loading=true and disables the button', async () => {
	const screen = await render(Button, {
		props: { props: { label: 'Saving', loading: true }, surface: 'test' },
	});
	const btn = screen.getByRole('button');
	await expect.element(btn).toBeDisabled();
	const el = btn.element() as HTMLButtonElement;
	// Loader2 applies animate-spin to its svg
	expect(el.querySelector('.animate-spin')).not.toBeNull();
	// aria-busy flags assistive tech
	expect(el.getAttribute('aria-busy')).toBe('true');
});

test('loading=true hides the icon (spinner replaces it) but keeps label', async () => {
	const screen = await render(Button, {
		props: { props: { label: 'Saving', icon: 'plus', loading: true }, surface: 'test' },
	});
	const el = screen.getByRole('button').element() as HTMLButtonElement;
	// Only one svg in the button — the spinner — not two (not spinner + plus).
	const svgs = el.querySelectorAll('svg');
	expect(svgs.length).toBe(1);
	expect(svgs[0].classList.contains('animate-spin')).toBe(true);
	await expect.element(screen.getByText('Saving')).toBeVisible();
});

test('icon-only Button uses size=icon by default when no explicit size prop', async () => {
	const screen = await render(Button, {
		props: { props: { icon: 'plus' }, surface: 'test' },
	});
	const el = screen.getByRole('button').element() as HTMLButtonElement;
	// shadcn icon size => size-8 (h-8 w-8) per button.svelte variants
	expect(el.className).toMatch(/size-8/);
});

test('icon-only Button gets aria-label from props.aria_label (snake_case)', async () => {
	const screen = await render(Button, {
		props: { props: { icon: 'plus', aria_label: 'Add item' }, surface: 'test' },
	});
	const el = screen.getByRole('button').element() as HTMLButtonElement;
	expect(el.getAttribute('aria-label')).toBe('Add item');
});

test('icon-only Button falls back to icon name for aria-label when aria_label missing', async () => {
	const screen = await render(Button, {
		props: { props: { icon: 'plus' }, surface: 'test' },
	});
	const el = screen.getByRole('button').element() as HTMLButtonElement;
	// No aria_label provided — fallback to the icon kebab name so SR users
	// still get SOME label, even if it's imperfect ("plus").
	expect(el.getAttribute('aria-label')).toBe('plus');
});

test('non-icon-only Button does NOT set aria-label (label text is the accessible name)', async () => {
	const screen = await render(Button, {
		props: { props: { label: 'Save', aria_label: 'ignored' }, surface: 'test' },
	});
	const el = screen.getByRole('button').element() as HTMLButtonElement;
	// When a visible label is present, aria-label must be omitted so the
	// visible text becomes the accessible name (avoids mismatched SR override).
	expect(el.getAttribute('aria-label')).toBeNull();
});

test('does NOT read deprecated props.color or props.outline', async () => {
	// Pre-deployment posture: no back-compat fallback to color/outline.
	// Passing color=red must NOT produce a destructive-variant button.
	const screen = await render(Button, {
		props: { props: { label: 'Legacy', color: 'red' }, surface: 'test' },
	});
	const el = screen.getByRole('button').element() as HTMLButtonElement;
	// Must NOT have the destructive variant surface class (bg-destructive/10);
	// the loose /destructive/ regex would false-positive on the base class's
	// ring-destructive/20 (aria-invalid styling).
	expect(el.className).not.toMatch(/bg-destructive\/10/);
	// Should have default variant instead.
	expect(el.className).toMatch(/bg-primary/);
});
