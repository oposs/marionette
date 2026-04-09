import { render } from 'vitest-browser-svelte';
import { expect, test, vi, beforeEach } from 'vitest';
import Form from './Form.svelte';

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
