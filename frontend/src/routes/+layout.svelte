<script>
	import '../app.css';
	import { Surface } from '$lib';
	let { children } = $props();
</script>

<Surface name="main" />
{@render children()}
