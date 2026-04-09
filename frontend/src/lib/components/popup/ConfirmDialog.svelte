<script lang="ts">
	import { sendAction } from '$lib/transport/dispatcher';
	import type { ComponentAction } from '$lib/transport/messages';
	import type { Snippet } from 'svelte';
	import * as Dialog from '$lib/components/ui/dialog';
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

<div>
	<Dialog.Header>
		{#if title}
			<Dialog.Title>{title}</Dialog.Title>
		{/if}
		{#if message}
			<Dialog.Description>{message}</Dialog.Description>
		{/if}
	</Dialog.Header>
	<Dialog.Footer>
		<ShadcnButton variant="outline" onclick={handleCancel}>
			{cancelLabel}
		</ShadcnButton>
		<ShadcnButton variant={destructive ? 'destructive' : 'default'} onclick={handleConfirm}>
			{confirmLabel}
		</ShadcnButton>
	</Dialog.Footer>
</div>
