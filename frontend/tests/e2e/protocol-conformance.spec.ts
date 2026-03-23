import { test, expect } from '@playwright/test';
import { captureWebSocketFrames } from '../helpers/ws-capture';
import { createValidator } from '../helpers/schema-validator';

test('hello message conforms to schema', async ({ page }) => {
	const frames = captureWebSocketFrames(page);
	await page.goto('/');

	await expect.poll(() => frames.filter((f) => f.data.type === 'hello').length, {
		timeout: 10000,
	}).toBeGreaterThan(0);

	const helloFrame = frames.find((f) => f.data.type === 'hello')!;
	const validator = createValidator();
	const valid = validator.validateHello(helloFrame.data);
	expect(valid, `Schema errors: ${validator.getErrors()}`).toBe(true);
});

test('render message conforms to schema', async ({ page }) => {
	const frames = captureWebSocketFrames(page);
	await page.goto('/');

	await expect.poll(
		() => frames.filter((f) => f.direction === 'received' && f.data.type === 'render').length,
		{ timeout: 10000 },
	).toBeGreaterThan(0);

	const renderFrame = frames.find(
		(f) => f.direction === 'received' && f.data.type === 'render',
	)!;
	const validator = createValidator();
	const valid = validator.validateRender(renderFrame.data);
	expect(valid, `Schema errors: ${validator.getErrors()}`).toBe(true);
});

test('action message conforms to schema', async ({ page }) => {
	const frames = captureWebSocketFrames(page);
	await page.goto('/');

	await expect.poll(
		() => frames.filter((f) => f.direction === 'sent' && f.data.type === 'action').length,
		{ timeout: 10000 },
	).toBeGreaterThan(0);

	const actionFrame = frames.find(
		(f) => f.direction === 'sent' && f.data.type === 'action',
	)!;
	const validator = createValidator();
	const valid = validator.validateAction(actionFrame.data);
	expect(valid, `Schema errors: ${validator.getErrors()}`).toBe(true);
});

test('patch message conforms to schema', async ({ page }) => {
	const frames = captureWebSocketFrames(page);
	await page.goto('/');

	// Wait for heading then click the button
	await expect(page.getByText('Welcome to Marionette')).toBeVisible({ timeout: 10000 });
	await page.getByText('Click Me').click();

	await expect.poll(
		() => frames.filter((f) => f.direction === 'received' && f.data.type === 'patch').length,
		{ timeout: 10000 },
	).toBeGreaterThan(0);

	const patchFrame = frames.find(
		(f) => f.direction === 'received' && f.data.type === 'patch',
	)!;
	const validator = createValidator();
	const valid = validator.validatePatch(patchFrame.data);
	expect(valid, `Schema errors: ${validator.getErrors()}`).toBe(true);
});
