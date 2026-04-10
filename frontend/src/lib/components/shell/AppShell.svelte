<script lang="ts">
	import type { ComponentAction } from '$lib/transport/messages';
	// Scaffold — real implementation in Plan 06.
	// eslint-disable-next-line @typescript-eslint/no-unused-vars
	let _props: {
		props: Record<string, unknown>;
		bind?: string;
		action?: ComponentAction;
		surface: string;
	} = $props();
</script>

<div data-component="app-shell">AppShell scaffold — implemented in Plan 06</div>
