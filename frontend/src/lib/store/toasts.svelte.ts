/**
 * Toast store — thin wrapper over svelte-sonner.
 *
 * The server dispatches `type: "event"` with `name: "toast"` and a hint
 * payload; the client owns chrome (stacking, fade, position, countdown)
 * via svelte-sonner while the protocol owns content
 * (message/severity/duration/action/component).
 *
 * Supported hint shapes:
 *   - { message, severity?, duration? }
 *   - { message, severity?, duration?, action: { label, action: { name, payload? } } }
 *   - { root, nodes, duration? }  // embedded SDUI tree via toast.custom()
 *
 * See CONCEPT.md §"Where the Client Is Smart" for the protocol-vs-client
 * boundary this implements.
 */
import { toast } from 'svelte-sonner';
import { sendAction } from '$lib/transport/dispatcher';
import ToastContent from '$lib/components/popup/ToastContent.svelte';
import type { ComponentNode } from '$lib/transport/messages';

type Severity = 'success' | 'error' | 'warning' | 'info';

interface ToastActionHint {
	label: string;
	action: { name: string; payload?: Record<string, unknown> };
}

interface MessageHint {
	message: string;
	severity?: Severity;
	duration?: number;
	action?: ToastActionHint;
}

interface ComponentHint {
	root: string;
	nodes: Record<string, ComponentNode>;
	duration?: number;
}

export type ToastHint = MessageHint | ComponentHint;

function isComponentHint(hint: Record<string, unknown>): boolean {
	return typeof hint.root === 'string' && typeof hint.nodes === 'object' && hint.nodes !== null;
}

const DEFAULT_DURATION = 5000;

/**
 * Show a toast from a server-sent event hint.
 *
 * Idempotent relative to the hint — calling twice with the same hint
 * displays two toasts (sonner stacks them). Silently returns on a
 * malformed or missing hint.
 */
export function showToast(hint: Record<string, unknown> | undefined | null): void {
	if (!hint || typeof hint !== 'object') return;

	// Embedded-component path: render an SDUI tree inside sonner chrome.
	if (isComponentHint(hint)) {
		const { root, nodes, duration } = hint as unknown as ComponentHint;
		// Bypass toast.custom's componentProps generic — ToastContent's
		// props come from a runtime hint, not a static binding. Sonner
		// receives the constructor + componentProps at runtime.
		(toast.custom as (c: unknown, data: unknown) => string | number)(ToastContent, {
			componentProps: { root, nodes },
			duration: duration ?? DEFAULT_DURATION,
		});
		return;
	}

	// Message path (plain text, optional severity + action).
	const msg = hint as unknown as MessageHint;
	if (typeof msg.message !== 'string') return;

	const severity: Severity = msg.severity ?? 'info';
	const opts: {
		duration: number;
		action?: { label: string; onClick: () => void };
	} = {
		duration: msg.duration ?? DEFAULT_DURATION,
	};

	if (msg.action && typeof msg.action.label === 'string' && msg.action.action) {
		const a = msg.action;
		opts.action = {
			label: a.label,
			onClick: () => {
				sendAction(a.action.name, a.action.payload);
			},
		};
	}

	// Dispatch to the right sonner method based on severity.
	switch (severity) {
		case 'success':
			toast.success(msg.message, opts);
			break;
		case 'error':
			toast.error(msg.message, opts);
			break;
		case 'warning':
			toast.warning(msg.message, opts);
			break;
		case 'info':
		default:
			toast.info(msg.message, opts);
			break;
	}
}
