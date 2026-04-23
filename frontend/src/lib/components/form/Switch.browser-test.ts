/**
 * RED harness for Switch SDUI leaf (scaffolded by Plan 14-01).
 *
 * Switch.svelte does not exist yet — Wave 2 Plan 14-06 creates it. Internal
 * wrap contract per D-B1:
 *   <Field.Field orientation="horizontal" data-invalid>
 *     <Field.Label>{label}</Field.Label>
 *     <Switch bind:checked aria-invalid />
 *     <Field.Error>{err}</Field.Error>
 *   </Field.Field>
 */
import { render } from 'vitest-browser-svelte';
import { expect, test, vi, beforeEach } from 'vitest';

// Phase 18 Plan 02 — blur-action dispatch tests need a sendAction mock.
// Mirrors the pattern used by Checkbox/SelectInput browser tests.
vi.mock('$lib/transport/dispatcher', () => ({
	sendAction: vi.fn(),
}));

import Switch from './Switch.svelte';
import { setData, getData, resetStore, setFullState } from '$lib/store/data.svelte';
import { sendAction } from '$lib/transport/dispatcher';

beforeEach(() => {
	resetStore('test');
	vi.clearAllMocks();
});

test('renders switch with label', async () => {
	const screen = await render(Switch, {
		props: { props: { label: 'Enable' }, surface: 'test' },
	});

	await expect.element(screen.getByText('Enable')).toBeVisible();
	await expect.element(screen.getByRole('switch')).toBeInTheDocument();
});

test('reflects bind boolean value (aria-checked true when /enabled=true)', async () => {
	setData('test', '/enabled', true);

	const screen = await render(Switch, {
		props: { props: { label: 'Enable' }, bind: '/enabled', surface: 'test' },
	});

	const sw = screen.getByRole('switch').element();
	expect(sw.getAttribute('aria-checked')).toBe('true');
});

test('toggling emits setData on bind', async () => {
	setData('test', '/enabled', false);

	const screen = await render(Switch, {
		props: { props: { label: 'Enable' }, bind: '/enabled', surface: 'test' },
	});

	const sw = screen.getByRole('switch').element() as HTMLElement;
	sw.click();
	// Give the click event a tick to flow through.
	await new Promise((r) => setTimeout(r, 50));

	expect(getData('test', '/enabled')).toBe(true);
});

test('data-invalid on wrapper when /_errors/{bind} is set', async () => {
	setData('test', '/_errors/enabled', 'required');

	const screen = await render(Switch, {
		props: { props: { label: 'Enable' }, bind: '/enabled', surface: 'test' },
	});

	await expect.element(screen.getByText('required')).toBeVisible();
	const wrapper = screen.baseElement.querySelector('[data-slot="field"]');
	expect(wrapper?.hasAttribute('data-invalid')).toBe(true);
});

// -----------------------------------------------------------------------------
// Phase 18 Plan 02 — Framework Gap 2: blur-action dispatch.
//
// Same contract as Checkbox: wrap in a `<div onfocusout>` and invoke
// handleBlur on focus-leave. Payload shape is `{ value: <boolean> }`.
// -----------------------------------------------------------------------------

test('blur dispatch: fires sendAction with boolean value on focusout when action.type === "blur"', async () => {
	const surface = 'test-blur-sw-' + crypto.randomUUID();
	setFullState(surface, { enabled: true });
	const screen = await render(Switch, {
		props: {
			props: { label: 'Enable' },
			bind: '/enabled',
			action: { type: 'blur', name: 'validate-switch' },
			surface,
		},
	});

	const wrapper = screen.baseElement.querySelector(
		'[data-slot="field"]'
	) as HTMLElement;
	expect(wrapper).toBeTruthy();
	const rootWrapper = wrapper.parentElement as HTMLElement;
	rootWrapper.dispatchEvent(new FocusEvent('focusout', { bubbles: true }));
	await new Promise((r) => setTimeout(r, 20));

	expect(sendAction).toHaveBeenCalled();
	const calls = (sendAction as ReturnType<typeof vi.fn>).mock.calls;
	const blurCall = calls.find((c) => c[0] === 'validate-switch');
	expect(blurCall).toBeTruthy();
	expect(blurCall![1]).toEqual({ value: true });
	expect(typeof (blurCall![1] as { value: unknown }).value).toBe('boolean');
	expect(blurCall![2]).toBeUndefined();
});

test('blur dispatch: emits false when switch is off', async () => {
	const surface = 'test-blur-sw-false-' + crypto.randomUUID();
	setFullState(surface, { enabled: false });
	const screen = await render(Switch, {
		props: {
			props: { label: 'Enable' },
			bind: '/enabled',
			action: { type: 'blur', name: 'validate-switch' },
			surface,
		},
	});
	const wrapper = screen.baseElement.querySelector(
		'[data-slot="field"]'
	) as HTMLElement;
	(wrapper.parentElement as HTMLElement).dispatchEvent(
		new FocusEvent('focusout', { bubbles: true })
	);
	await new Promise((r) => setTimeout(r, 20));

	const blurCall = (sendAction as ReturnType<typeof vi.fn>).mock.calls.find(
		(c) => c[0] === 'validate-switch'
	);
	expect(blurCall).toBeTruthy();
	expect(blurCall![1]).toEqual({ value: false });
});

test('blur dispatch: does NOT fire when no action is configured', async () => {
	const surface = 'test-noaction-sw-' + crypto.randomUUID();
	setFullState(surface, { enabled: true });
	const screen = await render(Switch, {
		props: {
			props: { label: 'Enable' },
			bind: '/enabled',
			surface,
		},
	});
	const wrapper = screen.baseElement.querySelector(
		'[data-slot="field"]'
	) as HTMLElement;
	(wrapper.parentElement as HTMLElement).dispatchEvent(
		new FocusEvent('focusout', { bubbles: true })
	);
	await new Promise((r) => setTimeout(r, 20));

	expect(sendAction).not.toHaveBeenCalled();
});

test('blur dispatch: does NOT fire when action.type !== "blur"', async () => {
	const surface = 'test-notblur-sw-' + crypto.randomUUID();
	setFullState(surface, { enabled: true });
	const screen = await render(Switch, {
		props: {
			props: { label: 'Enable' },
			bind: '/enabled',
			action: { type: 'change', name: 'other-action' },
			surface,
		},
	});
	const wrapper = screen.baseElement.querySelector(
		'[data-slot="field"]'
	) as HTMLElement;
	(wrapper.parentElement as HTMLElement).dispatchEvent(
		new FocusEvent('focusout', { bubbles: true })
	);
	await new Promise((r) => setTimeout(r, 20));

	expect(sendAction).not.toHaveBeenCalledWith(
		'other-action',
		expect.anything(),
		expect.anything()
	);
});

test('bind value is preserved through focusout (getData reads current boolean)', async () => {
	const surface = 'test-sw-pres-' + crypto.randomUUID();
	setFullState(surface, { enabled: true });
	const screen = await render(Switch, {
		props: {
			props: { label: 'Enable' },
			bind: '/enabled',
			action: { type: 'blur', name: 'validate-switch' },
			surface,
		},
	});
	const wrapper = screen.baseElement.querySelector(
		'[data-slot="field"]'
	) as HTMLElement;
	(wrapper.parentElement as HTMLElement).dispatchEvent(
		new FocusEvent('focusout', { bubbles: true })
	);
	await new Promise((r) => setTimeout(r, 20));
	expect(getData(surface, '/enabled')).toBe(true);
});
