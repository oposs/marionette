<script lang="ts">
	import * as Field from '$lib/components/ui/field';
	import * as Select from '$lib/components/ui/select';
	import { getAllData, getData, setData } from '$lib/store/data.svelte';
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
	let options = $derived(
		(props.options as Array<{ value: string; label: string }>) ?? []
	);
	let fieldError = $derived(
		bind ? ((getData(surface, '/_errors' + bind) as string) ?? '') : ''
	);
	let hasError = $derived(!!fieldError);

	function handleValueChange(newValue: string) {
		if (bind) {
			setData(surface, bind, newValue);
		}
		// If a `change` action is wired, dispatch it to the backend with
		// the full surface data payload (mirrors the Button pattern in
		// `Button.svelte`). This is what Phase 12 Plan 08's country-select
		// demo relies on to trigger node-patch flows (D-A6 focus
		// preservation + D-B15 toast lifecycle). Payload shape MUST stay
		// byte-identical: `{ ...(action.payload ?? {}), ...surfaceData }`.
		if (action?.type === 'change' && action.name) {
			const surfaceData = getAllData(surface) ?? {};
			const payload = {
				...((action.payload as Record<string, unknown>) ?? {}),
				...surfaceData,
			};
			sendAction(action.name, payload, action.target);
		}
	}

	function handleOpenChange(open: boolean) {
		if (!bind) return;
		if (open) {
			markDirty(bind);
		} else {
			// Pair mark/clear with open/close (mirrors focus/blur in TextInput)
			// so that dismissing the dropdown without a selection still clears
			// the dirty flag and does not strand pending optimistic state.
			// handleBlur handles both the clearDirty and the optional blur
			// action dispatch (Phase 18 Plan 02 — Framework Gap 2).
			handleBlur();
		}
	}

	// Phase 18 Plan 02 — Framework Gap 2: blur-action dispatch.
	// Mirrors TextInput.svelte lines 45-56. For a Select, the logical "blur"
	// moment is the popover close, not a focus-leave on the trigger (the
	// trigger briefly loses focus when the items portal opens, so focus-leave
	// would fire during interaction). handleOpenChange(false) is the correct
	// hook; this function is invoked from there and mirrors the dispatch
	// shape used by TextInput / Textarea.
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
	<Select.Root
		type="single"
		{value}
		onValueChange={handleValueChange}
		onOpenChange={handleOpenChange}
		disabled={props.disabled as boolean}
	>
		<Select.Trigger
			id={fieldId}
			class="w-full"
			aria-invalid={hasError || undefined}
		>
			{#if value && options.find((o) => o.value === value)}
				<span>{options.find((o) => o.value === value)?.label}</span>
			{:else}
				<span class="text-muted-foreground">{props.placeholder ?? 'Select...'}</span>
			{/if}
		</Select.Trigger>
		<Select.Content>
			{#each options as opt (opt.value)}
				<Select.Item value={opt.value} label={opt.label} />
			{/each}
		</Select.Content>
	</Select.Root>
	{#if props.description && !hasError}
		<Field.Description>{props.description}</Field.Description>
	{/if}
	{#if fieldError}
		<Field.Error>{fieldError}</Field.Error>
	{/if}
</Field.Field>
