<script lang="ts">
	import { Checkbox as FlowbiteCheckbox } from 'flowbite-svelte';
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

<FlowbiteCheckbox
	{checked}
	disabled={props.disabled as boolean}
	onchange={handleChange}
>
	{#if props.label}{props.label}{/if}
</FlowbiteCheckbox>
