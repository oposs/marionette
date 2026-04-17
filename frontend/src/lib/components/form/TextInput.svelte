<script lang="ts">
	import * as Field from '$lib/components/ui/field';
	import { Input } from '$lib/components/ui/input';
	import { getData, setData } from '$lib/store/data.svelte';
	import { markDirty, clearDirty } from '$lib/store/dirty.svelte';
	import { sendAction } from '$lib/transport/dispatcher';
	import type { ComponentAction } from '$lib/transport/messages';

	let {
		props = {},
		bind,
		action,
		surface,
	}: {
		props: Record<string, unknown>;
		bind?: string;
		action?: ComponentAction;
		surface: string;
	} = $props();

	// D-B4: stable id — handler-supplied wins; fall back to mount-time UUID.
	// SPA-only (adapter-static + SPA fallback), so crypto.randomUUID() is safe.
	// The fallback is captured ONCE at mount — $derived keeps id stable across
	// rerenders even if other props change.
	const fallbackId = crypto.randomUUID();
	let fieldId = $derived((props.id as string) ?? fallbackId);

	let value = $derived(bind ? ((getData(surface, bind) as string) ?? '') : '');
	let fieldError = $derived(
		bind ? ((getData(surface, '/_errors' + bind) as string) ?? '') : ''
	);
	let hasError = $derived(!!fieldError);

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

<Field.Field
	data-invalid={hasError || undefined}
	class={props.full_width ? 'col-span-full' : undefined}
>
	{#if props.label}
		<Field.Label for={fieldId}>{props.label}</Field.Label>
	{/if}
	<Input
		id={fieldId}
		type={(props.input_type as string) ?? 'text'}
		placeholder={props.placeholder as string}
		required={props.required as boolean}
		disabled={props.disabled as boolean}
		aria-invalid={hasError || undefined}
		{value}
		oninput={handleInput}
		onfocus={handleFocus}
		onblur={handleBlur}
	/>
	{#if props.description && !hasError}
		<Field.Description>{props.description}</Field.Description>
	{/if}
	{#if fieldError}
		<Field.Error>{fieldError}</Field.Error>
	{/if}
</Field.Field>
