<script lang="ts">
	import Surface from './Surface.svelte';
	import type { ComponentAction } from '$lib/transport/messages';

	// SurfaceMount is a pure redirection to the sub-surface identified by
	// `props.name`. The standard SDUI contract (props/bind/action/surface/children)
	// is accepted so the component is registry-compatible, but only `props.name`
	// is consumed — the rest are deliberately unused and exist only to satisfy
	// the contract.
	// eslint-disable-next-line @typescript-eslint/no-unused-vars
	let { props = {}, bind: _bind, action: _action, surface: _surface }: {
		props: Record<string, unknown>;
		bind?: string;
		action?: ComponentAction;
		surface: string;
	} = $props();

	let name = $derived((props.name as string | undefined) ?? '');
</script>

{#if name}
	<Surface {name} />
{/if}
