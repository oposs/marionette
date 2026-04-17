import { render } from 'vitest-browser-svelte';
import { expect, test, beforeEach } from 'vitest';
import Checkbox from './Checkbox.svelte';
import { resetStore, setData, setFullState, getData } from '$lib/store/data.svelte';

beforeEach(() => {
	resetStore('test');
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
