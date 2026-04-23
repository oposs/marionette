<script lang="ts">
	import { Button as ShadcnButton, type ButtonVariant, type ButtonSize } from '$lib/components/ui/button';
	import { getIcon } from '$lib/registry/icons';
	import Loader2 from '@lucide/svelte/icons/loader-2';
	import { sendAction } from '$lib/transport/dispatcher';
	import { getAllData } from '$lib/store/data.svelte';
	import type { ComponentAction, PatchOperation } from '$lib/transport/messages';
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

	// -------------------------------------------------------------------------
	// Phase 18 Plan 01 (Gap 1) — variant / size / loading / icon / aria_label
	// are now read directly from the backend-authoritative props, dropping
	// the Phase 11 legacy color/outline derivation. Pre-deployment posture:
	// no back-compat fallback. See .planning/phases/18-catalog-screens/
	// 18-RESEARCH.md §Q5 + §Gap 1 and Button.browser-test.ts for the contract.
	// -------------------------------------------------------------------------

	let variant = $derived(
		((props.variant as string | undefined) ?? 'default') as ButtonVariant
	);

	let isIconOnly = $derived(!props.label && !!props.icon);

	let size = $derived(
		((props.size as string | undefined) ?? (isIconOnly ? 'icon' : 'default')) as ButtonSize
	);

	let isLoading = $derived(props.loading === true);

	// aria-label policy: only emit when icon-only (no visible label). When a
	// visible label is present, it becomes the accessible name automatically
	// and an aria-label override would just clobber assistive-tech output.
	let ariaLabel = $derived(
		isIconOnly
			? ((props.aria_label as string | undefined) ??
				(props.icon as string | undefined) ??
				'button')
			: undefined
	);

	function handleClick() {
		if (action) {
			const optimisticField = action.optimistic as
				| { patch: PatchOperation[] }
				| undefined;
			// Include all surface data as payload so the backend receives form values
			const surfaceData = getAllData(surface) ?? {};
			const payload = {
				...(action.payload as Record<string, unknown> ?? {}),
				...surfaceData
			};
			sendAction(
				// Do NOT fall back to action.type here: `type` is a protocol
				// classifier (e.g. 'destructive'), not a backend action name.
				// If the server omits `name`, use a generic literal.
				action.name ?? 'button-click',
				payload,
				action.target,
				optimisticField ? { patch: optimisticField.patch } : undefined
			);
		}
	}
</script>

<ShadcnButton
	{variant}
	{size}
	disabled={isLoading || (props.disabled as boolean | undefined) === true}
	onclick={handleClick}
	class={props.icon && props.label ? 'gap-2' : ''}
	aria-label={ariaLabel}
	aria-busy={isLoading ? 'true' : undefined}
>
	{#if isLoading}
		<Loader2 class="size-4 animate-spin" aria-hidden="true" />
	{:else if props.icon}
		{@const IconComp = getIcon(props.icon as string)}
		<IconComp class="size-4" aria-hidden="true" />
	{/if}
	{#if props.label}
		{props.label}
	{/if}
</ShadcnButton>
