<script lang="ts">
	/**
	 * DataTable.svelte — Phase 13 Plan 05 rewrite.
	 *
	 * Adopts the shadcn-svelte data-table recipe verbatim, composed of:
	 *   • createSvelteTable from $lib/components/ui/data-table (TanStack Table adapter)
	 *   • createRuneVirtualizer from $lib/utils/virtualizer.svelte (virtual-core-direct
	 *     wrapper — the @tanstack/svelte-virtual store adapter is broken under
	 *     Svelte 5, see SvelteVirtualSmoke.svelte's decision comment).
	 *   • onIntersect from $lib/actions/viewport (IntersectionObserver sentinel)
	 *   • Per-kind cell snippets from datatable-cells.svelte + DataTableActions
	 *   • shadcn Table / Input / Select / DropdownMenu / Button primitives
	 *
	 * Props contract (server-driven, all optional unless noted):
	 *   columns: [{ key, label, sortable?, kind?, hidden_default? }]   REQUIRED
	 *   filters: [{ id, kind: 'text'|'select'|'date-range', label, options?, placeholder? }]
	 *   total_rows: number       — when set, sentinel idles once rows.length >= total_rows
	 *   page_size:  number       — fetch-rows limit per dispatch (default 50)
	 *   row_id_key: string       — field name used as row identity (default 'id')
	 *   source:     string       — backend fetch-rows source identifier; when empty
	 *                              or absent, the sentinel is a no-op (graceful
	 *                              degradation against pre-migration CRM handlers).
	 *
	 * Stale-fetch-rows discard (D-H3): `lastFetchRowsActionId` records the UUID
	 * returned by `sendAction('fetch-rows', …)`. Stale-discard is ENFORCED by the
	 * `fetching` in-flight flag + server FIFO ordering on a single WebSocket
	 * connection (see backend/crates/marionette/src/ws.rs) — the tracked id is a
	 * diagnostic marker and documentation of the correlation invariant.
	 */
	import {
		createSvelteTable,
		FlexRender,
		renderSnippet,
		renderComponent,
	} from '$lib/components/ui/data-table/index.js';
	import {
		getCoreRowModel,
		type ColumnDef,
		type SortingState,
		type VisibilityState,
	} from '@tanstack/table-core';
	import { onDestroy } from 'svelte';
	import ChevronUp from '@lucide/svelte/icons/chevron-up';
	import ChevronDown from '@lucide/svelte/icons/chevron-down';

	import * as Table from '$lib/components/ui/table';
	import { Input } from '$lib/components/ui/input';
	import { Button } from '$lib/components/ui/button';
	import * as Select from '$lib/components/ui/select';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu';

	import { getData } from '$lib/store/data.svelte';
	import { sendAction } from '$lib/transport/dispatcher';
	import { onIntersect } from '$lib/actions/viewport';
	import { createRuneVirtualizer } from '$lib/utils/virtualizer.svelte';
	import type { ComponentAction } from '$lib/transport/messages';
	import DataTableActions from './DataTableActions.svelte';
	import {
		dateCellSnippet,
		numberCellSnippet,
		badgeCellSnippet,
	} from './datatable-cells.svelte';

	// -------------------------------------------------------------------------
	// Props
	// -------------------------------------------------------------------------
	type SduiColumn = {
		key: string;
		label: string;
		sortable?: boolean;
		kind?: 'text' | 'badge' | 'actions' | 'date' | 'number';
		hidden_default?: boolean;
	};

	type FilterDef =
		| { id: string; kind: 'text'; label: string; placeholder?: string; span?: number }
		| {
				id: string;
				kind: 'select';
				label: string;
				options: { value: string; label: string }[];
				span?: number;
		  }
		| { id: string; kind: 'date-range'; label: string; span?: number };

	let {
		props = {},
		bind,
		action,
		surface,
	}: {
		props: Record<string, unknown>;
		bind?: string;
		action?: ComponentAction;
		surface: string;
	} = $props();

	// -------------------------------------------------------------------------
	// Derived prop accessors
	// -------------------------------------------------------------------------
	const sduiColumns = $derived((props.columns as SduiColumn[]) ?? []);
	const filterDefs = $derived((props.filters as FilterDef[]) ?? []);
	const totalRows = $derived((props.total_rows as number) ?? 0);
	const pageSize = $derived((props.page_size as number) ?? 50);
	const rowIdKey = $derived((props.row_id_key as string) ?? 'id');
	// `source` may be absent (Plan 13-06 CRM migration adds it via the backend
	// builder). When empty, the sentinel callback no-ops gracefully.
	const source = $derived((props.source as string) ?? '');

	// -------------------------------------------------------------------------
	// Bound collection → rows
	// -------------------------------------------------------------------------
	const rawData = $derived(
		bind
			? ((getData(surface, bind) as Record<string, Record<string, unknown>>) ?? {})
			: {},
	);
	const rowEntries = $derived(Object.entries(rawData));
	const rows = $derived(rowEntries.map(([, v]) => v));

	// -------------------------------------------------------------------------
	// Filter state (local, NOT /bind round-trip per D-C4)
	// -------------------------------------------------------------------------
	let filterValues = $state<Record<string, unknown>>({});
	let debounceTimer: ReturnType<typeof setTimeout> | undefined;

	// -------------------------------------------------------------------------
	// Sentinel / fetch state (declared BEFORE flushFilter so the reset path
	// can clear them without a forward reference).
	// -------------------------------------------------------------------------
	let scrollContainer: HTMLDivElement | undefined = $state();
	let fetching = $state(false);
	let exhausted = $state(false);
	/** D-H3 diagnostic marker — the UUID of the last fetch-rows dispatch. */
	let lastFetchRowsActionId: string | null = $state(null);
	/** Row count observed at the moment we dispatched a fetch-rows. Used to
	 *  detect "response landed" (row count grew) and to compute the delta. */
	let prevRowCount = $state(0);
	/** The `limit` we asked for on the most recent in-flight fetch-rows. */
	let expectedLimit = $state(0);

	function resetScrollAndSentinel(): void {
		if (scrollContainer) scrollContainer.scrollTop = 0;
		fetching = false;
		exhausted = false;
		lastFetchRowsActionId = null;
		prevRowCount = rows.length;
	}

	function flushFilter(): void {
		if (debounceTimer !== undefined) {
			clearTimeout(debounceTimer);
			debounceTimer = undefined;
		}
		const payload: Record<string, unknown> = {};
		for (const [k, v] of Object.entries(filterValues)) {
			if (v === undefined || v === null) continue;
			if (typeof v === 'string' && v === '') continue;
			if (typeof v === 'object' && v !== null && ('from' in v || 'to' in v)) {
				const fr = v as { from?: string; to?: string };
				if (!fr.from && !fr.to) continue;
			}
			payload[k] = v;
		}
		resetScrollAndSentinel();
		sendAction('filter', payload);
	}

	function scheduleFilter(delay = 300): void {
		if (debounceTimer !== undefined) clearTimeout(debounceTimer);
		debounceTimer = setTimeout(() => {
			debounceTimer = undefined;
			flushFilter();
		}, delay);
	}

	function handleTextChange(id: string, value: string): void {
		filterValues[id] = value;
		scheduleFilter(300);
	}

	function handleSelectChange(id: string, value: string): void {
		filterValues[id] = value;
		flushFilter();
	}

	function handleTextKeydown(e: KeyboardEvent): void {
		if (e.key === 'Enter') {
			e.preventDefault();
			flushFilter();
		}
	}

	function handleDateRangeChange(id: string, side: 'from' | 'to', value: string): void {
		const existing =
			(filterValues[id] as { from?: string; to?: string } | undefined) ?? {};
		filterValues[id] = { ...existing, [side]: value };
		scheduleFilter(300);
	}

	// Clean up timer on unmount.
	onDestroy(() => {
		if (debounceTimer !== undefined) clearTimeout(debounceTimer);
	});

	// -------------------------------------------------------------------------
	// TanStack column definitions (derived from SDUI columns + cell kinds)
	// -------------------------------------------------------------------------
	const columnDefs = $derived<ColumnDef<Record<string, unknown>>[]>(
		sduiColumns.map((c) => ({
			id: c.key,
			accessorKey: c.key,
			header: c.label,
			enableSorting: c.sortable ?? false,
			enableHiding: true,
			cell: (info) => {
				const value = info.row.original[c.key];
				switch (c.kind ?? 'text') {
					case 'actions': {
						const items =
							(value as Array<{ label: string; action: ComponentAction }>) ?? [];
						return renderComponent(DataTableActions, { items });
					}
					case 'date':
						return renderSnippet(dateCellSnippet, { iso: String(value ?? '') });
					case 'number':
						return renderSnippet(numberCellSnippet, {
							value: Number(value ?? 0),
						});
					case 'badge':
						return renderSnippet(badgeCellSnippet, {
							label: String(value ?? ''),
							variant: 'default',
						});
					case 'text':
					default:
						return String(value ?? '');
				}
			},
		})),
	);

	// -------------------------------------------------------------------------
	// TanStack state (sort + column visibility)
	// -------------------------------------------------------------------------
	let sorting = $state<SortingState>([]);
	let columnVisibility = $state<VisibilityState>({});

	// Initialise hidden-by-default columns from props. We guard with a set of
	// already-initialised column keys so later toggles by the user aren't
	// stomped on re-derives of sduiColumns.
	const initialisedKeys = new Set<string>();
	$effect(() => {
		for (const c of sduiColumns) {
			if (initialisedKeys.has(c.key)) continue;
			initialisedKeys.add(c.key);
			if (c.hidden_default === true) {
				columnVisibility[c.key] = false;
			}
		}
	});

	const table = createSvelteTable<Record<string, unknown>>({
		get data() {
			return rows;
		},
		get columns() {
			return columnDefs;
		},
		state: {
			get sorting() {
				return sorting;
			},
			get columnVisibility() {
				return columnVisibility;
			},
		},
		onSortingChange: (updater) => {
			const next =
				typeof updater === 'function' ? updater(sorting) : updater;
			sorting = next;
			const primary = next[0];
			if (primary) {
				resetScrollAndSentinel();
				sendAction('sort', {
					column: primary.id,
					direction: primary.desc ? 'desc' : 'asc',
				});
			}
		},
		onColumnVisibilityChange: (updater) => {
			columnVisibility =
				typeof updater === 'function' ? updater(columnVisibility) : updater;
		},
		manualSorting: true,
		getCoreRowModel: getCoreRowModel(),
	});

	// -------------------------------------------------------------------------
	// Virtualizer (virtual-core-direct per Plan 13-01's decision)
	// -------------------------------------------------------------------------
	const virtualizer = createRuneVirtualizer<HTMLDivElement, HTMLTableRowElement>(
		() => ({
			count: rows.length,
			getScrollElement: () => scrollContainer ?? null,
			estimateSize: () => 48,
			overscan: 8,
		}),
	);

	$effect(() => {
		if (scrollContainer) {
			virtualizer.mount();
			// Re-push latest options (count/etc.) into the live instance. We
			// cannot pass a thunk to setOptions; reach for the concrete values.
			virtualizer.setOptions({
				count: rows.length,
				overscan: 8,
				estimateSize: () => 48,
			});
		}
	});

	onDestroy(() => virtualizer.destroy());

	// -------------------------------------------------------------------------
	// Sentinel → fetch-rows
	// -------------------------------------------------------------------------
	function isEndOfData(): boolean {
		if (exhausted) return true;
		if (totalRows > 0 && rows.length >= totalRows) return true;
		return false;
	}

	function handleSentinelEnter(): void {
		if (fetching || isEndOfData() || !source) return;
		fetching = true;
		const offset = rows.length;
		const limit = pageSize;
		prevRowCount = rows.length;
		expectedLimit = limit;
		const id = sendAction('fetch-rows', { source, offset, limit });
		lastFetchRowsActionId = id;
	}

	// When new rows land (row count grew since the last dispatch), clear the
	// fetching flag. If the delta is smaller than the requested limit AND
	// total_rows isn't in play, mark the list exhausted — this is the D-D3
	// fewer-than-limit fallback contract.
	$effect(() => {
		const count = rows.length;
		if (fetching && count > prevRowCount) {
			const delta = count - prevRowCount;
			fetching = false;
			if (expectedLimit > 0 && delta < expectedLimit && totalRows === 0) {
				exhausted = true;
			}
			prevRowCount = count;
		}
	});

	// -------------------------------------------------------------------------
	// Row click (preserved from v1)
	// -------------------------------------------------------------------------
	function handleRowClick(row: Record<string, unknown>): void {
		if (action) {
			const id = String(row[rowIdKey] ?? '');
			sendAction(action.name ?? 'select-row', { id }, action.target);
		}
	}
