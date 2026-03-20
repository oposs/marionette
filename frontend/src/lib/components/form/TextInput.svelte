<script lang="ts">
	import { Input, Label, Helper } from 'flowbite-svelte';
	import { getData, setData } from '$lib/store/data.svelte';
	import { markDirty, clearDirty } from '$lib/store/dirty.svelte';
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

	let value = $derived(bind ? ((getData(surface, bind) as string) ?? '') : '');
	let fieldError = $derived(
		bind ? ((getData(surface, '/_errors' + bind) as string) ?? '') : ''
	);

	function handleInput(e: Event) {
		if (bind) {
			const target = e.currentTarget as HTMLInputElement;
			setData(surface, bind, target.value);
		}
	}

	function handleFocus() {
		if (bind) markDirty(bind);
	}

	function handleBlur() {
		if (bind) {
			clearDirty(bind, (op) => setData(surface, op.path, op.value));
			if (action?.type === 'blur') {
				sendAction(
					action.name ?? action.type,
					{ value: getData(surface, bind!) },
					action.target
				);
			}
		}
	}
</script>

<div class="w-full">
	{#if props.label}
		<Label class="mb-2">{props.label}</Label>
	{/if}
	<Input
		type={(props.type as string) ?? 'text'}
		placeholder={props.placeholder as string}
		required={props.required as boolean}
		disabled={props.disabled as boolean}
		{value}
		oninput={handleInput}
		onfocus={handleFocus}
		onblur={handleBlur}
		color={fieldError ? 'red' : undefined}
	/>
	{#if fieldError}
		<Helper class="mt-1 text-red-600">{fieldError}</Helper>
	{:else if props.helperText}
		<Helper class="mt-1">{props.helperText}</Helper>
	{/if}
</div>
