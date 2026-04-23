import { render } from 'vitest-browser-svelte';
import { expect, test, vi, beforeEach } from 'vitest';

// Phase 18 Plan 02 — blur-action dispatch tests need a sendAction mock.
// Mirrors the pattern used by SelectInput.browser-test.ts and Button/Form.
// MUST be declared before the Checkbox import because vi.mock is hoisted
// but the import resolution order still matters for the mock factory capture.
vi.mock('$lib/transport/dispatcher', () => ({
	sendAction: vi.fn(),
}));

import Checkbox from './Checkbox.svelte';
import { resetStore, setData, setFullState, getData } from '$lib/store/data.svelte';
import { sendAction } from '$lib/transport/dispatcher';

beforeEach(() => {
	resetStore('test');
	vi.clearAllMocks();
});

// -----------------------------------------------------------------------------
// Baseline tests (pre-Plan 14-04) — preserved verbatim.
// -----------------------------------------------------------------------------

test('renders with label', async () => {
	const screen = await render(Checkbox, {
		props: { props: { label: 'Active' }, surface: 'test' },
	});

	await expect.element(screen.getByText('Active')).toBeVisible();
});

test('renders checkbox element', async () => {
	const screen = await render(Checkbox, {
		props: { props: { label: 'Active' }, surface: 'test' },
	});

	// shadcn checkbox renders a button with role="checkbox"
	const checkbox = screen.getByRole('checkbox');
	await expect.element(checkbox).toBeVisible();
});

test('renders disabled state', async () => {
	const screen = await render(Checkbox, {
		props: { props: { label: 'Active', disabled: true }, surface: 'test' },
	});

	const checkbox = screen.getByRole('checkbox');
	await expect.element(checkbox).toBeDisabled();
});

// -----------------------------------------------------------------------------
// Phase 14 Plan 04 (FORM-01 + D-B1) — shadcn Field.Field anatomy with
// HORIZONTAL orientation (14-UI-SPEC.md §Component Visual Contracts — Checkbox).
//
// Checkbox differs from TextInput / SelectInput in one structural way:
// the Field.Field wrapper uses orientation="horizontal" so the checkbox
// control and its label sit inline on the same row. Description / Error
// flow beneath via the Field recipe's horizontal layout CSS.
//
// Each new test locks one slice of the Shared Leaf Anatomy:
// - Outer wrapper is `data-slot="field"` (Field.Field) with
//   `data-orientation="horizontal"` (set by the Field primitive when
//   its `orientation` prop is "horizontal" — see ui/field/field.svelte).
// - Field.Label `for` attribute matches the checkbox control id.
// - Clicking the label toggles the checkbox via the native `<label for>`
//   association, which calls `setData(surface, bind, newValue)`.
// - `data-invalid` on the wrapper and `aria-invalid` on the control are
//   attribute-presence semantics — OMITTED (not `"false"`) on no-error.
// - `Field.Description` renders when provided AND no error is active.
// - `Field.Description` is suppressed on error; `Field.Error` takes its place.
// - `props.full_width` adds `col-span-full` to the Field.Field wrapper.
// -----------------------------------------------------------------------------

test('wraps control in a Field.Field element (data-slot="field") with horizontal orientation', async () => {
	const screen = await render(Checkbox, {
		props: { props: { label: 'Send marketing emails' }, surface: 'test' },
	});
	const wrapper = screen.baseElement.querySelector('[data-slot="field"]');
	expect(wrapper).toBeTruthy();
	// Horizontal orientation — the Field primitive sets data-orientation from
	// its `orientation` prop (see ui/field/field.svelte line 48).
	expect(wrapper!.getAttribute('data-orientation')).toBe('horizontal');
	// The checkbox button must be a descendant of the Field.Field wrapper.
	const checkbox = wrapper?.querySelector('[role="checkbox"]');
	expect(checkbox).toBeTruthy();
});

test('Field.Label for attribute matches checkbox control id', async () => {
	const screen = await render(Checkbox, {
		props: {
			props: { label: 'Send marketing emails', id: 'opt-in-checkbox' },
			surface: 'test',
		},
	});
	const label = screen.baseElement.querySelector('label');
	const checkbox = screen.baseElement.querySelector(
		'[role="checkbox"]'
	) as HTMLButtonElement | null;
	expect(label).toBeTruthy();
	expect(checkbox).toBeTruthy();
	expect(checkbox!.id).toBe('opt-in-checkbox');
	expect(label!.getAttribute('for')).toBe(checkbox!.id);
});

test('clicking the label toggles the checkbox and writes the new value via bind', async () => {
	const surface = 'test-toggle-' + crypto.randomUUID();
	setFullState(surface, { optedIn: false });
	const screen = await render(Checkbox, {
		props: {
			props: { label: 'Send marketing emails', id: 'opt-in-toggle' },
			bind: '/optedIn',
			surface,
		},
	});
	const label = screen.baseElement.querySelector('label') as HTMLLabelElement;
	expect(label).toBeTruthy();
	// Clicking a <label for={id}> toggles the associated control natively.
	label.click();
	// bits-ui Checkbox fires onCheckedChange synchronously via the native click
	// dispatch; the bind-write must have landed.
	expect(getData(surface, '/optedIn')).toBe(true);
});

