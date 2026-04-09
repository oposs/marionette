<script lang="ts">
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

<form onsubmit={handleSubmit} class="space-y-4 shrink-0 overflow-y-auto">
	{#if Array.isArray(formErrors) && formErrors.length > 0}
		<div class="rounded-lg bg-destructive/10 p-4">
			{#each formErrors as error}
				<p class="text-sm text-destructive">{error}</p>
			{/each}
		</div>
	{/if}
	{@render children?.()}
</form>
