import { render } from 'vitest-browser-svelte';
import { expect, test, vi, beforeEach } from 'vitest';
import { createRawSnippet } from 'svelte';
import Form from './Form.svelte';
import { setFullState } from '$lib/store/data.svelte';

// Mock sendAction
vi.mock('$lib/transport/dispatcher', () => ({
	sendAction: vi.fn(),
}));

import { sendAction } from '$lib/transport/dispatcher';

beforeEach(() => {
	vi.clearAllMocks();
});

test('renders as form element', async () => {
	const screen = await render(Form, {
		props: { props: {}, surface: 'test' },
	});

	const form = screen.baseElement.querySelector('form');
	expect(form).toBeTruthy();
});

test('prevents default submit and dispatches action', async () => {
	const screen = await render(Form, {
		props: {
			props: {},
			action: { type: 'submit', name: 'save-form' },
			surface: 'test',
		},
	});

	const form = screen.baseElement.querySelector('form') as HTMLFormElement;
	// Dispatch submit event on the form
	form.dispatchEvent(new SubmitEvent('submit', { bubbles: true, cancelable: true }));

	expect(sendAction).toHaveBeenCalledWith('save-form', {}, undefined);
});

// -----------------------------------------------------------------------------
// Phase 14 Plan 02 (D-A3 + 14-UI-SPEC.md §Spacing Scale rule 4) — Form.svelte
// wraps children in a Field.Group with `space-y-6` (24px vertical rhythm
// between sibling FieldSets) and upgrades the error banner to the UI-SPEC
// Form banner styling (`bg-destructive/10 border border-destructive/50
// text-destructive rounded-md p-4 mb-4`, per-error `<p class="text-sm">`).
// -----------------------------------------------------------------------------

test('children are wrapped in a Field.Group with class space-y-6', async () => {
	const childSnippet = createRawSnippet(() => ({
		render: () => '<div id="form-child-node">hi</div>',
	}));
	const screen = await render(Form, {
		props: {
			props: {},
			surface: 'test-fg',
			children: childSnippet,
		},
	});
	const child = screen.baseElement.querySelector('#form-child-node');
	expect(child).toBeTruthy();
	// Field.Group renders a <div data-slot="field-group"> — the child's direct
	// parent MUST be that Field.Group, and its class list MUST contain
	// `space-y-6` (the 24px sibling-FieldSet rhythm per UI-SPEC rule 4).
	const parent = child!.parentElement as HTMLElement;
	expect(parent.getAttribute('data-slot')).toBe('field-group');
	expect(parent.className).toContain('space-y-6');
});

test('error banner uses UI-SPEC styling (bg-destructive/10, border-destructive/50, two messages)', async () => {
	// Seed form-level errors under a dedicated form bind.
	setFullState('test-form-errors', { _errors: { formBind: ['Oops', 'Second'] } });
	const screen = await render(Form, {
		props: {
			props: {},
			bind: '/formBind',
			surface: 'test-form-errors',
		},
	});
	const form = screen.baseElement.querySelector('form') as HTMLFormElement;
	expect(form).toBeTruthy();
	// The banner is the first <div> child under <form>.
	const banner = form.querySelector('div');
	expect(banner).toBeTruthy();
	const bannerClasses = banner!.className;
	expect(bannerClasses).toContain('bg-destructive/10');
	expect(bannerClasses).toContain('border-destructive/50');
	expect(bannerClasses).toContain('text-destructive');
	expect(bannerClasses).toContain('rounded-md');
	expect(bannerClasses).toContain('p-4');
	expect(bannerClasses).toContain('mb-4');
	// Both error messages render as separate <p> children.
	const paras = banner!.querySelectorAll('p');
	expect(paras.length).toBe(2);
	expect(paras[0].textContent).toContain('Oops');
	expect(paras[1].textContent).toContain('Second');
	// Per-paragraph styling: text-sm (color cascades from parent).
	expect(paras[0].className).toContain('text-sm');
});
