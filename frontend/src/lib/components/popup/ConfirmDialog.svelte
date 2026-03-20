<script lang="ts">
	import { Modal, Button } from 'flowbite-svelte';
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
		<h3 class="mb-4 text-lg font-semibold text-gray-900 dark:text-white">{title}</h3>
	{/if}
	{#if message}
		<p class="mb-6 text-gray-500 dark:text-gray-400">{message}</p>
	{/if}
	<div class="flex justify-end gap-3">
		<Button color="alternative" onclick={handleCancel}>
			{cancelLabel}
		</Button>
		<Button
			color={destructive ? 'red' : 'primary'}
			onclick={handleConfirm}
		>
			{confirmLabel}
		</Button>
	</div>
</div>
