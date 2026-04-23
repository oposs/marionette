<script lang="ts">
	import * as Field from '$lib/components/ui/field';
	import { RadioGroup, RadioGroupItem } from '$lib/components/ui/radio-group';
	import { Label } from '$lib/components/ui/label';
	import { getData, setData } from '$lib/store/data.svelte';
	import { clearDirty } from '$lib/store/dirty.svelte';
	import { sendAction } from '$lib/transport/dispatcher';
	import type { ComponentAction } from '$lib/transport/messages';

	type RadioOption = { value: string; label: string; description?: string };

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

	// D-B4: stable group id — handler-supplied wins; fall back to mount-time UUID.
	// SPA-only (adapter-static + SPA fallback), so crypto.randomUUID() is safe.
	const fallbackId = crypto.randomUUID();
	let groupId = $derived((props.id as string) ?? fallbackId);

	let options = $derived((props.options as RadioOption[]) ?? []);
	let value = $derived(bind ? ((getData(surface, bind) as string) ?? '') : '');
	let fieldError = $derived(
		bind ? ((getData(surface, '/_errors' + bind) as string) ?? '') : ''
	);
	let hasError = $derived(!!fieldError);

	function handleValueChange(newValue: string) {
		if (bind) {
			setData(surface, bind, newValue);
		}
	}

	// Phase 18 Plan 02 — Framework Gap 2: blur-action dispatch.
	// Mirrors TextInput.svelte lines 45-56. bits-ui's RadioGroup does not expose
	// a native onblur at the group level; wrapping in `<div onfocusout>` is the
	// canonical pattern here (as for Checkbox/Switch). focusout bubbles from
	// any individual RadioGroupItem so tab-out from any option triggers this.
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
  group. `class="contents"` keeps the wrapper visually transparent so the
  Field.Field grid layout stays intact (D-B1 anatomy preserved).
-->
<div onfocusout={handleBlur} class="contents">
	<Field.Field
		data-invalid={hasError || undefined}
		class={props.full_width ? 'col-span-full' : undefined}
	>
		{#if props.label}
			<Field.Label>{props.label}</Field.Label>
		{/if}
		<RadioGroup
			{value}
			onValueChange={handleValueChange}
			disabled={props.disabled as boolean}
			aria-invalid={hasError || undefined}
		>
			{#each options as opt (opt.value)}
				{@const itemId = `${groupId}-${opt.value}`}
				<div class="flex items-start gap-2">
					<RadioGroupItem value={opt.value} id={itemId} />
					<div class="grid gap-1">
						<Label for={itemId} class="font-semibold">{opt.label}</Label>
						{#if opt.description}
							<p class="text-xs text-muted-foreground">{opt.description}</p>
						{/if}
					</div>
				</div>
			{/each}
		</RadioGroup>
		{#if props.description && !hasError}
			<Field.Description>{props.description}</Field.Description>
		{/if}
		{#if fieldError}
			<Field.Error>{fieldError}</Field.Error>
		{/if}
	</Field.Field>
</div>
