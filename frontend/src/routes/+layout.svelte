<script>
	import '../app.css';
	import { ConnectionBanner, Surface } from '$lib';
	let { children } = $props();
</script>

<ConnectionBanner />
<div class="flex h-screen">
	<Surface name="sidebar" />
	<Surface name="main" />
</div>
<Surface name="modal" />
<Surface name="toast" />
{@render children()}
