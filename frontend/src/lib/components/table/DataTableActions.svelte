<script lang="ts" module>
	import type { ComponentAction } from '$lib/transport/messages';

	/**
	 * Per-row action item. Matches the shape CRM handlers already produce
	 * (see `contact.rs:423`, `company.rs:126`, `user.rs:87`). Phase 13 Plan 05's
	 * DataTable rewrite wires `column.kind: 'actions'` to this component via
	 * `renderComponent(DataTableActions, { items: row.original[col.key] })`.
	 */
	export interface ActionItem {
		label: string;
		action: ComponentAction;
	}
</script>

<script lang="ts">
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu';
	import { Button } from '$lib/components/ui/button';
	import EllipsisVertical from '@lucide/svelte/icons/ellipsis-vertical';
	import { sendAction } from '$lib/transport/dispatcher';

	let {
		items = [],
	}: {
		items?: ActionItem[];
	} = $props();

	function handleSelect(item: ActionItem): void {
		const name = item.action.name ?? item.action.type;
		const payload = item.action.payload as Record<string, unknown> | undefined;
		const target = item.action.target;
		sendAction(name, payload, target);
	}
</script>

<DropdownMenu.Root>
	<DropdownMenu.Trigger>
		{#snippet child({ props })}
			<Button {...props} variant="ghost" size="icon" aria-label="Row actions">
				<EllipsisVertical class="size-4" />
			</Button>
		{/snippet}
	</DropdownMenu.Trigger>
	<DropdownMenu.Content align="end">
		{#each items as item (item.label)}
			<DropdownMenu.Item onSelect={() => handleSelect(item)}>
				{item.label}
			</DropdownMenu.Item>
		{/each}
	</DropdownMenu.Content>
</DropdownMenu.Root>
