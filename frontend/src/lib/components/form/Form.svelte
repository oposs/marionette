<script lang="ts">
	import * as Field from '$lib/components/ui/field';
	import { getData } from '$lib/store/data.svelte';
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

	let formErrors = $derived(
		bind ? ((getData(surface, '/_errors' + bind) as string[]) ?? []) : []
	);

	function handleSubmit(e: SubmitEvent) {
		e.preventDefault();
		if (action) {
			sendAction(action.name ?? 'submit', {}, action.target);
		}
	}
</script>

<form onsubmit={handleSubmit} class="shrink-0 overflow-y-auto">
	{#if Array.isArray(formErrors) && formErrors.length > 0}
		<div
			class="bg-destructive/10 border border-destructive/50 text-destructive rounded-md p-4 mb-4"
		>
			{#each formErrors as error}
				<p class="text-sm">{error}</p>
			{/each}
		</div>
	{/if}
	<Field.Group class="space-y-6">
		{@render children?.()}
	</Field.Group>
</form>
