<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { ComponentAction } from '$lib/transport/messages';
	import { getData } from '$lib/store/data.svelte';
	import { Alert } from 'flowbite-svelte';
	import ExclamationCircleOutline from 'flowbite-svelte-icons/ExclamationCircleOutline.svelte';

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
		<Alert color="red" class="mb-2">
			{#snippet icon()}
				<ExclamationCircleOutline class="w-5 h-5" />
			{/snippet}
			{error.message}
			{#if error.path}
				<span class="text-xs text-red-400 ml-1">{error.path}</span>
			{/if}
		</Alert>
	{/each}
{/if}