</script>

<div class="flex flex-col gap-4 h-full">
	<!-- Top region: filter bar + column visibility -->
	<div class="flex items-center gap-2 flex-wrap">
		{#each filterDefs as f (f.id)}
			{#if f.kind === 'text'}
				<Input
					class="max-w-sm"
					placeholder={f.placeholder ?? f.label}
					aria-label={f.label}
					value={String(filterValues[f.id] ?? '')}
					oninput={(e) =>
						handleTextChange(f.id, (e.currentTarget as HTMLInputElement).value)}
					onkeydown={handleTextKeydown}
				/>
			{:else if f.kind === 'select'}
				<Select.Root
					type="single"
					value={String(filterValues[f.id] ?? '')}
					onValueChange={(v) => handleSelectChange(f.id, v ?? '')}
				>
					<Select.Trigger class="w-[180px]" aria-label={f.label}>
						{f.label}
					</Select.Trigger>
					<Select.Content>
						{#each f.options as opt (opt.value)}
							<Select.Item value={opt.value}>{opt.label}</Select.Item>
						{/each}
					</Select.Content>
				</Select.Root>
			{:else if f.kind === 'date-range'}
				<Input
					type="date"
					aria-label={`${f.label} from`}
					value={(filterValues[f.id] as { from?: string } | undefined)?.from ?? ''}
					oninput={(e) =>
						handleDateRangeChange(
							f.id,
							'from',
							(e.currentTarget as HTMLInputElement).value,
						)}
					onkeydown={handleTextKeydown}
				/>
				<Input
					type="date"
					aria-label={`${f.label} to`}
					value={(filterValues[f.id] as { to?: string } | undefined)?.to ?? ''}
					oninput={(e) =>
						handleDateRangeChange(
							f.id,
							'to',
							(e.currentTarget as HTMLInputElement).value,
						)}
					onkeydown={handleTextKeydown}
				/>
			{/if}
		{/each}

		<!-- Columns visibility dropdown -->
		<DropdownMenu.Root>
			<DropdownMenu.Trigger>
				{#snippet child({ props: trigProps })}
					<Button {...trigProps} variant="outline" class="ms-auto">Columns</Button>
				{/snippet}
			</DropdownMenu.Trigger>
			<DropdownMenu.Content align="end">
				{#each table.getAllColumns().filter((c) => c.getCanHide()) as column (column.id)}
					<DropdownMenu.CheckboxItem
						class="capitalize"
						checked={column.getIsVisible()}
						onCheckedChange={(v) => column.toggleVisibility(!!v)}
					>
						{column.id}
					</DropdownMenu.CheckboxItem>
				{/each}
			</DropdownMenu.Content>
		</DropdownMenu.Root>
	</div>

	<!-- Virtualised scroll container.
	     Inline `overflow-y: auto` + `height: 400px` + `flex: none` so the
	     layout survives even when Tailwind classes aren't in scope (e.g.
	     standalone browser-tests). Production callers wrap DataTable in a
	     Container with its own height — the 400px default is a sane
	     fallback that keeps the viewport scrollable. -->
	<div
		bind:this={scrollContainer}
		data-testid="datatable-scroll"
		class="border rounded-md"
		style="height: 400px; overflow-y: auto; flex: none; min-height: 0;"
	>
		<!--
		   Layout note: we use a single <table> for the header and a
		   div-based grid for the virtualised body. HTML <tbody> ignores
		   explicit CSS `height` when nested inside an anonymous table
		   wrapper, which prevents the scroll container from ever being
		   scrollable; switching to a div mimicking tbody respects the
		   virtualiser's total size and gives the IntersectionObserver a
		   real overflow to anchor against.
		-->
		<table
			data-testid="datatable-inner"
			class="w-full caption-bottom text-sm"
			data-slot="table"
		>
			<thead class="[&_tr]:border-b" data-slot="table-header">
				{#each table.getHeaderGroups() as headerGroup (headerGroup.id)}
					<tr
						class="hover:bg-muted/50 data-[state=selected]:bg-muted border-b transition-colors"
						data-slot="table-row"
					>
						{#each headerGroup.headers as header (header.id)}
							<th
								role="columnheader"
								data-slot="table-head"
								class="text-foreground h-10 px-2 text-left align-middle font-medium whitespace-nowrap flex-1 {header.column.getCanSort()
									? 'cursor-pointer hover:bg-accent select-none'
									: ''}"
								onclick={() =>
									header.column.getCanSort() && header.column.toggleSorting()}
							>
								<FlexRender
									content={header.column.columnDef.header}
									context={header.getContext()}
								/>
								{#if header.column.getIsSorted() === 'asc'}
									<ChevronUp class="size-4 inline ms-1" />
								{:else if header.column.getIsSorted() === 'desc'}
									<ChevronDown class="size-4 inline ms-1" />
								{/if}
							</th>
						{/each}
					</tr>
				{/each}
			</thead>
		</table>
		<div
			role="rowgroup"
			data-testid="datatable-body"
			style="position: relative; height: {virtualizer.totalSize}px; width: 100%;"
		>
			{#each virtualizer.virtualItems as vi (vi.key)}
				{@const row = table.getRowModel().rows[vi.index]}
				{#if row}
					<!-- svelte-ignore a11y_click_events_have_key_events -->
					<!-- svelte-ignore a11y_interactive_supports_focus -->
					<!-- svelte-ignore a11y_no_static_element_interactions -->
					<div
						role="row"
						tabindex="-1"
						data-index={vi.index}
						data-slot="table-row"
						style="position: absolute; top: 0; left: 0; width: 100%; height: {vi.size}px; transform: translateY({vi.start}px); display: flex;"
						class="hover:bg-muted/50 border-b {action ? 'cursor-pointer' : ''}"
						onclick={() => handleRowClick(row.original)}
					>
						{#each row.getVisibleCells() as cell (cell.id)}
							<div
								role="cell"
								data-slot="table-cell"
								class="px-4 py-3 text-sm align-middle flex-1"
							>
								<FlexRender
									content={cell.column.columnDef.cell}
									context={cell.getContext()}
								/>
							</div>
						{/each}
					</div>
				{/if}
			{/each}

			{#if !isEndOfData() && source && scrollContainer}
				<div
					style="position: absolute; bottom: 0; left: 0; height: 1px; width: 100%;"
					use:onIntersect={{
						onEnter: handleSentinelEnter,
						root: scrollContainer,
						rootMargin: '200px',
						enabled: !fetching,
					}}
				></div>
			{/if}
		</div>
	</div>
</div>
