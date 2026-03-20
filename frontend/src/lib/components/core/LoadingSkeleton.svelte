<script lang="ts">
	let { lines = 3 }: { lines?: number } = $props();

	const widths = [100, 85, 70];
</script>

{#each Array(lines) as _, i}
	<div
		class="animate-pulse bg-gray-200 dark:bg-gray-700 rounded-sm h-4 mb-2"
		style="width: {widths[i % widths.length]}%"
	></div>
{/each}
