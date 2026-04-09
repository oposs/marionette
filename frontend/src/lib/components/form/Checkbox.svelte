<script lang="ts">
	import { getData, setData } from '$lib/store/data.svelte';
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

	let checked = $derived(bind ? ((getData(surface, bind) as boolean) ?? false) : false);

	function handleChange(e: Event) {
		if (bind) {
			const target = e.currentTarget as HTMLInputElement;
			setData(surface, bind, target.checked);
		}
	}
</script>

<label class="inline-flex items-center gap-2 text-sm font-medium text-foreground">
	<input
		type="checkbox"
		checked={checked}
		disabled={props.disabled as boolean}
		onchange={handleChange}
		class="size-4 rounded border-input text-primary focus:ring-2 focus:ring-ring"
	/>
	{#if props.label}{props.label}{/if}
</label>
