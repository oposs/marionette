<script lang="ts">
	import ExclamationCircleOutline from 'flowbite-svelte-icons/ExclamationCircleOutline.svelte';

	import type { Snippet } from 'svelte';

	let { children }: { children: Snippet } = $props();
</script>

<svelte:boundary>
	{@render children()}

	{#snippet failed(error, reset)}
		<div class="border border-orange-400 bg-orange-50 p-4">
			<div class="flex items-center gap-2">
				<ExclamationCircleOutline class="w-5 h-5 text-orange-500" />
				<p class="text-orange-700 text-sm">Something went wrong rendering this component.</p>
			</div>
			<p class="text-orange-500 text-xs mt-1">Try refreshing the page.</p>
		</div>
	{/snippet}
</svelte:boundary>
