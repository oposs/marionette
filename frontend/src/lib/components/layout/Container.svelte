<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { ComponentAction } from '$lib/transport/messages';
	import * as Card from '$lib/components/ui/card';
	import { getIcon } from '$lib/registry/icons';

	let {
		props = {},
		bind,
		action,
		surface,
		children
	}: {
		props: Record<string, unknown>;
		bind?: string;
		action?: ComponentAction;
		surface: string;
		children?: Snippet;
	} = $props();

	const paddingMap: Record<string, string> = {
		none: 'p-0',
		xs: 'p-1',
		sm: 'p-2',
		md: 'p-4',
		lg: 'p-6',
		xl: 'p-8'
	};

	let paddingClass = $derived(
		paddingMap[(props.padding as string) ?? 'md'] ?? 'p-4'
	);

	let isCard = $derived(Boolean(props.card));

	// CAT-05 addition (Plan 18-08 Task 0): optional display-only lucide icon.
	// Mirrors NavItem.svelte's getIcon derivation; when props.icon is absent
	// (the common case) this stays undefined and no icon renders.
	let IconComponent = $derived(
		props.icon ? getIcon(props.icon as string) : undefined
	);
</script>

{#if isCard}
	<div class="flex items-start md:items-center justify-center min-h-0 md:min-h-[60vh]">
		<Card.Root class="w-full max-w-md {paddingClass} {(props.class as string) ?? ''}">
			{#if IconComponent}
				<IconComponent class="size-4" aria-hidden="true" />
			{/if}
			{@render children?.()}
		</Card.Root>
	</div>
{:else}
	<div class="flex flex-col flex-1 min-h-0 {paddingClass} {(props.class as string) ?? ''}">
		{#if IconComponent}
			<IconComponent class="size-4" aria-hidden="true" />
		{/if}
		{@render children?.()}
	</div>
{/if}
