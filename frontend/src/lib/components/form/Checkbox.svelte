<script lang="ts">
	import { Checkbox as ShadcnCheckbox } from '$lib/components/ui/checkbox';
	import { Label } from '$lib/components/ui/label';
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

	function handleCheckedChange(val: boolean | 'indeterminate') {
		if (bind) {
			setData(surface, bind, val === true);
		}
	}
</script>

<div class="flex items-center gap-2">
	<ShadcnCheckbox
		{checked}
		onCheckedChange={handleCheckedChange}
		disabled={props.disabled as boolean}
	/>
	{#if props.label}
		<Label class="font-semibold">{props.label}</Label>
	{/if}
</div>
