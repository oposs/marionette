<script lang="ts">
	import ChevronUp from '@lucide/svelte/icons/chevron-up';
	import ChevronDown from '@lucide/svelte/icons/chevron-down';
	import { getData } from '$lib/store/data.svelte';
	import { sendAction } from '$lib/transport/dispatcher';
	import type { ComponentAction } from '$lib/transport/messages';
	import type { Snippet } from 'svelte';

	const ROW_HEIGHT = 48;
	const BUFFER = 3;
	const CHUNK_SIZE = 50;

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

	type ColumnDef = { key: string; label: string; sortable?: boolean };

	let columns = $derived((props.columns as ColumnDef[]) ?? []);
	let explicitTotalRows = $derived((props.totalRows as number) ?? 0);
	let rowIdKey = $derived((props.rowIdKey as string) ?? 'id');

	// Virtual scroll state
	let scrollTop = $state(0);
	let containerHeight = $state(0);

	// Sort state
	let sortColumn = $state('');
	let sortDir = $state<'asc' | 'desc' | ''>('');

	// Derive rows from keyed collection data
	let rawData = $derived(
		bind ? ((getData(surface, bind) as Record<string, Record<string, unknown>>) ?? {}) : {}
	);
	let rows = $derived(Object.entries(rawData));

	// Use explicit totalRows from props if provided, otherwise fall back to actual row count.
	// This allows virtual scrolling for large server-paginated datasets while still
	// working correctly when all data is sent at once (no totalRows prop).
	let totalRows = $derived(explicitTotalRows > 0 ? explicitTotalRows : rows.length);

	// Virtual scroll computed values
	let totalHeight = $derived(totalRows * ROW_HEIGHT);
	let visibleStart = $derived(
		Math.max(0, Math.floor(scrollTop / ROW_HEIGHT) - BUFFER)
	);
	let visibleEnd = $derived(
		Math.min(totalRows, Math.ceil((scrollTop + containerHeight) / ROW_HEIGHT) + BUFFER)
	);
	let visibleRows = $derived(rows.slice(visibleStart, visibleEnd));
	let offsetY = $derived(visibleStart * ROW_HEIGHT);

	// Prefetch trigger
	$effect(() => {
		if (explicitTotalRows > 0 && visibleEnd > 0 && visibleEnd >= rows.length - CHUNK_SIZE * 2 && rows.length < explicitTotalRows) {
			sendAction('fetch-rows', { offset: rows.length, limit: CHUNK_SIZE });
		}
	});

	function handleSort(col: ColumnDef) {
		if (!col.sortable) return;

		if (sortColumn === col.key) {
			if (sortDir === 'asc') {
				sortDir = 'desc';
			} else if (sortDir === 'desc') {
				sortDir = '';
				sortColumn = '';
			}
		} else {
			sortColumn = col.key;
			sortDir = 'asc';
		}

		if (sortColumn && sortDir) {
			sendAction('sort', { column: sortColumn, direction: sortDir });
		}
	}

	function handleRowClick(rowId: string) {
		if (action) {
			sendAction(action.name ?? 'select-row', { id: rowId }, action.target);
		}
	}

	function handleScroll(e: Event) {
		const target = e.currentTarget as HTMLDivElement;
		scrollTop = target.scrollTop;
	}
</script>

<div
	class="overflow-y-auto"
	style="height: 100%;"
	onscroll={handleScroll}
	bind:clientHeight={containerHeight}
>
	<div style="height: {totalHeight}px; position: relative;">
		<div style="transform: translateY({offsetY}px);">
			<table class="w-full text-left text-sm">
				<thead class="bg-muted text-xs uppercase text-foreground">
					<tr>
						{#each columns as col (col.key)}
							<th
								class="px-6 py-3 {col.sortable ? 'cursor-pointer hover:bg-accent' : ''}"
								onclick={() => handleSort(col)}
							>
								{col.label}
								{#if sortColumn === col.key}
									{#if sortDir === 'asc'}
										<ChevronUp class="size-4 inline" />
									{:else if sortDir === 'desc'}
										<ChevronDown class="size-4 inline" />
									{/if}
								{/if}
							</th>
						{/each}
					</tr>
				</thead>
				<tbody>
					{#each visibleRows as [rowKey, rowData] (rowKey)}
						<tr
							style="height: {ROW_HEIGHT}px"
							class="border-b border-border bg-background hover:bg-accent {action ? 'cursor-pointer' : ''}"
							onclick={() => handleRowClick(String(rowData[rowIdKey] ?? rowKey))}
						>
							{#each columns as col (col.key)}
								<td class="px-6 py-4 text-muted-foreground">{String(rowData[col.key] ?? '')}</td>
							{/each}
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
	</div>
</div>
