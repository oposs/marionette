/**
 * RED harness for Textarea SDUI leaf (scaffolded by Plan 14-01).
 *
 * Textarea.svelte does not exist yet — Wave 2 Plan 14-05 creates it. These
 * tests intentionally fail with an import-resolve error until then. Do NOT
 * add guards (`it.skip.if`, `test.fails`, etc.) — downstream waves need a
 * genuine RED baseline to flip GREEN.
 *
 * Assertions follow the D-B1 internal Field.Field wrap contract:
 *   <Field.Field data-invalid>
 *     <Field.Label for={id}>{label}</Field.Label>
 *     <Textarea id={id} aria-invalid rows=.. placeholder=.. />
 *     <Field.Description>{props.description}</Field.Description>
 *     <Field.Error>{err}</Field.Error>
 *   </Field.Field>
 * Plus `col-span-full` when props.full_width === true (D-C4).
 */
import { render } from 'vitest-browser-svelte';
import { expect, test, beforeEach } from 'vitest';
import Textarea from './Textarea.svelte';
import { setData, resetStore } from '$lib/store/data.svelte';

beforeEach(() => {
	resetStore('test');
});

test('renders textarea with placeholder', async () => {
	const screen = await render(Textarea, {
		props: { props: { placeholder: 'hello' }, surface: 'test' },
	});

	const textbox = screen.getByRole('textbox');
	await expect.element(textbox).toBeInTheDocument();
	const el = textbox.element() as HTMLTextAreaElement;
	expect(el.placeholder).toBe('hello');
});

test('renders description when no error', async () => {
	const screen = await render(Textarea, {
		props: { props: { description: 'help me' }, surface: 'test' },
	});

	await expect.element(screen.getByText('help me')).toBeVisible();
});

test('renders Field.Error and data-invalid when /_errors/{bind} is set', async () => {
	setData('test', '/_errors/foo', 'bad');

	const screen = await render(Textarea, {
		props: { props: { label: 'Foo' }, bind: '/foo', surface: 'test' },
	});

	await expect.element(screen.getByText('bad')).toBeVisible();
	// The Field.Field wrapper carries data-invalid (attribute presence, per
	// shadcn pattern — Pitfall #4 warns us not to serialize `false`).
	const wrapper = screen.baseElement.querySelector('[data-slot="field"]');
	expect(wrapper?.hasAttribute('data-invalid')).toBe(true);
});

test('sets aria-invalid on the textarea when error', async () => {
	setData('test', '/_errors/foo', 'bad');

	const screen = await render(Textarea, {
		props: { props: { label: 'Foo' }, bind: '/foo', surface: 'test' },
	});

	const textbox = screen.getByRole('textbox').element() as HTMLTextAreaElement;
	expect(textbox.getAttribute('aria-invalid')).toBeTruthy();
});

test('adds col-span-full class to wrapper when props.full_width=true', async () => {
	const screen = await render(Textarea, {
		props: { props: { full_width: true }, surface: 'test' },
	});

	const wrapper = screen.baseElement.querySelector('[data-slot="field"]');
	expect(wrapper?.className).toContain('col-span-full');
});

test('rows prop is forwarded to native textarea', async () => {
	const screen = await render(Textarea, {
		props: { props: { rows: 6 }, surface: 'test' },
	});

	const textbox = screen.getByRole('textbox').element() as HTMLTextAreaElement;
	expect(textbox.getAttribute('rows')).toBe('6');
});
