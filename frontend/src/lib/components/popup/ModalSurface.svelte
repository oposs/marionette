<script lang="ts">
	import { getSurfaceTree } from '$lib/store/surfaces.svelte';
	import NodeRenderer from '$lib/components/core/NodeRenderer.svelte';
	import { sendAction } from '$lib/transport/dispatcher';

	let tree = $derived(getSurfaceTree('modal'));
	let isOpen = $derived(tree !== undefined);

	let rootProps = $derived(
		tree ? ((tree.nodes[tree.root]?.props ?? {}) as Record<string, unknown>) : {}
	);

	function handleClose() {
		sendAction('close-modal');
	}
</script>

{#if isOpen}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
		onclick={handleClose}
		onkeydown={(e) => e.key === 'Escape' && handleClose()}
	>
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div
			class="relative w-full max-w-md rounded-lg bg-background shadow-lg"
			onclick={(e) => e.stopPropagation()}
			onkeydown={() => {}}
		>
			{#if tree}
				<NodeRenderer nodeId={tree.root} nodes={tree.nodes} surface="modal" />
			{/if}
		</div>
	</div>
{/if}
