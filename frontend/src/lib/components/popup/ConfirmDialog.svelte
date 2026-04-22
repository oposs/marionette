<script lang="ts">
	import { sendAction } from '$lib/transport/dispatcher';
	import type { ComponentAction } from '$lib/transport/messages';
	import type { Snippet } from 'svelte';
	import { Button as ShadcnButton } from '$lib/components/ui/button';

	let {
		props = {},
		bind,
		action,
		surface,
		children,
	}: {
		props: Record<string, unknown>;
		bind?: string;
		action?: ComponentAction;
		surface: string;
		children?: Snippet;
	} = $props();

	// G-04 fix — Phase 17 Plan 17-05 Task 7 (corrective pass):
	// Backend ConfirmDialog struct now carries `confirm_label`, `cancel_label`,
	// `cancel_action`, and `destructive` (snake_case via ComponentBuilder derive,
	// matching DataTable's `page_size` precedent).
	//
	// For backward compatibility with the pre-existing browser tests that use
	// camelCase (`confirmLabel`, `cancelLabel`), we read both shapes and
	// prefer the snake_case form when both are present.
	let title = $derived((props.title as string) ?? '');
	let message = $derived((props.message as string) ?? '');
	let confirmLabel = $derived(
		(props.confirm_label as string) ?? (props.confirmLabel as string) ?? 'Confirm',
	);
	let cancelLabel = $derived(
		(props.cancel_label as string) ?? (props.cancelLabel as string) ?? 'Cancel',
	);
	let cancelAction = $derived(
		(props.cancel_action as string) ?? (props.cancelAction as string) ?? 'close-modal',
	);
	let destructive = $derived((props.destructive as boolean) ?? false);

	function handleConfirm() {
		if (action) {
			sendAction(action.name ?? action.type, {}, action.target);
		}
	}

	function handleCancel() {
		sendAction(cancelAction);
	}
</script>

<!--
	ConfirmDialog is rendered as a child of ModalSurface's Dialog.Content via
	NodeRenderer. Because Svelte context set by Dialog.Root does not propagate
	cleanly through NodeRenderer, using Dialog.Title / Dialog.Description here
	would leave `aria-labelledby` / `aria-describedby` wiring broken and the
	Dialog.Footer layout incorrect. Render plain markup that assumes the
	wrapping Dialog.Content is supplied by ModalSurface.
-->
<div>
	<div class="mb-4">
		{#if title}
			<h2 class="text-lg font-semibold">{title}</h2>
		{/if}
		{#if message}
			<p class="text-sm text-muted-foreground mt-1">{message}</p>
		{/if}
	</div>
	<div class="flex justify-end gap-2 mt-6">
		<ShadcnButton variant="outline" onclick={handleCancel}>
			{cancelLabel}
		</ShadcnButton>
		<ShadcnButton variant={destructive ? 'destructive' : 'default'} onclick={handleConfirm}>
			{confirmLabel}
		</ShadcnButton>
	</div>
</div>
