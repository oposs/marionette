<script lang="ts">
	import { Button as ShadcnButton } from '$lib/components/ui/button';
	import Filter from '@lucide/svelte/icons/filter';
	import NodeRenderer from '$lib/components/core/NodeRenderer.svelte';
	import { sendAction } from '$lib/transport/dispatcher';
	import { getAllData } from '$lib/store/data.svelte';
	import type { ComponentAction, ComponentNode } from '$lib/transport/messages';
	import type { Snippet } from 'svelte';

	let {
		props = {},
		bind,
		action,
		surface,
		children,
	}: {
		props: Record<string, unknown>;
		bind?: string;
		action?: ComponentAction;
		surface: string;
		children?: Snippet;
	} = $props();

	type ToolbarAction = ComponentAction & { label?: string; icon?: string };
	type FilterDef = { id: string; label?: string };

	let title = $derived((props.title as string) ?? '');
	let toolbarActions = $derived((props.toolbar as ToolbarAction[]) ?? []);
	let filters = $derived((props.filters as FilterDef[]) ?? []);
	let filterColumns = $derived((props.filterColumns as number) ?? 3);
	let nodes = $derived((props.nodes as Record<string, ComponentNode>) ?? {});
	let screenNodeId = $derived((props.screenNodeId as string) ?? '');
	let tableId = $derived((props.tableId as string) ?? '');

	let filtersVisible = $state(false);

	function handleToolbarAction(act: ToolbarAction) {
		const surfaceData = getAllData(surface) ?? {};
		const payload = {
			...(act.payload as Record<string, unknown> ?? {}),
			...surfaceData,
		};
		sendAction(act.name ?? act.type, payload, act.target);
	}

	function handleFilterSubmit(e: SubmitEvent) {
		e.preventDefault();
		const surfaceData = getAllData(surface) ?? {};
		sendAction('filter', surfaceData);
	}
</script>

<div class="flex flex-col gap-4 h-full">
	<!-- Header -->
	<div class="flex items-center justify-between">
		{#if title}
			<h1 class="text-xl font-semibold text-foreground">{title}</h1>
		{/if}

		<!-- Toolbar -->
		<div class="flex items-center gap-2">
			{#if filters.length > 0}
				<ShadcnButton variant="ghost" size="sm" class="md:hidden" onclick={() => (filtersVisible = !filtersVisible)}>
					<Filter class="size-4" />
					{filtersVisible ? 'Hide Filters' : 'Show Filters'}
				</ShadcnButton>
			{/if}
			{#each toolbarActions as act}
				<ShadcnButton
					variant={act.type === 'primary' ? 'default' : 'outline'}
					onclick={() => handleToolbarAction(act)}
				>
					{act.label ?? act.name ?? ''}
				</ShadcnButton>
			{/each}
		</div>
	</div>

	<!-- Filter area -->
	{#if filters.length > 0}
		<form
			onsubmit={handleFilterSubmit}
			class="hidden md:block {filtersVisible ? '!block' : ''}"
		>
			<div class="grid gap-4" style="grid-template-columns: repeat({filterColumns}, 1fr)">
				{#each filters as filter (filter.id)}
					<NodeRenderer nodeId={filter.id} {nodes} {surface} />
				{/each}
			</div>
			<div class="flex justify-end mt-2">
				<ShadcnButton variant="outline" size="sm" type="submit">Apply Filters</ShadcnButton>
			</div>
		</form>
	{/if}

	<!-- Table content -->
	<div class="flex-1 min-h-0">
		{#if screenNodeId && nodes[screenNodeId]}
			<NodeRenderer nodeId={screenNodeId} {nodes} {surface} />
		{:else if tableId && nodes[tableId]}
			<NodeRenderer nodeId={tableId} {nodes} {surface} />
		{/if}
		{@render children?.()}
	</div>
</div>
