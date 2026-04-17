import { render } from 'vitest-browser-svelte';
import { expect, test, beforeEach } from 'vitest';
import TextInput from './TextInput.svelte';
import { setFullState, resetStore, setData } from '$lib/store/data.svelte';
import { resetDirty } from '$lib/store/dirty.svelte';

beforeEach(() => {
	resetStore('test');
	resetDirty();
});

test('renders label when provided', async () => {
	const screen = await render(TextInput, {
		props: { props: { label: 'Email' }, surface: 'test' },
	});

	await expect.element(screen.getByText('Email')).toBeVisible();
});

test('renders input element', async () => {
	const screen = await render(TextInput, {
		props: { props: {}, surface: 'test' },
	});

	const input = screen.baseElement.querySelector('input');
	expect(input).toBeTruthy();
});

test('renders error state', async () => {
	setFullState('test', { _errors: { email: 'Email is required' } });

	const screen = await render(TextInput, {
		props: { props: { label: 'Email' }, bind: '/email', surface: 'test' },
	});

	await expect.element(screen.getByText('Email is required')).toBeVisible();
	// Error message should use destructive text
	const errorEl = screen.getByText('Email is required').element();
	expect(errorEl.className).toContain('text-destructive');
});

test('renders placeholder', async () => {
	const screen = await render(TextInput, {
		props: { props: { placeholder: 'Enter email' }, surface: 'test' },
	});

	const input = screen.baseElement.querySelector('input') as HTMLInputElement;
	expect(input.placeholder).toBe('Enter email');
});

// -----------------------------------------------------------------------------
// D-H4a: props.input_type (backend-authoritative, snake_case) — Phase 13 Plan 07
//
// The backend serializes TextInput's input type via `props.input_type`
// (see builders/standard.rs TextInput.input_type field). Prior to Phase 13
// the Svelte component incorrectly read `props.type`, so password fields
// rendered as `<input type="text">`. Pre-deployment posture: no back-compat
// fallback to `props.type` — the only authoritative source is `input_type`.
// -----------------------------------------------------------------------------

test('defaults to type="text" when no input_type set', async () => {
	const screen = await render(TextInput, {
		props: { props: { label: 'Name' }, surface: 'test' },
	});
	const input = screen.baseElement.querySelector('input') as HTMLInputElement;
	expect(input.getAttribute('type')).toBe('text');
});

test('reads props.input_type (backend-authoritative) — password field', async () => {
	const screen = await render(TextInput, {
		props: { props: { label: 'Password', input_type: 'password' }, surface: 'test' },
	});
	const input = screen.baseElement.querySelector('input') as HTMLInputElement;
	expect(input.getAttribute('type')).toBe('password');
});

test('reads props.input_type for email', async () => {
	const screen = await render(TextInput, {
		props: { props: { label: 'Email', input_type: 'email' }, surface: 'test' },
	});
	const input = screen.baseElement.querySelector('input') as HTMLInputElement;
	expect(input.getAttribute('type')).toBe('email');
});

test('ignores legacy props.type (no backward-compat fallback per pre-deployment posture)', async () => {
	// Pre-deployment: there is no deployed base shipping props.type.
	// If a caller mistakenly passes props.type, it is silently ignored and
	// the input falls back to the default 'text'. This documents the
	// no-compat-shim posture and guards against accidental reintroduction.
	const screen = await render(TextInput, {
		props: { props: { label: 'Legacy', type: 'password' }, surface: 'test' },
	});
	const input = screen.baseElement.querySelector('input') as HTMLInputElement;
	expect(input.getAttribute('type')).toBe('text');
});

// -----------------------------------------------------------------------------
// Phase 14 Plan 02 (FORM-01 + D-B1) — shadcn Field.Field anatomy.
//
// Each new `it(...)` block locks one slice of the Shared Leaf Anatomy contract
// from `.planning/phases/14-formscreen-enhancements/14-UI-SPEC.md`:
// - The outer wrapper is a `data-slot="field"` (Field.Field) element.
// - `Field.Label for={id}` matches the `<input id={id}>` exactly.
// - Clicking the label focuses the input (a11y guarantee).
// - `data-invalid` and `aria-invalid` are attribute-presence semantics —
//   the attribute is OMITTED (not `"false"`) when there is no error.
// - `Field.Description` renders when provided AND no error is active.
// - When an error is active, `Field.Description` is hidden and `Field.Error`
//   replaces it (shadcn recipe pattern).
// - `props.full_width` adds `col-span-full` to the Field.Field wrapper
//   (per-field override for FieldSet grid — D-C4).
// - Handler-supplied `props.id` is used verbatim.
// - Falls back to a stable mount-time UUID when `props.id` is omitted, so
//   two renders without id produce different ids, but a rerender of the same
//   component instance keeps the id stable.
// -----------------------------------------------------------------------------

test('wraps input in a Field.Field element (data-slot="field")', async () => {
	const screen = await render(TextInput, {
		props: { props: { label: 'Full name' }, surface: 'test' },
	});
	const wrapper = screen.baseElement.querySelector('[data-slot="field"]');
	expect(wrapper).toBeTruthy();
	// The input must be a descendant of the Field.Field wrapper.
	const input = wrapper?.querySelector('input');
	expect(input).toBeTruthy();
});

test('Field.Label for attribute matches input id', async () => {
	const screen = await render(TextInput, {
		props: { props: { label: 'Full name', id: 'my-name-field' }, surface: 'test' },
	});
	const label = screen.baseElement.querySelector('label');
	const input = screen.baseElement.querySelector('input') as HTMLInputElement;
	expect(label).toBeTruthy();
	expect(input).toBeTruthy();
	expect(label!.getAttribute('for')).toBe(input.id);
	expect(input.id).toBe('my-name-field');
});

