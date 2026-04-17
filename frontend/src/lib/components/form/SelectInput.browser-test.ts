import { render, cleanup } from 'vitest-browser-svelte';
import { expect, test, vi, beforeEach } from 'vitest';

// Mock sendAction (mirrors Button/Form browser-test pattern) so the Phase 12
// change-action dispatch assertion can verify the argument shape without
// opening a real WebSocket. MUST be declared before the SelectInput import
// because vi.mock is hoisted but the import resolution order still matters
// for the mock factory capture.
vi.mock('$lib/transport/dispatcher', () => ({
	sendAction: vi.fn(),
}));

import SelectInput from './SelectInput.svelte';
import { setFullState, setData, resetStore } from '$lib/store/data.svelte';
import { resetDirty } from '$lib/store/dirty.svelte';
import { sendAction } from '$lib/transport/dispatcher';

beforeEach(() => {
	resetStore('test');
	resetDirty();
	vi.clearAllMocks();
});

test('renders label when provided', async () => {
	const screen = await render(SelectInput, {
		props: {
			props: { label: 'Country', options: [] },
			surface: 'test',
		},
	});

	await expect.element(screen.getByText('Country')).toBeVisible();
});

test('renders trigger with placeholder', async () => {
	const screen = await render(SelectInput, {
		props: {
			props: { placeholder: 'Pick one', options: [] },
			surface: 'test',
		},
	});

	await expect.element(screen.getByText('Pick one')).toBeVisible();
});

test('renders select trigger', async () => {
	const screen = await render(SelectInput, {
		props: {
			props: { options: [{ value: 'a', label: 'Alpha' }] },
			surface: 'test',
		},
	});

	// shadcn Select renders a button as trigger
	const trigger = screen.baseElement.querySelector('[data-slot="select-trigger"]');
	expect(trigger).toBeTruthy();
});

// -----------------------------------------------------------------------------
// Phase 14 Plan 03 (FORM-01 + D-B1) — shadcn Field.Field anatomy for Select.
//
// Each new test block locks one slice of the Shared Leaf Anatomy contract from
// `.planning/phases/14-formscreen-enhancements/14-UI-SPEC.md` applied to the
// Select primitive:
// - The outer wrapper is a `data-slot="field"` (Field.Field) element.
// - `Field.Label for={id}` matches the `<Select.Trigger id={id}>` exactly.
// - `data-invalid` (wrapper) and `aria-invalid` (trigger) are attribute-
//   presence semantics — OMITTED (not `"false"`) when no error.
// - `Field.Description` renders when provided AND no error is active.
// - When an error is active, `Field.Description` is hidden and `Field.Error`
//   replaces it.
// - `props.full_width` adds `col-span-full` to the Field.Field wrapper.
// - Handler-supplied `props.id` is used verbatim on the trigger and the label.
// - Phase 12 change-action dispatch (country-select node-patch demo) still
//   fires with the merged `{ ...action.payload, ...surfaceData }` payload.
// -----------------------------------------------------------------------------

test('wraps select trigger in a Field.Field element (data-slot="field")', async () => {
	const screen = await render(SelectInput, {
		props: { props: { label: 'Country', options: [] }, surface: 'test' },
	});
	const wrapper = screen.baseElement.querySelector('[data-slot="field"]');
	expect(wrapper).toBeTruthy();
	// The select trigger must be a descendant of the Field.Field wrapper.
	const trigger = wrapper?.querySelector('[data-slot="select-trigger"]');
	expect(trigger).toBeTruthy();
});

test('Field.Label for attribute matches Select.Trigger id', async () => {
	const screen = await render(SelectInput, {
		props: {
			props: { label: 'Country', options: [], id: 'my-country-field' },
			surface: 'test',
		},
	});
	const label = screen.baseElement.querySelector('label');
	const trigger = screen.baseElement.querySelector(
		'[data-slot="select-trigger"]'
	) as HTMLElement;
	expect(label).toBeTruthy();
	expect(trigger).toBeTruthy();
	expect(label!.getAttribute('for')).toBe(trigger.id);
	expect(trigger.id).toBe('my-country-field');
});

test('renders Field.Description when provided and no error', async () => {
	const screen = await render(SelectInput, {
		props: {
			props: {
				label: 'Country',
				options: [],
				description: 'Pick your country of residence.',
			},
			surface: 'test',
		},
	});
	const desc = screen.baseElement.querySelector('[data-slot="field-description"]');
	expect(desc).toBeTruthy();
	expect(desc!.textContent).toContain('Pick your country of residence.');
});

test('hides Field.Description and shows Field.Error when error is active', async () => {
	const surface = 'test-' + crypto.randomUUID();
	setData(surface, '/_errors/country', 'Choose one.');
	const screen = await render(SelectInput, {
		props: {
			props: { label: 'Country', options: [], description: 'Optional helper text.' },
			bind: '/country',
			surface,
		},
	});
	const desc = screen.baseElement.querySelector('[data-slot="field-description"]');
	const err = screen.baseElement.querySelector('[data-slot="field-error"]');
	expect(desc).toBeNull();
	expect(err).toBeTruthy();
	expect(err!.textContent).toContain('Choose one.');
});

test('data-invalid attribute is OMITTED on no-error (Pitfall #4)', async () => {
	// Seed surface first so getData() doesn't trigger getStore auto-create
	// inside $derived (which would be a state_unsafe_mutation).
	setFullState('test-no-err', {});
	const screen = await render(SelectInput, {
		props: {
			props: { label: 'Country', options: [] },
			bind: '/country',
			surface: 'test-no-err',
		},
	});
	const wrapper = screen.baseElement.querySelector('[data-slot="field"]');
	expect(wrapper).toBeTruthy();
	// Attribute presence semantics: null means absent.
	expect(wrapper!.getAttribute('data-invalid')).toBeNull();
});

