// Marionette Svelte library -- public API

// Core rendering
export { default as Surface } from './components/core/Surface.svelte';
export { default as NodeRenderer } from './components/core/NodeRenderer.svelte';
export { default as ConnectionBanner } from './components/core/ConnectionBanner.svelte';

// Registry
export { register, getComponent, registerAll } from './registry/registry';

// Store
export { getData, setData, applyPatch, setFullState, resetStore } from './store/data.svelte';
export { markDirty, clearDirty, isDirty } from './store/dirty.svelte';
export { applyOptimistic, confirmOptimistic, rollbackOptimistic } from './store/optimistic.svelte';
export { setSurfaceTree, getSurfaceTree, clearSurfaceTree } from './store/surfaces.svelte';

// Transport
export { sendAction } from './transport/dispatcher';
export { isConnected } from './transport/websocket.svelte';

// Routing
export { updateUrl } from './routing/router.svelte';

// Initialization
export { initMarionette, destroyMarionette } from './init';

// Types
export type {
	ProtocolMessage, RenderMessage, PatchMessage, ActionMessage,
	EventMessage, ErrorMessage, HelloMessage,
	ComponentNode, ComponentAction, PatchOperation, ValidationError,
} from './transport/messages';
