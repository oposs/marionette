<script lang="ts">
	import { getComponent } from '$lib/registry/registry';
	import { getData } from '$lib/store/data.svelte';
	import FallbackComponent from './FallbackComponent.svelte';
	import ErrorBoundary from './ErrorBoundary.svelte';
	import NodeRenderer from './NodeRenderer.svelte';
	import type { ComponentNode } from '$lib/transport/messages';

	let { nodeId, nodes, surface }: {
		nodeId: string;
		nodes: Record<string, ComponentNode>;
		surface: string;
	} = $props();

	let node = $derived(nodes[nodeId]);
	let ResolvedComponent = $derived(node ? getComponent(node.type) : undefined);
</script>

{#if node}
	{#if !node.visible || getData(surface, node.visible)}
		<ErrorBoundary>
			{#if ResolvedComponent}
				<ResolvedComponent
					props={node.props ?? {}}
					bind={node.bind}
					action={node.action}
					{surface}
				>
					{#snippet children()}
						{#if node.children}
							{#each node.children as childId (childId)}
								<NodeRenderer nodeId={childId} {nodes} {surface} />
							{/each}
						{/if}
					{/snippet}
				</ResolvedComponent>
			{:else}
				<FallbackComponent nodeType={node.type} props={node.props} {surface} />
			{/if}
		</ErrorBoundary>
	{/if}
{/if}
