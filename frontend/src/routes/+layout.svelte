<script>
	import '../app.css';
	import { Surface } from '$lib';
	import ModalSurface from '$lib/components/popup/ModalSurface.svelte';
	let { children } = $props();
</script>

<Surface name="main" />
<ModalSurface />
{@render children()}
