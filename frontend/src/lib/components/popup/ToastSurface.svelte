<script lang="ts">
	import { Toast } from 'flowbite-svelte';
	import { fly } from 'svelte/transition';
	import type { ToastProps } from 'flowbite-svelte';

	interface ToastItem {
		id: string;
		severity: string;
		message: string;
		duration: number;
	}

	let toasts: ToastItem[] = $state([]);

	const severityToColor: Record<string, ToastProps['color']> = {
		success: 'green',
		error: 'red',
		warning: 'yellow',
		info: 'blue',
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

<div class="fixed top-4 right-4 z-[60] flex flex-col gap-2" style="max-width: 384px;">
	{#each toasts as toast (toast.id)}
		<div transition:fly={{ x: 100, duration: 200 }}>
			<Toast
				color={severityToColor[toast.severity] ?? 'blue'}
				dismissable
				onclick={() => removeToast(toast.id)}
			>
				{toast.message}
			</Toast>
		</div>
	{/each}
</div>
