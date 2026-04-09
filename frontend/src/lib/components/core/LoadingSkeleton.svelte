<script lang="ts">
	import { Skeleton } from '$lib/components/ui/skeleton';

	let { lines = 3 }: { lines?: number } = $props();

	const widths = [100, 85, 70];
</script>

<div class="flex flex-col gap-2">
	{#each Array(lines) as _, i}
		<Skeleton class="h-4" style="width: {widths[i % widths.length]}%" />
	{/each}
</div>
