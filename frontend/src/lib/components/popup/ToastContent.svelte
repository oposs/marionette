<script lang="ts">
	import NodeRenderer from '$lib/components/core/NodeRenderer.svelte';
	import type { ComponentNode } from '$lib/transport/messages';

	// Wrapper for sonner's `toast.custom()` path — renders a server-supplied
	// SDUI node tree inside toast chrome. Sonner owns the overlay mechanics
	// (stacking, fade, position, countdown); the tree here owns content.
	// See CONCEPT.md §"Where the Client Is Smart".
	//
	// The toast lives outside any normal Surface, so we supply a synthetic
	// surface name ("__toast__") for NodeRenderer. Data bindings on toast
	// nodes read from this synthetic surface's data store (empty by default).
	let {
		root,
		nodes,
	}: {
		root: string;
		nodes: Record<string, ComponentNode>;
	} = $props();
</script>

<NodeRenderer nodeId={root} {nodes} surface="__toast__" />
