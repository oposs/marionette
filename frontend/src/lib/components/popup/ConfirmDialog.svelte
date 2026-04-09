<script lang="ts">
	import { sendAction } from '$lib/transport/dispatcher';
	import type { ComponentAction } from '$lib/transport/messages';
	import type { Snippet } from 'svelte';

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

	let title = $derived((props.title as string) ?? '');
	let message = $derived((props.message as string) ?? '');
	let confirmLabel = $derived((props.confirmLabel as string) ?? 'Confirm');
	let cancelLabel = $derived((props.cancelLabel as string) ?? 'Cancel');
	let destructive = $derived((props.destructive as boolean) ?? false);

	function handleConfirm() {
		if (action) {
			sendAction(action.name ?? action.type, {}, action.target);
		}
	}

	function handleCancel() {
		sendAction('close-modal');
	}
</script>

<div class="p-4">
	{#if title}
		<h3 class="mb-4 text-lg font-semibold text-foreground">{title}</h3>
	{/if}
	{#if message}
		<p class="mb-6 text-muted-foreground">{message}</p>
	{/if}
	<div class="flex justify-end gap-3">
		<button
			type="button"
			class="inline-flex items-center justify-center rounded-md text-sm font-medium h-10 px-4 py-2 border border-input bg-background hover:bg-accent hover:text-accent-foreground"
			onclick={handleCancel}
		>
			{cancelLabel}
		</button>
		<button
			type="button"
			class="inline-flex items-center justify-center rounded-md text-sm font-medium h-10 px-4 py-2 {destructive ? 'bg-destructive text-destructive-foreground hover:bg-destructive/90' : 'bg-primary text-primary-foreground hover:bg-primary/90'}"
			onclick={handleConfirm}
		>
			{confirmLabel}
		</button>
	</div>
</div>
