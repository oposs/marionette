<script lang="ts">
	import { getSurfaceTree } from '$lib/store/surfaces.svelte';
	import NodeRenderer from '$lib/components/core/NodeRenderer.svelte';
	import { sendAction } from '$lib/transport/dispatcher';
	import * as Dialog from '$lib/components/ui/dialog';
	import type { ComponentNode } from '$lib/transport/messages';

	let tree = $derived(getSurfaceTree('modal'));

	// G-04 fix — Phase 17 gap-closure (Plan 17-05 Task 4).
	//
	// A Render message ALWAYS creates a surface tree via setSurfaceTree,
	// so `tree !== undefined` alone cannot discriminate open vs closed.
	// Backend handlers close the modal by rendering an empty Container
	// (id="modal-empty") as a close-sentinel — this derivation treats
	// such a tree as closed. Any other tree root (Container with children,
	// ConfirmDialog, etc.) opens the Dialog.
	//
	// See:
	//   backend/crates/gallery-demo/src/handlers/modal.rs (emits "modal-empty")
	//   backend/crates/gallery-demo/src/handlers/confirm.rs (confirm_close_with_toast)
	//   backend/crates/gallery-demo/src/handlers/navigate.rs (initial "modal-empty" seed)
	function isEmptyContainer(node: ComponentNode | undefined): boolean {
		if (!node) return true;
		if (node.type !== 'container') return false;
		return !node.children || node.children.length === 0;
	}

	let isOpen = $derived(
		tree !== undefined && !isEmptyContainer(tree.nodes[tree.root])
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
