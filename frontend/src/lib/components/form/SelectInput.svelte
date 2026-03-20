<script lang="ts">
	import { Select, Label } from 'flowbite-svelte';
	import { getData, setData } from '$lib/store/data.svelte';
	import { markDirty, clearDirty } from '$lib/store/dirty.svelte';
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

	let value = $derived(bind ? ((getData(surface, bind) as string) ?? '') : '');
	let options = $derived(
		(props.options as Array<{ value: string; label: string }>) ?? []
	);

	function handleChange(e: Event) {
		if (bind) {
			const target = e.currentTarget as HTMLSelectElement;
			setData(surface, bind, target.value);
		}
	}

	function handleFocus() {
		if (bind) markDirty(bind);
	}

	function handleBlur() {
		if (bind) {
			clearDirty(bind, (op) => setData(surface, op.path, op.value));
		}
	}
</script>

<div class="w-full">
	{#if props.label}
		<Label class="mb-2">{props.label}</Label>
	{/if}
	<Select
		{value}
		disabled={props.disabled as boolean}
		placeholder={props.placeholder as string}
		onchange={handleChange}
		onfocus={handleFocus}
		onblur={handleBlur}
	>
		{#if props.placeholder}
			<option value="" disabled selected>{props.placeholder}</option>
		{/if}
		{#each options as opt (opt.value)}
			<option value={opt.value}>{opt.label}</option>
		{/each}
	</Select>
</div>
