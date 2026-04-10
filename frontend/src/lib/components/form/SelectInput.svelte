<script lang="ts">
	import * as Select from '$lib/components/ui/select';
	import { Label } from '$lib/components/ui/label';
	import { getData, setData } from '$lib/store/data.svelte';
	import { markDirty, clearDirty } from '$lib/store/dirty.svelte';
	import type { ComponentAction } from '$lib/transport/messages';
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

	let value = $derived(bind ? ((getData(surface, bind) as string) ?? '') : '');
	let options = $derived(
		(props.options as Array<{ value: string; label: string }>) ?? []
	);

	function handleValueChange(newValue: string) {
		if (bind) {
			setData(surface, bind, newValue);
		}
	}

	function handleOpenChange(open: boolean) {
		if (!bind) return;
		if (open) {
			markDirty(bind);
		} else {
			// Pair mark/clear with open/close (mirrors focus/blur in TextInput)
			// so that dismissing the dropdown without a selection still clears
			// the dirty flag and does not strand pending optimistic state.
			clearDirty(bind, (op) => setData(surface, op.path, op.value));
		}
	}
</script>

<div class="grid w-full gap-2">
	{#if props.label}
		<Label class="font-semibold">{props.label}</Label>
	{/if}
	<Select.Root type="single" value={value} onValueChange={handleValueChange} onOpenChange={handleOpenChange} disabled={props.disabled as boolean}>
		<Select.Trigger class="w-full">
			{#if value && options.find(o => o.value === value)}
				<span>{options.find(o => o.value === value)?.label}</span>
			{:else}
				<span class="text-muted-foreground">{props.placeholder ?? 'Select...'}</span>
			{/if}
		</Select.Trigger>
		<Select.Content>
			{#each options as opt (opt.value)}
				<Select.Item value={opt.value} label={opt.label} />
			{/each}
		</Select.Content>
	</Select.Root>
</div>
