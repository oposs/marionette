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

	// The bound value can take two shapes depending on producer:
	//   1. string — a single field-error message (blur-validate handlers
	//      write this at `/_errors/<field>` paths; see docs/OpenSDUI-CONCEPT.md
	//      "Errors as data").
	//   2. ErrorEntry[] — a form-level error list (docs/OpenSDUI-CONCEPT.md §Data).
	// Previously this component cast the value as ErrorEntry[] unconditionally,
	// which iterated a string as its characters — producing 29 empty boxes
	// for "Enter a valid email address." Normalise to an ErrorEntry[] up
	// front, dropping anything without a message string.
	let errors = $derived.by((): ErrorEntry[] => {
		if (!bind) return [];
		const raw = getData(surface, bind);
		if (raw == null || raw === '') return [];
		if (typeof raw === 'string') return [{ message: raw }];
		if (Array.isArray(raw)) {
			return raw
				.filter((e): e is ErrorEntry =>
					typeof e === 'object' &&
					e !== null &&
					typeof (e as ErrorEntry).message === 'string' &&
					(e as ErrorEntry).message.length > 0
				);
		}
		return [];
	});
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
