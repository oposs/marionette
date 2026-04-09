<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { ComponentAction } from '$lib/transport/messages';
	import * as Card from '$lib/components/ui/card';

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
</script>

{#if isCard}
	<div class="flex items-start md:items-center justify-center min-h-0 md:min-h-[60vh]">
		<Card.Root class="w-full max-w-md {paddingClass} {(props.class as string) ?? ''}">
			{@render children?.()}
		</Card.Root>
	</div>
{:else}
	<div class="flex flex-col flex-1 min-h-0 {paddingClass} {(props.class as string) ?? ''}">
		{@render children?.()}
	</div>
{/if}
