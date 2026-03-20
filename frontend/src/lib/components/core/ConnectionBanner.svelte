<script lang="ts">
	import { isConnected } from '$lib/transport/websocket.svelte';
	import Spinner from 'flowbite-svelte/Spinner.svelte';
</script>

{#if !isConnected()}
	<div class="bg-yellow-50 text-yellow-800 text-sm px-4 py-2 flex items-center gap-2 fixed top-0 left-0 right-0 z-50">
		<Spinner size="4" color="yellow" />
		<span>Connection lost. Reconnecting...</span>
	</div>
{/if}
