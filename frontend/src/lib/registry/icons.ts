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
// Phase 19 Plan 19-01: 17 new icons for EXER-01/02/03 exerciser screens
// (16 from 19-UI-SPEC §Design System + rotate-ccw which is referenced
// at UI-SPEC lines 223/277 as the Reset/Remeasure CTA icon).
import Activity from '@lucide/svelte/icons/activity';
import Focus from '@lucide/svelte/icons/focus';
import Type from '@lucide/svelte/icons/type';
import Languages from '@lucide/svelte/icons/languages';
import MoveHorizontal from '@lucide/svelte/icons/move-horizontal';
import Gauge from '@lucide/svelte/icons/gauge';
import Timer from '@lucide/svelte/icons/timer';
import Cpu from '@lucide/svelte/icons/cpu';
import Zap from '@lucide/svelte/icons/zap';
import LayoutDashboard from '@lucide/svelte/icons/layout-dashboard';
import Layers from '@lucide/svelte/icons/layers';
import TriangleAlert from '@lucide/svelte/icons/triangle-alert';
import CircleCheck from '@lucide/svelte/icons/circle-check';
import CircleX from '@lucide/svelte/icons/circle-x';
import Play from '@lucide/svelte/icons/play';
import Pause from '@lucide/svelte/icons/pause';
import RotateCcw from '@lucide/svelte/icons/rotate-ccw';

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
	// Phase 19 Plan 19-01: exerciser screen icons (17 entries).
	['activity', Activity],
	['focus', Focus],
	['type', Type],
	['languages', Languages],
	['move-horizontal', MoveHorizontal],
	['gauge', Gauge],
	['timer', Timer],
	['cpu', Cpu],
	['zap', Zap],
	['layout-dashboard', LayoutDashboard],
	['layers', Layers],
	['triangle-alert', TriangleAlert],
	['circle-check', CircleCheck],
	['circle-x', CircleX],
	['play', Play],
	['pause', Pause],
	['rotate-ccw', RotateCcw],
];

for (const [name, component] of defaults) {
	registerIcon(name, component);
}
