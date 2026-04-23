/**
 * RED harness for RadioGroup SDUI leaf (scaffolded by Plan 14-01).
 *
 * RadioGroup.svelte does not exist yet — Wave 2 Plan 14-06 creates it. These
 * tests intentionally fail with import-resolve until then. Internal wrap
 * contract per D-B1:
 *   <Field.Field data-invalid>
 *     <Field.Label>{label}</Field.Label>
 *     <RadioGroup.Root bind:value={value}>
 *       {#each options as opt}
 *         <RadioGroup.Item value={opt.value} id={..}/> <Label>{opt.label}</Label>
 *         {#if opt.description}<span class="text-muted-foreground">…</span>{/if}
 *       {/each}
 *     </RadioGroup.Root>
 *     <Field.Error>{err}</Field.Error>
 *   </Field.Field>
 */
import { render } from 'vitest-browser-svelte';
import { expect, test, vi, beforeEach } from 'vitest';

// Phase 18 Plan 02 — blur-action dispatch tests need a sendAction mock.
// Mirrors Checkbox/Switch/SelectInput browser-test pattern.
vi.mock('$lib/transport/dispatcher', () => ({
	sendAction: vi.fn(),
}));

import RadioGroup from './RadioGroup.svelte';
import { setData, resetStore, setFullState, getData } from '$lib/store/data.svelte';
import { sendAction } from '$lib/transport/dispatcher';

beforeEach(() => {
	resetStore('test');
	vi.clearAllMocks();
});

const fruitOptions = [
	{ value: 'a', label: 'Apple' },
	{ value: 'b', label: 'Banana' },
];

test('renders group with legend from props.label as Field.Label', async () => {
	const screen = await render(RadioGroup, {
		props: {
			props: { label: 'Pick one', options: fruitOptions },
			surface: 'test',
		},
	});

	await expect.element(screen.getByText('Pick one')).toBeVisible();
});

test('renders each option with a radio input', async () => {
	const screen = await render(RadioGroup, {
		props: {
			props: { label: 'Pick one', options: fruitOptions },
			surface: 'test',
		},
	});

	// Bits-ui RadioGroup.Item renders with role="radio"
	const radios = screen.baseElement.querySelectorAll('[role="radio"]');
	expect(radios.length).toBe(2);
});

test('selects option when bind value matches', async () => {
	setData('test', '/choice', 'b');

	const screen = await render(RadioGroup, {
		props: {
			props: { label: 'Pick one', options: fruitOptions },
			bind: '/choice',
			surface: 'test',
		},
	});

	const radios = screen.baseElement.querySelectorAll('[role="radio"]');
	// Second radio (value=b) should be checked
	expect(radios[1]?.getAttribute('aria-checked')).toBe('true');
});

test('shows Field.Error and data-invalid when /_errors/{bind} is set', async () => {
	setData('test', '/_errors/choice', 'must pick');

	const screen = await render(RadioGroup, {
		props: {
			props: { label: 'Pick one', options: fruitOptions },
			bind: '/choice',
			surface: 'test',
		},
	});

	await expect.element(screen.getByText('must pick')).toBeVisible();
	const wrapper = screen.baseElement.querySelector('[data-slot="field"]');
	expect(wrapper?.hasAttribute('data-invalid')).toBe(true);
});

test('renders per-option description text adjacent to option label', async () => {
	// A4 assumption check — if shadcn RadioGroup doesn't support per-item
	// description, the rendered DOM is a plain-text span adjacent to label.
	const optsWithDesc = [
		{ value: 'a', label: 'Apple', description: 'red fruit' },
		{ value: 'b', label: 'Banana' },
	];

	const screen = await render(RadioGroup, {
		props: {
			props: { label: 'Pick one', options: optsWithDesc },
			surface: 'test',
		},
	});

	await expect.element(screen.getByText('red fruit')).toBeVisible();
});

// -----------------------------------------------------------------------------
// Phase 18 Plan 02 — Framework Gap 2: blur-action dispatch.
//
// RadioGroup's bound value is a string (selected option's value, or "" if
// nothing selected). We wrap in a <div onfocusout> so focus-leave from any
// radio child bubbles to the wrapper and fires handleBlur().
// -----------------------------------------------------------------------------

test('blur dispatch: fires sendAction with string value on focusout when action.type === "blur"', async () => {
	const surface = 'test-blur-rg-' + crypto.randomUUID();
	setFullState(surface, { choice: 'b' });
	const screen = await render(RadioGroup, {
		props: {
			props: { label: 'Pick one', options: fruitOptions },
			bind: '/choice',
			action: { type: 'blur', name: 'validate-radio' },
			surface,
		},
	});

	const wrapper = screen.baseElement.querySelector(
		'[data-slot="field"]'
	) as HTMLElement;
	expect(wrapper).toBeTruthy();
	const rootWrapper = wrapper.parentElement as HTMLElement;
	expect(rootWrapper).toBeTruthy();
	rootWrapper.dispatchEvent(new FocusEvent('focusout', { bubbles: true }));
	await new Promise((r) => setTimeout(r, 20));

	expect(sendAction).toHaveBeenCalled();
	const calls = (sendAction as ReturnType<typeof vi.fn>).mock.calls;
	const blurCall = calls.find((c) => c[0] === 'validate-radio');
	expect(blurCall).toBeTruthy();
	// String payload — the selected option's value.
	expect(blurCall![1]).toEqual({ value: 'b' });
	expect(typeof (blurCall![1] as { value: unknown }).value).toBe('string');
	expect(blurCall![2]).toBeUndefined();
});

test('blur dispatch: emits empty-string when no option is selected', async () => {
	const surface = 'test-blur-rg-empty-' + crypto.randomUUID();
	setFullState(surface, { choice: '' });
	const screen = await render(RadioGroup, {
		props: {
			props: { label: 'Pick one', options: fruitOptions },
			bind: '/choice',
			action: { type: 'blur', name: 'validate-radio' },
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
		(c) => c[0] === 'validate-radio'
	);
	expect(blurCall).toBeTruthy();
	expect(blurCall![1]).toEqual({ value: '' });
});

test('blur dispatch: does NOT fire when no action is configured', async () => {
	const surface = 'test-noaction-rg-' + crypto.randomUUID();
	setFullState(surface, { choice: 'a' });
	const screen = await render(RadioGroup, {
		props: {
			props: { label: 'Pick one', options: fruitOptions },
			bind: '/choice',
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
	const surface = 'test-notblur-rg-' + crypto.randomUUID();
	setFullState(surface, { choice: 'a' });
	const screen = await render(RadioGroup, {
		props: {
			props: { label: 'Pick one', options: fruitOptions },
			bind: '/choice',
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

test('bind value is preserved through focusout (getData reads current string)', async () => {
	const surface = 'test-rg-pres-' + crypto.randomUUID();
	setFullState(surface, { choice: 'a' });
	const screen = await render(RadioGroup, {
		props: {
			props: { label: 'Pick one', options: fruitOptions },
			bind: '/choice',
			action: { type: 'blur', name: 'validate-radio' },
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
	expect(getData(surface, '/choice')).toBe('a');
});
