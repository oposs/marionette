<script lang="ts">
	let { props = {}, nodeType = 'unknown', surface = '' }: {
		props?: Record<string, unknown>;
		nodeType?: string;
		surface?: string;
	} = $props();
</script>

{#if import.meta.env.DEV}
	<div class="border-2 border-dashed border-red-500 bg-red-50 p-4">
		<p class="text-red-700 text-sm font-semibold">Unknown component: {nodeType}</p>
		<pre class="text-red-600 font-mono text-xs mt-2">{JSON.stringify(props, null, 2)}</pre>
	</div>
{:else}
	{(() => { console.warn('Unknown component type:', nodeType, 'on surface:', surface); return ''; })()}
{/if}
