<script lang="ts">
	import { fly } from 'svelte/transition';
	import X from '@lucide/svelte/icons/x';
	import { getToasts, removeToast } from '$lib/store/toasts.svelte';

	// Toast state lives in the module-level store so that the dispatcher (and
	// any other non-component code) can call `addToast` via a plain import.
	// An instance-level `export function` inside a .svelte <script> is not
	// reachable as a module-scope symbol in Svelte 5.
	let toasts = $derived(getToasts());

	const severityClass: Record<string, string> = {
		success: 'border-primary/30 bg-primary/10 text-foreground',
		error: 'border-destructive/30 bg-destructive/10 text-destructive',
		warning: 'border-yellow-500/30 bg-yellow-950/10 text-foreground dark:bg-yellow-950 dark:text-foreground',
		info: 'border-border bg-card text-foreground',
	};
</script>

<div class="fixed bottom-4 right-4 z-[60] flex flex-col gap-2 max-w-sm">
	{#each toasts as toast (toast.id)}
		<div transition:fly={{ x: 100, duration: 200 }}>
			<div class="flex items-center justify-between rounded-md border p-4 shadow-lg {severityClass[toast.severity] ?? severityClass.info}">
				<span class="text-sm">{toast.message}</span>
				<button
					class="ml-4 text-current opacity-50 hover:opacity-100"
					onclick={() => removeToast(toast.id)}
					aria-label="Dismiss"
				><X class="size-4" /></button>
			</div>
		</div>
	{/each}
</div>
