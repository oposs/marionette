<script lang="ts">
	import * as Field from '$lib/components/ui/field';
	import { Checkbox as ShadcnCheckbox } from '$lib/components/ui/checkbox';
	import { getData, setData } from '$lib/store/data.svelte';
	import { clearDirty } from '$lib/store/dirty.svelte';
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

	// Phase 18 Plan 02 — Framework Gap 2: blur-action dispatch.
	// Mirrors TextInput.svelte lines 45-56. bits-ui's Checkbox button does not
	// expose a reliable native onblur; we hook the focusout event on an outer
	// wrapper div so focus-leave from the control (or any nested label click
	// sequence) fires handleBlur once.
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

<!--
  The outer <div onfocusout> wrapper is the blur-signal catcher for the
  Checkbox. `class="contents"` makes the wrapper visually transparent so
  it does NOT disturb the Field.Field grid layout (D-B1 anatomy). The
  Field.Field retains its horizontal orientation from Phase 14 Plan 04.
-->
<div onfocusout={handleBlur} class="contents">
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
</div>
