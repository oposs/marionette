<script lang="ts">
	import NodeRenderer from './NodeRenderer.svelte';
	import LoadingSkeleton from './LoadingSkeleton.svelte';
	import { getSurfaceTree } from '$lib/store/surfaces.svelte';

	let { name, class: className = '' }: {
		name: string;
		class?: string;
	} = $props();

	let tree = $derived(getSurfaceTree(name));

	// Surface-specific layout classes
	const layoutClasses: Record<string, string> = {
		main: 'bg-background p-6 overflow-y-auto min-w-[320px] flex-1',
		sidebar: 'bg-sidebar border-r border-sidebar-border p-4 overflow-y-auto w-64 shrink-0',
		modal: '',
		toast: '',
	};

	let surfaceClass = $derived(layoutClasses[name] ?? '');
</script>

<div class="{surfaceClass} {className}" data-surface={name}>
	{#if tree}
		<NodeRenderer nodeId={tree.root} nodes={tree.nodes} surface={name} />
	{:else}
		<LoadingSkeleton />
	{/if}
</div>
