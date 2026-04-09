<script lang="ts">
	import { isConnected } from '$lib/transport/websocket.svelte';
	import Loader2 from '@lucide/svelte/icons/loader-2';
</script>

{#if !isConnected()}
	<div class="bg-destructive text-primary-foreground text-sm font-semibold px-4 py-2 flex items-center justify-center gap-2 fixed top-0 left-0 right-0 z-50">
		<Loader2 class="size-4 animate-spin" />
		<span>Connection lost. Reconnecting...</span>
	</div>
{/if}
