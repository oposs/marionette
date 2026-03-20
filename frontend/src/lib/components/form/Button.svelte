<script lang="ts">
	import { Button as FlowbiteButton } from 'flowbite-svelte';
	import type { ButtonProps } from 'flowbite-svelte';
	import { sendAction } from '$lib/transport/dispatcher';
	import type { ComponentAction, PatchOperation } from '$lib/transport/messages';
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

	function handleClick() {
		if (action) {
			const optimisticField = action.optimistic as
				| { patch: PatchOperation[] }
				| undefined;
			sendAction(
				action.name ?? action.type,
				(action.payload as Record<string, unknown>) ?? {},
				action.target,
				optimisticField ? { patch: optimisticField.patch } : undefined
			);
		}
	}
</script>

<FlowbiteButton
	color={(props.color as ButtonProps['color']) ?? 'primary'}
	size={(props.size as ButtonProps['size']) ?? 'md'}
	disabled={props.disabled as boolean}
	outline={props.outline as boolean}
	onclick={handleClick}
>
	{props.label ?? ''}
</FlowbiteButton>
