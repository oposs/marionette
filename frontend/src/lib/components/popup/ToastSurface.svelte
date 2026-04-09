<script lang="ts">
	import { fly } from 'svelte/transition';

	interface ToastItem {
		id: string;
		severity: string;
		message: string;
		duration: number;
	}

	let toasts: ToastItem[] = $state([]);

	const severityClass: Record<string, string> = {
		success: 'border-green-500/30 bg-green-50 text-green-800',
		error: 'border-destructive/30 bg-destructive/10 text-destructive',
		warning: 'border-yellow-500/30 bg-yellow-50 text-yellow-800',
		info: 'border-border bg-background text-foreground',
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

<div class="fixed top-4 left-4 right-4 md:left-auto z-[60] flex flex-col gap-2 md:max-w-sm">
	{#each toasts as toast (toast.id)}
		<div transition:fly={{ x: 100, duration: 200 }}>
			<div class="flex items-center justify-between rounded-md border p-4 shadow-md {severityClass[toast.severity] ?? severityClass.info}">
				<span class="text-sm">{toast.message}</span>
				<button
					class="ml-4 text-current opacity-50 hover:opacity-100"
					onclick={() => removeToast(toast.id)}
					aria-label="Dismiss"
				>&times;</button>
			</div>
		</div>
	{/each}
</div>
