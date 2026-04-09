<script lang="ts">
	import { getSurfaceTree } from '$lib/store/surfaces.svelte';
	import NodeRenderer from '$lib/components/core/NodeRenderer.svelte';
	import { sendAction } from '$lib/transport/dispatcher';
	import * as Dialog from '$lib/components/ui/dialog';

	let tree = $derived(getSurfaceTree('modal'));
	let isOpen = $derived(tree !== undefined);

	let rootProps = $derived(
		tree ? ((tree.nodes[tree.root]?.props ?? {}) as Record<string, unknown>) : {}
	);

	function handleClose() {
		sendAction('close-modal');
	}
</script>

<Dialog.Root open={isOpen} onOpenChange={(open) => { if (!open) handleClose(); }}>
	<Dialog.Content class="sm:max-w-lg">
		{#if tree}
			<NodeRenderer nodeId={tree.root} nodes={tree.nodes} surface="modal" />
		{/if}
	</Dialog.Content>
</Dialog.Root>
