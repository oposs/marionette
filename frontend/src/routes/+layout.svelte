<script>
	import '../app.css';
	import { Surface } from '$lib';
	import ModalSurface from '$lib/components/popup/ModalSurface.svelte';
	import { Toaster } from 'svelte-sonner';
	let { children } = $props();
</script>

<Surface name="main" />
<ModalSurface />
<Toaster richColors closeButton position="bottom-right" />
{@render children()}