test('clicking the label focuses the input', async () => {
	const screen = await render(TextInput, {
		props: { props: { label: 'Full name', id: 'focus-me' }, surface: 'test' },
	});
	const label = screen.baseElement.querySelector('label') as HTMLLabelElement;
	const input = screen.baseElement.querySelector('input') as HTMLInputElement;
	expect(label).toBeTruthy();
	expect(input).toBeTruthy();
	label.click();
	expect(document.activeElement).toBe(input);
});

test('renders Field.Description when provided and no error', async () => {
	const screen = await render(TextInput, {
		props: {
			props: { label: 'Email', description: "We'll never share your email." },
			surface: 'test',
		},
	});
	const desc = screen.baseElement.querySelector('[data-slot="field-description"]');
	expect(desc).toBeTruthy();
	expect(desc!.textContent).toContain("We'll never share your email.");
});

test('hides Field.Description and shows Field.Error when error is active', async () => {
	const surface = 'test-' + crypto.randomUUID();
	setData(surface, '/_errors/email', 'Invalid email.');
	const screen = await render(TextInput, {
		props: {
			props: { label: 'Email', description: 'Optional helper text.' },
			bind: '/email',
			surface,
		},
	});
	const desc = screen.baseElement.querySelector('[data-slot="field-description"]');
	const err = screen.baseElement.querySelector('[data-slot="field-error"]');
	expect(desc).toBeNull();
	expect(err).toBeTruthy();
	expect(err!.textContent).toContain('Invalid email.');
});

test('data-invalid attribute is present on error, OMITTED on no-error', async () => {
	// No error — attribute omitted (Pitfall #4).
	const screen1 = await render(TextInput, {
		props: { props: { label: 'Email' }, bind: '/email', surface: 'test-no-err' },
	});
	const wrapper1 = screen1.baseElement.querySelector('[data-slot="field"]');
	expect(wrapper1).toBeTruthy();
	// Attribute presence semantics: null means absent.
	expect(wrapper1!.getAttribute('data-invalid')).toBeNull();

	// Error — attribute present.
	const surface = 'test-err-' + crypto.randomUUID();
	setData(surface, '/_errors/email', 'Required.');
	const screen2 = await render(TextInput, {
		props: { props: { label: 'Email' }, bind: '/email', surface },
	});
	const wrapper2 = screen2.baseElement.querySelector('[data-slot="field"]');
	expect(wrapper2).toBeTruthy();
	// Attribute present — shadcn CSS keys on presence, not value.
	expect(wrapper2!.getAttribute('data-invalid')).not.toBeNull();
});

test('aria-invalid on input is present on error, OMITTED on no-error', async () => {
	const screen1 = await render(TextInput, {
		props: { props: { label: 'Email' }, bind: '/email', surface: 'test-no-aria' },
	});
	const input1 = screen1.baseElement.querySelector('input') as HTMLInputElement;
	expect(input1.getAttribute('aria-invalid')).toBeNull();

	const surface = 'test-aria-' + crypto.randomUUID();
	setData(surface, '/_errors/email', 'Required.');
	const screen2 = await render(TextInput, {
		props: { props: { label: 'Email' }, bind: '/email', surface },
	});
	const input2 = screen2.baseElement.querySelector('input') as HTMLInputElement;
	expect(input2.getAttribute('aria-invalid')).not.toBeNull();
});

test('full_width: true adds col-span-full class to Field.Field wrapper', async () => {
	const screen = await render(TextInput, {
		props: { props: { label: 'Bio', full_width: true }, surface: 'test' },
	});
	const wrapper = screen.baseElement.querySelector('[data-slot="field"]') as HTMLElement;
	expect(wrapper).toBeTruthy();
	expect(wrapper.className).toContain('col-span-full');
});

test('full_width omitted: wrapper does NOT have col-span-full', async () => {
	const screen = await render(TextInput, {
		props: { props: { label: 'Name' }, surface: 'test' },
	});
	const wrapper = screen.baseElement.querySelector('[data-slot="field"]') as HTMLElement;
	expect(wrapper).toBeTruthy();
	expect(wrapper.className).not.toContain('col-span-full');
});

test('handler-supplied id is used verbatim', async () => {
	const screen = await render(TextInput, {
		props: { props: { label: 'Name', id: 'custom-handler-id' }, surface: 'test' },
	});
	const input = screen.baseElement.querySelector('input') as HTMLInputElement;
	expect(input.id).toBe('custom-handler-id');
});

test('falls back to a stable id when props.id omitted (two renders produce different ids)', async () => {
	const screen1 = await render(TextInput, {
		props: { props: { label: 'A' }, surface: 'test-id-a' },
	});
	const screen2 = await render(TextInput, {
		props: { props: { label: 'B' }, surface: 'test-id-b' },
	});
	const id1 = (screen1.baseElement.querySelector('input') as HTMLInputElement).id;
	const id2 = (screen2.baseElement.querySelector('input') as HTMLInputElement).id;
	expect(id1).toBeTruthy();
	expect(id2).toBeTruthy();
	expect(id1).not.toBe(id2);
});

test('id fallback is stable across rerenders of the same instance', async () => {
	const screen = await render(TextInput, {
		props: { props: { label: 'Stable' }, surface: 'test-stable' },
	});
	const initialId = (screen.baseElement.querySelector('input') as HTMLInputElement).id;
	expect(initialId).toBeTruthy();
	// Rerender with the same props — id must stay the same.
	await screen.rerender({ props: { label: 'Stable' }, surface: 'test-stable' });
	const rerenderedId = (screen.baseElement.querySelector('input') as HTMLInputElement).id;
	expect(rerenderedId).toBe(initialId);
});
