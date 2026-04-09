<script lang="ts">
	import AlertCircle from '@lucide/svelte/icons/alert-circle';

	import type { Snippet } from 'svelte';

	let { children }: { children: Snippet } = $props();
</script>

<svelte:boundary>
	{@render children()}

	{#snippet failed(error, reset)}
		<div class="border border-destructive/40 bg-destructive/10 p-4 rounded-md">
			<div class="flex items-center gap-2">
				<AlertCircle class="size-5 text-destructive" />
				<p class="text-destructive text-sm">Something went wrong rendering this component.</p>
			</div>
			<p class="text-destructive/70 text-xs mt-1">Try refreshing the page.</p>
		</div>
	{/snippet}
</svelte:boundary>
