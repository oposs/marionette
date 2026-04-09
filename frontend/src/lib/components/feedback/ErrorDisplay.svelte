<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { ComponentAction } from '$lib/transport/messages';
	import { getData } from '$lib/store/data.svelte';
	import AlertCircle from '@lucide/svelte/icons/alert-circle';

	let {
		props = {},
		bind,
		action,
		surface,
		children
	}: {
		props: Record<string, unknown>;
		bind?: string;
		action?: ComponentAction;
		surface: string;
		children?: Snippet;
	} = $props();

	interface ErrorEntry {
		path?: string;
		message: string;
	}

	let errors = $derived(
		bind ? (getData(surface, bind) as ErrorEntry[] | undefined) ?? [] : []
	);
</script>

{#if errors.length > 0}
	{#each errors as error}
		<div class="mb-2 flex items-center gap-2 rounded-md border border-destructive/20 bg-destructive/10 p-4 text-destructive">
			<AlertCircle class="size-5 shrink-0" />
			<span class="text-sm">{error.message}</span>
			{#if error.path}
				<span class="text-xs opacity-60 ml-1">{error.path}</span>
			{/if}
		</div>
	{/each}
{/if}
