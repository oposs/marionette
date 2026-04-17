<script lang="ts">
	import * as Field from '$lib/components/ui/field';
	import { Checkbox as ShadcnCheckbox } from '$lib/components/ui/checkbox';
	import { getData, setData } from '$lib/store/data.svelte';
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

	let checked = $derived(bind ? ((getData(surface, bind) as boolean) ?? false) : false);
	let fieldError = $derived(
		bind ? ((getData(surface, '/_errors' + bind) as string) ?? '') : ''
	);
	let hasError = $derived(!!fieldError);

	function handleCheckedChange(val: boolean | 'indeterminate') {
		if (bind) {
			setData(surface, bind, val === true);
		}
	}
</script>

<Field.Field
	orientation="horizontal"
	data-invalid={hasError || undefined}
	class={props.full_width ? 'col-span-full' : undefined}
>
	<ShadcnCheckbox
		id={fieldId}
		{checked}
		onCheckedChange={handleCheckedChange}
		disabled={props.disabled as boolean}
		aria-invalid={hasError || undefined}
	/>
	{#if props.label}
		<Field.Label for={fieldId}>{props.label}</Field.Label>
	{/if}
	{#if props.description && !hasError}
		<Field.Description>{props.description}</Field.Description>
	{/if}
	{#if fieldError}
		<Field.Error>{fieldError}</Field.Error>
	{/if}
</Field.Field>
