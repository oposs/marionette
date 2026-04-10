<script lang="ts">
	import type { ComponentAction } from '$lib/transport/messages';
	import * as Sidebar from '$lib/components/ui/sidebar';
	import NodeRenderer from '$lib/components/core/NodeRenderer.svelte';
	import { getSurfaceTree } from '$lib/store/surfaces.svelte';

	// AppShell composes the six shell slots (sidebar / header / footer / main /
	// popups / toasts) using shadcn Sidebar primitives. Slot node IDs come in as
	// props (sidebarNodeId, headerNodeId, ...) and each slot mounts via the
	// existing NodeRenderer reading from the shell's own surface tree. Missing
	// slot IDs are skipped gracefully.
	// eslint-disable-next-line @typescript-eslint/no-unused-vars
	let { props = {}, bind: _bind, action: _action, surface }: {
		props: Record<string, unknown>;
		bind?: string;
		action?: ComponentAction;
		surface: string;
	} = $props();

	let sidebarId = $derived(props.sidebarNodeId as string | undefined);
	let headerId = $derived(props.headerNodeId as string | undefined);
	let footerId = $derived(props.footerNodeId as string | undefined);
	let mainId = $derived(props.mainNodeId as string | undefined);
	let popupsId = $derived(props.popupsNodeId as string | undefined);
	let toastsId = $derived(props.toastsNodeId as string | undefined);

	// The shell lives IN `surface` — look up slot children from this surface's tree.
	let tree = $derived(getSurfaceTree(surface));
	let nodes = $derived(tree?.nodes ?? {});
</script>

<Sidebar.Provider>
	<Sidebar.Root collapsible="offcanvas">
		<Sidebar.Content>
			{#if sidebarId && nodes[sidebarId]}
				<NodeRenderer nodeId={sidebarId} {nodes} {surface} />
			{/if}
		</Sidebar.Content>
	</Sidebar.Root>
	<Sidebar.Inset>
		<div class="flex min-h-screen flex-col">
			<header class="flex items-center gap-2 border-b bg-background px-4 py-2">
				<Sidebar.Trigger />
				{#if headerId && nodes[headerId]}
					<div class="flex flex-1 items-center justify-between">
						<NodeRenderer nodeId={headerId} {nodes} {surface} />
					</div>
				{/if}
			</header>
			<main class="flex-1 overflow-auto bg-background">
				{#if mainId && nodes[mainId]}
					<NodeRenderer nodeId={mainId} {nodes} {surface} />
				{/if}
			</main>
			<footer class="border-t bg-background px-4 py-2 text-xs text-muted-foreground">
				{#if footerId && nodes[footerId]}
					<NodeRenderer nodeId={footerId} {nodes} {surface} />
				{/if}
			</footer>
			{#if popupsId && nodes[popupsId]}
				<NodeRenderer nodeId={popupsId} {nodes} {surface} />
			{/if}
			{#if toastsId && nodes[toastsId]}
				<NodeRenderer nodeId={toastsId} {nodes} {surface} />
			{/if}
		</div>
	</Sidebar.Inset>
</Sidebar.Provider>
