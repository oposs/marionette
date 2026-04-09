<script lang="ts">
	import * as Card from '$lib/components/ui/card';
	import { Separator } from '$lib/components/ui/separator';
	import { Button as ShadcnButton } from '$lib/components/ui/button';
	import ArrowLeft from '@lucide/svelte/icons/arrow-left';
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

	type SectionDef = {
		title?: string;
		fields: string[];
	};

	let title = $derived((props.title as string) ?? '');
	let backAction = $derived(props.back_action as ComponentAction | undefined);
	let sections = $derived((props.sections as SectionDef[]) ?? []);
	let columns = $derived((props.columns as number) ?? 1);
	let nodes = $derived((props.nodes as Record<string, ComponentNode>) ?? {});
	let actions = $derived((props.actions as ComponentAction[]) ?? []);

	function handleBack() {
		if (backAction) {
			sendAction(backAction.name ?? 'go-back', {}, backAction.target);
		}
	}

	function handleAction(act: ComponentAction) {
		const surfaceData = getAllData(surface) ?? {};
		const payload = {
			...(act.payload as Record<string, unknown> ?? {}),
			...surfaceData,
		};
		sendAction(act.name ?? act.type, payload, act.target);
	}

	function handleSubmit(e: SubmitEvent) {
		e.preventDefault();
		if (action) {
			const surfaceData = getAllData(surface) ?? {};
			const payload = {
				...(action.payload as Record<string, unknown> ?? {}),
				...surfaceData,
			};
			sendAction(action.name ?? 'submit', payload, action.target);
		}
	}
</script>

<div class="flex flex-col gap-6">
	<!-- Header -->
	<div class="flex items-center gap-4">
		{#if backAction}
			<ShadcnButton variant="ghost" size="icon" onclick={handleBack} aria-label="Go back">
				<ArrowLeft class="size-5" />
			</ShadcnButton>
		{/if}
		{#if title}
			<h1 class="text-xl font-semibold text-foreground">{title}</h1>
		{/if}
	</div>

	<!-- Form -->
	<form onsubmit={handleSubmit} class="flex flex-col gap-6">
		{#each sections as section, i}
			{#if i > 0}
				<Separator />
			{/if}
			<Card.Root class="p-4">
				{#if section.title}
					<h3 class="text-base font-semibold text-foreground mb-4">{section.title}</h3>
				{/if}
				<div class="grid grid-cols-1 gap-4" style="grid-template-columns: repeat({columns}, 1fr)">
					{#each section.fields as fieldId (fieldId)}
						<NodeRenderer nodeId={fieldId} {nodes} {surface} />
					{/each}
				</div>
			</Card.Root>
		{/each}

		<!-- Action bar -->
		{#if actions.length > 0}
			<div class="flex justify-end gap-2 pt-4 border-t border-border">
				{#each actions as act}
					<ShadcnButton
						variant={act.type === 'destructive' ? 'destructive' : (act.type as import('$lib/components/ui/button').ButtonVariant) || 'default'}
						onclick={() => handleAction(act)}
					>
						{act.name ?? ''}
					</ShadcnButton>
				{/each}
			</div>
		{/if}

		{@render children?.()}
	</form>
</div>
