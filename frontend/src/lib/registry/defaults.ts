/**
 * Default component registrations.
 *
 * Plans 04 and 05 will populate this with built-in components
 * (side-nav, text-input, data-table, etc.). For now it registers
 * an empty set so the init flow works end-to-end.
 */
import { registerAll } from './registry';

export function registerDefaults(): void {
	registerAll({
		// Will be populated as components are built in Plans 04 and 05:
		// 'side-nav': SideNav,
		// 'text-input': TextInput,
		// etc.
	});
}
