/**
 * Toast store — module-level singleton for toast notifications.
 *
 * This must be a standalone `.svelte.ts` module (not an instance export on
 * ToastSurface.svelte) so that dispatcher code and any consumer can reach it
 * via a normal `import { addToast } from '$lib/store/toasts.svelte'` without
 * needing a `bind:this` reference to the mounted component.
 */

interface ToastItem {
	id: string;
	severity: string;
	message: string;
	duration: number;
}

let toasts = $state<ToastItem[]>([]);

/** Returns the reactive toasts array (tracked by $state). */
export function getToasts(): ToastItem[] {
	return toasts;
}

/**
 * Add a toast from an EventMessage-shaped payload.
 *
 * Accepts the same `{ name, hint? }` shape used by server-sent events so
 * callers in the dispatcher can forward the event directly.
 */
export function addToast(event: { name: string; hint?: Record<string, unknown> }): void {
	const id = crypto.randomUUID();
	const severity = (event.hint?.severity as string) ?? 'info';
	const message = (event.hint?.message as string) ?? event.name;
	const duration = (event.hint?.duration as number) ?? 5000;

	toasts.push({ id, severity, message, duration });

	setTimeout(() => removeToast(id), duration);
}

/** Remove a toast by id. */
export function removeToast(id: string): void {
	toasts = toasts.filter((t) => t.id !== id);
}
