<script lang="ts">
	import { sendAction } from '$lib/transport/dispatcher';
	import { getAllData } from '$lib/store/data.svelte';
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
			// Include all surface data as payload so the backend receives form values
			const surfaceData = getAllData(surface) ?? {};
			const payload = {
				...(action.payload as Record<string, unknown> ?? {}),
				...surfaceData
			};
			sendAction(
				action.name ?? action.type,
				payload,
				action.target,
				optimisticField ? { patch: optimisticField.patch } : undefined
			);
		}
	}

	let colorClass = $derived(
		(props.color as string) === 'red'
			? 'bg-destructive text-destructive-foreground hover:bg-destructive/90'
			: (props.outline as boolean)
				? 'border border-input bg-background hover:bg-accent hover:text-accent-foreground'
				: 'bg-primary text-primary-foreground hover:bg-primary/90'
	);
</script>

<button
	type="button"
	class="inline-flex items-center justify-center rounded-md text-sm font-medium h-10 px-4 py-2 w-full md:w-auto disabled:opacity-50 disabled:pointer-events-none {colorClass}"
	disabled={props.disabled as boolean}
	onclick={handleClick}
>
	{props.label ?? ''}
</button>
