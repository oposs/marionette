<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { ComponentAction } from '$lib/transport/messages';
	import { Card } from 'flowbite-svelte';

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
</script>

<Card class="{paddingClass} {(props.class as string) ?? ''}">
	{@render children?.()}
</Card>
