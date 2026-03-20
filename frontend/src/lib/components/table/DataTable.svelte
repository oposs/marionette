<script lang="ts">
	import {
		Table,
		TableHead,
		TableBody,
		TableHeadCell,
		TableBodyRow,
		TableBodyCell,
	} from 'flowbite-svelte';
	import { ChevronUpOutline, ChevronDownOutline } from 'flowbite-svelte-icons';
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
	let totalRows = $derived((props.totalRows as number) ?? 0);
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
		if (visibleEnd > 0 && visibleEnd >= rows.length - CHUNK_SIZE * 2 && rows.length < totalRows) {
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
			<Table>
				<TableHead>
					{#each columns as col (col.key)}
						<TableHeadCell
							class={col.sortable ? 'cursor-pointer hover:bg-gray-100' : ''}
							onclick={() => handleSort(col)}
						>
							{col.label}
							{#if sortColumn === col.key}
								{#if sortDir === 'asc'}
									<ChevronUpOutline class="w-4 h-4 inline" />
								{:else if sortDir === 'desc'}
									<ChevronDownOutline class="w-4 h-4 inline" />
								{/if}
							{/if}
						</TableHeadCell>
					{/each}
				</TableHead>
				<TableBody>
					{#each visibleRows as [rowKey, rowData] (rowKey)}
						<TableBodyRow
							style="height: {ROW_HEIGHT}px"
							class={action ? 'cursor-pointer' : ''}
							onclick={() => handleRowClick(String(rowData[rowIdKey] ?? rowKey))}
						>
							{#each columns as col (col.key)}
								<TableBodyCell>{String(rowData[col.key] ?? '')}</TableBodyCell>
							{/each}
						</TableBodyRow>
					{/each}
				</TableBody>
			</Table>
		</div>
	</div>
</div>