test('renders Field.Description when provided and no error', async () => {
	const screen = await render(Checkbox, {
		props: {
			props: {
				label: 'Send marketing emails',
				description: 'We will only send occasional product updates.',
			},
			surface: 'test',
		},
	});
	const desc = screen.baseElement.querySelector('[data-slot="field-description"]');
	expect(desc).toBeTruthy();
	expect(desc!.textContent).toContain('We will only send occasional product updates.');
});

test('hides Field.Description and shows Field.Error when error is active', async () => {
	const surface = 'test-err-' + crypto.randomUUID();
	setData(surface, '/_errors/optedIn', 'Please opt in.');
	const screen = await render(Checkbox, {
		props: {
			props: {
				label: 'Send marketing emails',
				description: 'Optional helper text.',
			},
			bind: '/optedIn',
			surface,
		},
	});
	const desc = screen.baseElement.querySelector('[data-slot="field-description"]');
	const err = screen.baseElement.querySelector('[data-slot="field-error"]');
	expect(desc).toBeNull();
	expect(err).toBeTruthy();
	expect(err!.textContent).toContain('Please opt in.');
});

test('data-invalid attribute is OMITTED on no-error (Pitfall #4)', async () => {
	setFullState('test-no-err', {});
	const screen = await render(Checkbox, {
		props: {
			props: { label: 'Send marketing emails' },
			bind: '/optedIn',
			surface: 'test-no-err',
		},
	});
	const wrapper = screen.baseElement.querySelector('[data-slot="field"]');
	expect(wrapper).toBeTruthy();
	expect(wrapper!.getAttribute('data-invalid')).toBeNull();
});

test('data-invalid attribute is present on error', async () => {
	const surface = 'test-di-err-' + crypto.randomUUID();
	setData(surface, '/_errors/optedIn', 'Required.');
	const screen = await render(Checkbox, {
		props: {
			props: { label: 'Send marketing emails' },
			bind: '/optedIn',
			surface,
		},
	});
	const wrapper = screen.baseElement.querySelector('[data-slot="field"]');
	expect(wrapper).toBeTruthy();
	expect(wrapper!.getAttribute('data-invalid')).not.toBeNull();
});

test('aria-invalid on checkbox is OMITTED on no-error', async () => {
	setFullState('test-no-aria', {});
	const screen = await render(Checkbox, {
		props: {
			props: { label: 'Send marketing emails' },
			bind: '/optedIn',
			surface: 'test-no-aria',
		},
	});
	const checkbox = screen.baseElement.querySelector(
		'[role="checkbox"]'
	) as HTMLButtonElement;
	expect(checkbox).toBeTruthy();
	expect(checkbox.getAttribute('aria-invalid')).toBeNull();
});

test('aria-invalid on checkbox is present on error', async () => {
	const surface = 'test-aria-' + crypto.randomUUID();
	setData(surface, '/_errors/optedIn', 'Required.');
	const screen = await render(Checkbox, {
		props: {
			props: { label: 'Send marketing emails' },
			bind: '/optedIn',
			surface,
		},
	});
	const checkbox = screen.baseElement.querySelector(
		'[role="checkbox"]'
	) as HTMLButtonElement;
	expect(checkbox).toBeTruthy();
	expect(checkbox.getAttribute('aria-invalid')).not.toBeNull();
});

test('full_width: true adds col-span-full class to Field.Field wrapper', async () => {
	const screen = await render(Checkbox, {
		props: { props: { label: 'Agree to terms', full_width: true }, surface: 'test' },
	});
	const wrapper = screen.baseElement.querySelector('[data-slot="field"]') as HTMLElement;
	expect(wrapper).toBeTruthy();
	expect(wrapper.className).toContain('col-span-full');
});

test('full_width omitted: wrapper does NOT have col-span-full', async () => {
	const screen = await render(Checkbox, {
		props: { props: { label: 'Agree' }, surface: 'test' },
	});
	const wrapper = screen.baseElement.querySelector('[data-slot="field"]') as HTMLElement;
	expect(wrapper).toBeTruthy();
	expect(wrapper.className).not.toContain('col-span-full');
});

test('handler-supplied id is used verbatim on checkbox control', async () => {
	const screen = await render(Checkbox, {
		props: {
			props: { label: 'Agree', id: 'handler-supplied-id' },
			surface: 'test',
		},
	});
	const checkbox = screen.baseElement.querySelector(
		'[role="checkbox"]'
	) as HTMLButtonElement;
	expect(checkbox).toBeTruthy();
	expect(checkbox.id).toBe('handler-supplied-id');
});

