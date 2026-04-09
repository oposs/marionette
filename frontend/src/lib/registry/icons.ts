import type { Component } from 'svelte';
import CircleHelp from '@lucide/svelte/icons/circle-help';
import Plus from '@lucide/svelte/icons/plus';
import ChevronUp from '@lucide/svelte/icons/chevron-up';
import ChevronDown from '@lucide/svelte/icons/chevron-down';
import AlertCircle from '@lucide/svelte/icons/alert-circle';
import X from '@lucide/svelte/icons/x';
import Menu from '@lucide/svelte/icons/menu';
import ArrowLeft from '@lucide/svelte/icons/arrow-left';
import Search from '@lucide/svelte/icons/search';
import Filter from '@lucide/svelte/icons/filter';
import Pencil from '@lucide/svelte/icons/pencil';
import Trash2 from '@lucide/svelte/icons/trash-2';
import Check from '@lucide/svelte/icons/check';
import Loader2 from '@lucide/svelte/icons/loader-2';

const ICON_REGISTRY: Record<string, Component> = {};

export function registerIcon(name: string, component: Component): void {
	ICON_REGISTRY[name] = component;
}

export function getIcon(name: string): Component {
	return ICON_REGISTRY[name] ?? CircleHelp;
}

// Register default icons used by the CRM demo
const defaults: [string, Component][] = [
	['plus', Plus],
	['chevron-up', ChevronUp],
	['chevron-down', ChevronDown],
	['alert-circle', AlertCircle],
	['x', X],
	['menu', Menu],
	['arrow-left', ArrowLeft],
	['search', Search],
	['filter', Filter],
	['pencil', Pencil],
	['trash', Trash2],
	['check', Check],
	['loader', Loader2],
	['circle-help', CircleHelp],
];

for (const [name, component] of defaults) {
	registerIcon(name, component);
}
