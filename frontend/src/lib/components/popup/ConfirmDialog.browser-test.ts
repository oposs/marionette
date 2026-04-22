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

// NOTE (Phase 17 Plan 17-05 Task 7 corrective pass): ConfirmDialog was
// switched to plain markup in commit c2c005f (Phase 11 CR-02 fix) — the
// component renders <h2>/<p>/<ShadcnButton> inside a bare <div>, NOT
// inside Dialog.Title / Dialog.Description / Dialog.Footer wrappers.
// The wrapping Dialog.Root/Dialog.Content is supplied by ModalSurface
// (or ConfirmDialogTestWrapper) but ConfirmDialog itself does not emit
// data-slot="dialog-title"/"dialog-description"/"dialog-footer" markers.
// These tests query the actual rendered structure accordingly.
//
// The dialog content renders in a Portal so we query through
// screen.baseElement.ownerDocument rather than the scoped screen.

test('renders title and message', async () => {
	const screen = await render(ConfirmDialogTestWrapper, {
		props: {
			props: { title: 'Delete?', message: 'Are you sure?' },
			surface: 'test',
		},
	});

	const doc = screen.baseElement.ownerDocument;
	// Wait for portal content to mount
	await expect
		.element(doc.querySelector('h2.text-lg.font-semibold')! as HTMLElement)
		.toBeVisible();
	expect(doc.querySelector('h2.text-lg.font-semibold')!.textContent).toBe('Delete?');
	expect(doc.querySelector('p.text-sm')!.textContent).toBe('Are you sure?');
});

test('renders confirm and cancel buttons', async () => {
	const screen = await render(ConfirmDialogTestWrapper, {
		props: {
			props: { confirmLabel: 'Delete', cancelLabel: 'Keep' },
			surface: 'test',
		},
	});

	const doc = screen.baseElement.ownerDocument;
	// Portal mounts on first tick — wait for our button to appear
	await expect
		.element(doc.querySelector('[role="dialog"]')! as HTMLElement)
		.toBeVisible();
	const dialog = doc.querySelector('[role="dialog"]')!;
	const buttonLabels = Array.from(dialog.querySelectorAll('button')).map(
		(b) => b.textContent?.trim() ?? '',
	);
	expect(buttonLabels).toContain('Delete');
	expect(buttonLabels).toContain('Keep');
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
	await expect
		.element(doc.querySelector('[role="dialog"]')! as HTMLElement)
		.toBeVisible();
	const dialog = doc.querySelector('[role="dialog"]')!;
	const buttons = dialog.querySelectorAll('button');
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
	await expect
		.element(doc.querySelector('[role="dialog"]')! as HTMLElement)
		.toBeVisible();
	const dialog = doc.querySelector('[role="dialog"]')!;
	const buttons = dialog.querySelectorAll('button');
	const cancelBtn = Array.from(buttons).find((b) => b.textContent?.trim() === 'No');
	cancelBtn!.click();

	expect(sendAction).toHaveBeenCalledWith('close-modal');
});

// Phase 17 Plan 17-05 Task 7 — G-04 corrective pass:
// ConfirmDialog.svelte now reads a configurable `cancel_action` prop
// (falling back to the legacy `cancelAction` form and then 'close-modal').
// Backend's handle_confirm_open wires this to `gallery-demo/confirm-reject`
// so clicking Cancel/Reject triggers the reject handler (which closes the
// modal + enqueues a "Confirm rejected" toast) instead of the old
// frontend-hardcoded 'close-modal' no-toast path.
test('dispatches custom cancel_action on cancel click when provided', async () => {
	const screen = await render(ConfirmDialogTestWrapper, {
		props: {
			props: {
				confirm_label: 'Accept',
				cancel_label: 'Reject',
				cancel_action: 'gallery-demo/confirm-reject',
			},
			surface: 'test',
		},
	});

	const doc = screen.baseElement.ownerDocument;
	await expect
		.element(doc.querySelector('[role="dialog"]')! as HTMLElement)
		.toBeVisible();
	const dialog = doc.querySelector('[role="dialog"]')!;
	const buttons = dialog.querySelectorAll('button');
	const cancelBtn = Array.from(buttons).find((b) => b.textContent?.trim() === 'Reject');
	cancelBtn!.click();

	expect(sendAction).toHaveBeenCalledWith('gallery-demo/confirm-reject');
});