// -----------------------------------------------------------------------------
// Phase 18 Plan 02 — Framework Gap 2: blur-action dispatch.
//
// Checkbox does not expose a reliable native onblur on its bits-ui button, so
// we wrap the Field in a `<div onfocusout>` and call handleBlur() when focus
// leaves the group. The contract mirrors TextInput/Textarea: on focusout,
// if action.type === 'blur', dispatch sendAction(name, { value }, target) with
// the current bound boolean value.
// -----------------------------------------------------------------------------

test('blur dispatch: fires sendAction with boolean value on focusout when action.type === "blur"', async () => {
	const surface = 'test-blur-cb-' + crypto.randomUUID();
	setFullState(surface, { optedIn: true });
	const screen = await render(Checkbox, {
		props: {
			props: { label: 'Subscribe' },
			bind: '/optedIn',
			action: { type: 'blur', name: 'validate-checkbox' },
			surface,
		},
	});

	// Fire focusout on the Field root (the outer div wrapper Task 2 adds).
	const wrapper = screen.baseElement.querySelector(
		'[data-slot="field"]'
	) as HTMLElement;
	expect(wrapper).toBeTruthy();
	// The extra wrapper div is the parent of the Field.Field element.
	const rootWrapper = wrapper.parentElement as HTMLElement;
	expect(rootWrapper).toBeTruthy();
	rootWrapper.dispatchEvent(new FocusEvent('focusout', { bubbles: true }));
	await new Promise((r) => setTimeout(r, 20));

	expect(sendAction).toHaveBeenCalled();
	const calls = (sendAction as ReturnType<typeof vi.fn>).mock.calls;
	const blurCall = calls.find((c) => c[0] === 'validate-checkbox');
	expect(blurCall).toBeTruthy();
	// Payload value must be a literal boolean, NOT the string "true".
	expect(blurCall![1]).toEqual({ value: true });
	expect(typeof (blurCall![1] as { value: unknown }).value).toBe('boolean');
	expect(blurCall![2]).toBeUndefined();
});

test('blur dispatch: emits false when checkbox is unchecked', async () => {
	const surface = 'test-blur-cb-false-' + crypto.randomUUID();
	setFullState(surface, { optedIn: false });
	const screen = await render(Checkbox, {
		props: {
			props: { label: 'Subscribe' },
			bind: '/optedIn',
			action: { type: 'blur', name: 'validate-checkbox' },
			surface,
		},
	});
	const wrapper = screen.baseElement.querySelector(
		'[data-slot="field"]'
	) as HTMLElement;
	const rootWrapper = wrapper.parentElement as HTMLElement;
	rootWrapper.dispatchEvent(new FocusEvent('focusout', { bubbles: true }));
	await new Promise((r) => setTimeout(r, 20));

	const blurCall = (sendAction as ReturnType<typeof vi.fn>).mock.calls.find(
		(c) => c[0] === 'validate-checkbox'
	);
	expect(blurCall).toBeTruthy();
	expect(blurCall![1]).toEqual({ value: false });
});

test('blur dispatch: does NOT fire when no action is configured', async () => {
	const surface = 'test-noaction-cb-' + crypto.randomUUID();
	setFullState(surface, { optedIn: true });
	const screen = await render(Checkbox, {
		props: {
			props: { label: 'Subscribe' },
			bind: '/optedIn',
			surface,
		},
	});
	const wrapper = screen.baseElement.querySelector(
		'[data-slot="field"]'
	) as HTMLElement;
	const rootWrapper = wrapper.parentElement as HTMLElement;
	rootWrapper.dispatchEvent(new FocusEvent('focusout', { bubbles: true }));
	await new Promise((r) => setTimeout(r, 20));

	expect(sendAction).not.toHaveBeenCalled();
});

test('blur dispatch: does NOT fire when action.type !== "blur"', async () => {
	const surface = 'test-notblur-cb-' + crypto.randomUUID();
	setFullState(surface, { optedIn: true });
	const screen = await render(Checkbox, {
		props: {
			props: { label: 'Subscribe' },
			bind: '/optedIn',
			action: { type: 'change', name: 'other-action' },
			surface,
		},
	});
	const wrapper = screen.baseElement.querySelector(
		'[data-slot="field"]'
	) as HTMLElement;
	const rootWrapper = wrapper.parentElement as HTMLElement;
	rootWrapper.dispatchEvent(new FocusEvent('focusout', { bubbles: true }));
	await new Promise((r) => setTimeout(r, 20));

	// Blur must not dispatch a non-blur action.
	expect(sendAction).not.toHaveBeenCalledWith(
		'other-action',
		expect.anything(),
		expect.anything()
	);
});

test('bind value is preserved through focusout (getData reads current boolean)', async () => {
	// Sanity regression: focusout must not clobber or coerce the bound value.
	const surface = 'test-cb-pres-' + crypto.randomUUID();
	setFullState(surface, { optedIn: true });
	const screen = await render(Checkbox, {
		props: {
			props: { label: 'Subscribe' },
			bind: '/optedIn',
			action: { type: 'blur', name: 'validate-checkbox' },
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
	expect(getData(surface, '/optedIn')).toBe(true);
});