test('data-invalid attribute is present on error', async () => {
	const surface = 'test-err-' + crypto.randomUUID();
	setData(surface, '/_errors/country', 'Required.');
	const screen = await render(SelectInput, {
		props: { props: { label: 'Country', options: [] }, bind: '/country', surface },
	});
	const wrapper = screen.baseElement.querySelector('[data-slot="field"]');
	expect(wrapper).toBeTruthy();
	// Attribute present — shadcn CSS keys on presence, not value.
	expect(wrapper!.getAttribute('data-invalid')).not.toBeNull();
});

test('aria-invalid on trigger is OMITTED on no-error', async () => {
	setFullState('test-no-aria-sel', {});
	const screen = await render(SelectInput, {
		props: {
			props: { label: 'Country', options: [] },
			bind: '/country',
			surface: 'test-no-aria-sel',
		},
	});
	const trigger = screen.baseElement.querySelector(
		'[data-slot="select-trigger"]'
	) as HTMLElement;
	expect(trigger).toBeTruthy();
	expect(trigger.getAttribute('aria-invalid')).toBeNull();
});

test('aria-invalid on trigger is present on error', async () => {
	const surface = 'test-aria-sel-' + crypto.randomUUID();
	setData(surface, '/_errors/country', 'Required.');
	const screen = await render(SelectInput, {
		props: { props: { label: 'Country', options: [] }, bind: '/country', surface },
	});
	const trigger = screen.baseElement.querySelector(
		'[data-slot="select-trigger"]'
	) as HTMLElement;
	expect(trigger).toBeTruthy();
	expect(trigger.getAttribute('aria-invalid')).not.toBeNull();
});

test('full_width: true adds col-span-full class to Field.Field wrapper', async () => {
	const screen = await render(SelectInput, {
		props: {
			props: { label: 'Country', options: [], full_width: true },
			surface: 'test',
		},
	});
	const wrapper = screen.baseElement.querySelector('[data-slot="field"]') as HTMLElement;
	expect(wrapper).toBeTruthy();
	expect(wrapper.className).toContain('col-span-full');
});

test('full_width omitted: wrapper does NOT have col-span-full', async () => {
	const screen = await render(SelectInput, {
		props: { props: { label: 'Country', options: [] }, surface: 'test' },
	});
	const wrapper = screen.baseElement.querySelector('[data-slot="field"]') as HTMLElement;
	expect(wrapper).toBeTruthy();
	expect(wrapper.className).not.toContain('col-span-full');
});

test('handler-supplied id is used verbatim on trigger', async () => {
	const screen = await render(SelectInput, {
		props: {
			props: { label: 'Country', options: [], id: 'custom-handler-id' },
			surface: 'test',
		},
	});
	const trigger = screen.baseElement.querySelector(
		'[data-slot="select-trigger"]'
	) as HTMLElement;
	expect(trigger.id).toBe('custom-handler-id');
});

test('falls back to a stable id when props.id omitted (two renders produce different ids)', async () => {
	// Two separate component instances must produce independent ids. Clean up
	// the first render so its trigger is removed from the shared DOM root
	// before capturing the second render's id.
	const screen1 = await render(SelectInput, {
		props: { props: { label: 'A', options: [] }, surface: 'test-id-a' },
	});
	const id1 = (
		screen1.baseElement.querySelector('[data-slot="select-trigger"]') as HTMLElement
	).id;
	cleanup();
	const screen2 = await render(SelectInput, {
		props: { props: { label: 'B', options: [] }, surface: 'test-id-b' },
	});
	const id2 = (
		screen2.baseElement.querySelector('[data-slot="select-trigger"]') as HTMLElement
	).id;
	expect(id1).toBeTruthy();
	expect(id2).toBeTruthy();
	expect(id1).not.toBe(id2);
});

test('change-action dispatch fires with merged payload on value change (Phase 12 D-A6)', async () => {
	// Pre-seed the surface with some form state — the Phase 12 contact form
	// merges the full surface data into the change-action payload so the
	// server can re-validate cross-field rules (see 14-03-PLAN.md threat
	// T-14-03-04, accepted). The payload shape MUST stay byte-identical
	// to the pre-rewrite behavior.
	const surface = 'test-change-' + crypto.randomUUID();
	setFullState(surface, {
		contactForm: { name: 'Alice', country: '' },
	});
	const screen = await render(SelectInput, {
		props: {
			props: {
				label: 'Country',
				options: [
					{ value: 'CH', label: 'Switzerland' },
					{ value: 'US', label: 'United States' },
				],
			},
			bind: '/contactForm/country',
			action: { type: 'change', name: 'contact_country_change' },
			surface,
		},
	});

	// Open the dropdown and click an item. vitest-browser-svelte's locator
	// retry-wait makes this robust to Select.Content portal mount timing.
	const trigger = screen.getByRole('combobox', { name: 'Country' });
	await trigger.click();
	const item = screen.getByText('Switzerland');
	await item.click();

	// The dispatched payload must be `{ ...(action.payload ?? {}), ...surfaceData }`.
	// Note: at the moment of dispatch the data store has already been updated
	// with the new value via setData, so the merged surface data reflects it.
	expect(sendAction).toHaveBeenCalledTimes(1);
	const call = (sendAction as ReturnType<typeof vi.fn>).mock.calls[0];
	expect(call[0]).toBe('contact_country_change');
	const payload = call[1] as Record<string, unknown>;
	expect(payload).toMatchObject({
		contactForm: { name: 'Alice', country: 'CH' },
	});
});
