<script lang="ts">
	let { props = {}, nodeType = 'unknown', surface = '' }: {
		props?: Record<string, unknown>;
		nodeType?: string;
		surface?: string;
	} = $props();

	// In production, log unknown component types but render nothing. Lifting
	// the side effect into $effect avoids duplicate warnings when Svelte
	// re-evaluates template expressions during reconciliation.
	$effect(() => {
		if (!import.meta.env.DEV) {
			console.warn('Unknown component type:', nodeType, 'on surface:', surface);
		}
	});
</script>

{#if import.meta.env.DEV}
	<div class="border-2 border-dashed border-destructive bg-destructive/10 p-4 rounded-md">
		<p class="text-destructive text-sm font-semibold">Unknown component: {nodeType}</p>
		<pre class="text-destructive font-mono text-xs mt-2">{JSON.stringify(props, null, 2)}</pre>
	</div>
{/if}
