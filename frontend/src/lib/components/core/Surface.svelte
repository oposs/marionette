<script lang="ts">
	import NodeRenderer from './NodeRenderer.svelte';
	import LoadingSkeleton from './LoadingSkeleton.svelte';
	import { getSurfaceTree } from '$lib/store/surfaces.svelte';

	let { name, class: className = '' }: {
		name: string;
		class?: string;
	} = $props();

	let tree = $derived(getSurfaceTree(name));

	// Surface-specific layout classes.
	// Phase 12: only `main` remains — it is the single top-level Surface
	// mounted by routes/+layout.svelte. `sidebar` / `modal` / `toast` are now
	// mounted recursively via SurfaceMount inside AppShell, and get no
	// top-level layout class (the shell handles their framing).
	const layoutClasses: Record<string, string> = {
		main: 'bg-background p-6 overflow-y-auto min-w-[320px] flex-1',
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
