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
	{@const nodeProps = node.props ?? {}}
	{@const nodeBind = node.bind}
	{@const nodeAction = node.action}
	{@const nodeVisible = node.visible}
	{@const nodeChildren = node.children}
	{@const nodeType = node.type}
	{#if !nodeVisible || getData(surface, nodeVisible)}
		<ErrorBoundary>
			{#if ResolvedComponent}
				<ResolvedComponent
					props={nodeProps}
					bind={nodeBind}
					action={nodeAction}
					{surface}
				>
					{#snippet children()}
						{#if nodeChildren}
							{#each nodeChildren as childId (childId)}
								<NodeRenderer nodeId={childId} {nodes} {surface} />
							{/each}
						{/if}
					{/snippet}
				</ResolvedComponent>
			{:else}
				<FallbackComponent nodeType={nodeType} props={nodeProps} {surface} />
			{/if}
		</ErrorBoundary>
	{/if}
{/if}
