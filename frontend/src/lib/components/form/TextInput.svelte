<script lang="ts">
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
		<label class="mb-2 block text-sm font-medium text-foreground">
			{props.label}
		</label>
	{/if}
	<input
		type={(props.type as string) ?? 'text'}
		placeholder={props.placeholder as string}
		required={props.required as boolean}
		disabled={props.disabled as boolean}
		{value}
		oninput={handleInput}
		onfocus={handleFocus}
		onblur={handleBlur}
		class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50 {fieldError ? 'border-destructive' : ''}"
	/>
	{#if fieldError}
		<p class="mt-1 text-sm text-destructive">{fieldError}</p>
	{:else if props.helperText}
		<p class="mt-1 text-sm text-muted-foreground">{props.helperText}</p>
	{/if}
</div>
