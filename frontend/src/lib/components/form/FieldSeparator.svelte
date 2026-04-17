<script lang="ts">
	import * as Field from '$lib/components/ui/field';
	import type { ComponentAction } from '$lib/transport/messages';

	// SDUI contract declares all four standard props for NodeRenderer
	// invocation uniformity. FieldSeparator consumes none — it is a thin,
	// prop-less visual divider (D-C2 preferred explicit-node path) placed
	// between sibling FieldSet nodes in the adjacency list.
	let {
		props = {},
		bind,
		action,
		surface,
	}: {
		props: Record<string, unknown>;
		bind?: string;
		action?: ComponentAction;
		surface: string;
	} = $props();
</script>

<Field.Separator />
