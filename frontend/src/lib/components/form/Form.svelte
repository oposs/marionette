<script lang="ts">
	import * as Field from '$lib/components/ui/field';
	import { getData } from '$lib/store/data.svelte';
	import { sendAction } from '$lib/transport/dispatcher';
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

	let formErrors = $derived(
		bind ? ((getData(surface, '/_errors' + bind) as string[]) ?? []) : []
	);

	function handleSubmit(e: SubmitEvent) {
		e.preventDefault();
		if (!action) return;
		// Phase 15 D-G2 — pass collected form values as the submit payload,
		// not `{}`. When `bind` is set, the Form's bound subtree in the data
		// store already holds the latest values (every leaf Field writes
		// directly to `/bind/<field>` via setData on input). When `bind` is
		// absent the dispatch still fires with an empty object; the
		// handler-side contract is that `action` being set implies the
		// caller wants the dispatch regardless of payload shape.
		const payload =
			bind !== undefined
				? ((getData(surface, bind) as Record<string, unknown> | undefined) ?? {})
				: {};
		sendAction(action.name ?? 'submit', payload, action.target);
	}
</script>

<!--
	Form layout:
	- `max-w-md` caps total form width — fields don't stretch across the
	  whole page.
	- `Field.Group` has `flex w-full flex-col` upstream; adding `items-start`
	  here stops its children from being stretched to full width by the
	  flex default. Field.Field itself sets `w-full`, so actual fields
	  still fill the form; bare children (submit button, etc.) render at
	  their natural inline-flex width.
-->
<form
	onsubmit={handleSubmit}
	class="shrink-0 overflow-y-auto w-full max-w-md"
>
	{#if Array.isArray(formErrors) && formErrors.length > 0}
		<div
			class="bg-destructive/10 border border-destructive/50 text-destructive rounded-md p-4 mb-4"
		>
			{#each formErrors as error}
				<p class="text-sm">{error}</p>
			{/each}
		</div>
	{/if}
	<Field.Group class="space-y-6 items-start">
		{@render children?.()}
	</Field.Group>
</form>
