<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { ComponentAction } from '$lib/transport/messages';
	import Loader2 from '@lucide/svelte/icons/loader-2';

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

<div class="flex items-center justify-center">
	<Loader2 class="size-6 animate-spin text-primary" />
</div>
