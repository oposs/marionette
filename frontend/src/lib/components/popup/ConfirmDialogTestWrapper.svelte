<script lang="ts">
	import * as Dialog from '$lib/components/ui/dialog';
	import ConfirmDialog from './ConfirmDialog.svelte';
	import type { ComponentAction } from '$lib/transport/messages';
	import type { Snippet } from 'svelte';

	let {
		props = {},
		action,
		surface = 'test',
	}: {
		props: Record<string, unknown>;
		action?: ComponentAction;
		surface?: string;
	} = $props();
</script>

<Dialog.Root open={true}>
	<Dialog.Content>
		<ConfirmDialog {props} {action} {surface} />
	</Dialog.Content>
</Dialog.Root>
