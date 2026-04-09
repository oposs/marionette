import { render } from 'vitest-browser-svelte';
import { expect, test, vi, beforeEach } from 'vitest';
import ConfirmDialogTestWrapper from './ConfirmDialogTestWrapper.svelte';

// Mock sendAction for action dispatch assertions
vi.mock('$lib/transport/dispatcher', () => ({
	sendAction: vi.fn(),
}));

import { sendAction } from '$lib/transport/dispatcher';

beforeEach(() => {
	vi.clearAllMocks();
});

test('renders title and message', async () => {
	const screen = await render(ConfirmDialogTestWrapper, {
		props: {
			props: { title: 'Delete?', message: 'Are you sure?' },
			surface: 'test',
		},
	});

	// Dialog content is rendered in a portal, query from the document
	const doc = screen.baseElement.ownerDocument;
	await expect.element(doc.querySelector('[data-slot="dialog-title"]')! as HTMLElement).toBeVisible();
	expect(doc.querySelector('[data-slot="dialog-title"]')!.textContent).toBe('Delete?');
	expect(doc.querySelector('[data-slot="dialog-description"]')!.textContent).toBe('Are you sure?');
});

test('renders confirm and cancel buttons', async () => {
	const screen = await render(ConfirmDialogTestWrapper, {
		props: {
			props: { confirmLabel: 'Delete', cancelLabel: 'Keep' },
			surface: 'test',
		},
	});

	const doc = screen.baseElement.ownerDocument;
	const footer = doc.querySelector('[data-slot="dialog-footer"]')!;
	expect(footer.textContent).toContain('Delete');
	expect(footer.textContent).toContain('Keep');
});

test('dispatches action on confirm click', async () => {
	const screen = await render(ConfirmDialogTestWrapper, {
		props: {
			props: { confirmLabel: 'Yes', cancelLabel: 'No' },
			action: { type: 'delete', name: 'delete-contact' },
			surface: 'test',
		},
	});

	const doc = screen.baseElement.ownerDocument;
	const footer = doc.querySelector('[data-slot="dialog-footer"]')!;
	// Confirm button is the last button in footer (per component order: cancel first, confirm second)
	const buttons = footer.querySelectorAll('button');
	const confirmBtn = Array.from(buttons).find((b) => b.textContent?.trim() === 'Yes');
	confirmBtn!.click();

	expect(sendAction).toHaveBeenCalledWith('delete-contact', {}, undefined);
});

test('dispatches close-modal on cancel click', async () => {
	const screen = await render(ConfirmDialogTestWrapper, {
		props: {
			props: { confirmLabel: 'Yes', cancelLabel: 'No' },
			surface: 'test',
		},
	});

	const doc = screen.baseElement.ownerDocument;
	const footer = doc.querySelector('[data-slot="dialog-footer"]')!;
	const buttons = footer.querySelectorAll('button');
	const cancelBtn = Array.from(buttons).find((b) => b.textContent?.trim() === 'No');
	cancelBtn!.click();

	expect(sendAction).toHaveBeenCalledWith('close-modal');
});
