<script lang="ts">
	import { fly } from 'svelte/transition';
	import X from '@lucide/svelte/icons/x';

	interface ToastItem {
		id: string;
		severity: string;
		message: string;
		duration: number;
	}

	let toasts: ToastItem[] = $state([]);

	const severityClass: Record<string, string> = {
		success: 'border-primary/30 bg-primary/10 text-foreground',
		error: 'border-destructive/30 bg-destructive/10 text-destructive',
		warning: 'border-yellow-500/30 bg-yellow-950/10 text-foreground dark:bg-yellow-950 dark:text-foreground',
		info: 'border-border bg-card text-foreground',
	};

	export function addToast(event: {
		name: string;
		hint?: Record<string, unknown>;
	}): void {
		const id = crypto.randomUUID();
		const severity = (event.hint?.severity as string) ?? 'info';
		const message = (event.hint?.message as string) ?? event.name;
		const duration = (event.hint?.duration as number) ?? 5000;

		toasts.push({ id, severity, message, duration });

		setTimeout(() => removeToast(id), duration);
	}

	function removeToast(id: string): void {
		toasts = toasts.filter((t) => t.id !== id);
	}
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
