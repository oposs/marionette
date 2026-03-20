<script lang="ts">
	import { Modal } from 'flowbite-svelte';
	import type { ModalProps } from 'flowbite-svelte';
	import { getSurfaceTree } from '$lib/store/surfaces.svelte';
	import NodeRenderer from '$lib/components/core/NodeRenderer.svelte';
	import { sendAction } from '$lib/transport/dispatcher';

	let tree = $derived(getSurfaceTree('modal'));
	let isOpen = $derived(tree !== undefined);

	let rootProps = $derived(
		tree ? ((tree.nodes[tree.root]?.props ?? {}) as Record<string, unknown>) : {}
	);

	// Map size prop to Flowbite Modal size: sm -> xs, md -> sm, lg -> md (Flowbite's sizes are one step larger)
	const sizeMap: Record<string, ModalProps['size']> = {
		sm: 'xs',
		md: 'sm',
		lg: 'md',
	};
	let modalSize = $derived(sizeMap[(rootProps.size as string) ?? 'md'] ?? 'sm');

	function handleClose() {
		sendAction('close-modal');
	}
</script>

<Modal
	open={isOpen}
	size={modalSize}
	dismissable
	onclose={handleClose}
>
	{#if tree}
		<NodeRenderer nodeId={tree.root} nodes={tree.nodes} surface="modal" />
	{/if}
</Modal>
