/**
 * Component registry: maps type strings to Svelte components.
 *
 * The backend sends component nodes with a `type` field. The registry
 * resolves that type string to the corresponding Svelte component class
 * so the NodeRenderer can instantiate it.
 */
import type { Component } from 'svelte';

const registry = new Map<string, Component>();

/** Register a single component type. */
export function register(type: string, component: Component): void {
	registry.set(type, component);
}

/** Look up a component by its type string. */
export function getComponent(type: string): Component | undefined {
	return registry.get(type);
}

/** Register multiple component types at once. */
export function registerAll(components: Record<string, Component>): void {
	for (const [type, component] of Object.entries(components)) {
		registry.set(type, component);
	}
}

/** Clear all registered components (for testing). */
export function clearRegistry(): void {
	registry.clear();
}
