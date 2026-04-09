<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { ComponentAction } from '$lib/transport/messages';

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
		<div class="w-full max-w-md rounded-lg border border-border bg-card shadow-sm {paddingClass} {(props.class as string) ?? ''}">
			{@render children?.()}
		</div>
	</div>
{:else}
	<div class="flex flex-col flex-1 min-h-0 {paddingClass} {(props.class as string) ?? ''}">
		{@render children?.()}
	</div>
{/if}
