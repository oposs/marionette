<script lang="ts">
	import * as Tooltip from "$lib/components/ui/tooltip/index.js";
	import { cn, type WithElementRef } from "$lib/utils.js";
	import type { HTMLAttributes } from "svelte/elements";
	import {
		SIDEBAR_COOKIE_MAX_AGE,
		SIDEBAR_COOKIE_NAME,
		SIDEBAR_WIDTH,
		SIDEBAR_WIDTH_ICON,
	} from "./constants.js";
	import { setSidebar } from "./context.svelte.js";

	let {
		ref = $bindable(null),
		open = $bindable(true),
		onOpenChange = () => {},
		class: className,
		style,
		children,
		...restProps
	}: WithElementRef<HTMLAttributes<HTMLDivElement>> & {
		open?: boolean;
		onOpenChange?: (open: boolean) => void;
	} = $props();

	const sidebar = setSidebar({
		open: () => open,
		setOpen: (value: boolean) => {
			open = value;
			onOpenChange(value);

			// This sets the cookie to keep the sidebar state.
			document.cookie = `${SIDEBAR_COOKIE_NAME}=${open}; path=/; max-age=${SIDEBAR_COOKIE_MAX_AGE}`;
		},
	});

	// Phase 19 EXER-01 probe hook — see frontend/src/lib/exer01/observe.svelte.ts.
	// Exposes the OUTER Sidebar.Provider's state handle on `window` so the
	// nestability probe can compare it against the inner provider's handle
	// obtained via `getContext` from inside EXER-01's inner shell. Gated on
	// DEV so production builds ship nothing. The first-mount guard ensures
	// the inner provider's setSidebar does NOT clobber the outer handle —
	// Svelte mounts outer before inner, so whichever mounts first is outer.
	if (import.meta.env.DEV && typeof window !== "undefined") {
		const w = window as unknown as { __mrnExer01OuterSidebar?: unknown };
		if (w.__mrnExer01OuterSidebar === undefined) {
			w.__mrnExer01OuterSidebar = sidebar;
		}
	}
</script>

<svelte:window onkeydown={sidebar.handleShortcutKeydown} />

<Tooltip.Provider delayDuration={0}>
	<div
		data-slot="sidebar-wrapper"
		style="--sidebar-width: {SIDEBAR_WIDTH}; --sidebar-width-icon: {SIDEBAR_WIDTH_ICON}; {style}"
		class={cn(
			"group/sidebar-wrapper has-data-[variant=inset]:bg-sidebar flex min-h-svh w-full",
			className
		)}
		bind:this={ref}
		{...restProps}
	>
		{@render children?.()}
	</div>
</Tooltip.Provider>
