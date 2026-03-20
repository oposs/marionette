<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { ComponentAction } from '$lib/transport/messages';
	import { Spinner as FlowbiteSpinner } from 'flowbite-svelte';

	let {
		props = {},
		bind,
		action,
		surface,
		children
	}: {
		props: Record<string, unknown>;
		bind?: string;
		action?: ComponentAction;
		surface: string;
		children?: Snippet;
	} = $props();
</script>

<FlowbiteSpinner
	size={(props.size ?? '8') as '4' | '5' | '6' | '8' | '10' | '12' | '16'}
	color={(props.color ?? 'primary') as 'primary' | 'secondary' | 'gray' | 'red' | 'green' | 'blue' | 'yellow'}
/>
