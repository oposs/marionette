/**
 * TypeScript interfaces for all Marionette protocol message types.
 * Matches spec/schemas/ definitions exactly.
 */

// --- Supporting types ---

/** Surface identifiers for multi-surface rendering */
export type Surface = 'main' | 'sidebar' | 'modal' | 'toast';

/** JSON Pointer path (RFC 6901) */
export type JsonPointer = string;

/** Message correlation ID */
export type MessageId = string;

// --- Patch operations (discriminated union, matches Rust tagged enum) ---

/** Data op: set a value at a JSON Pointer path. */
export interface PatchOperationSet {
	op: 'set';
	path: string;
	value: unknown;
}

/** Node op: replace (or create) the component at this node ID. */
export interface PatchOperationSetNode {
	op: 'set-node';
	id: string;
	component: ComponentNode;
}

/** Node op: delete the node with this ID. */
export interface PatchOperationDeleteNode {
	op: 'delete-node';
	id: string;
}

/** Node op: replace the children array of the given node. */
export interface PatchOperationSetChildren {
	op: 'set-children';
	id: string;
	children: string[];
}

/** Node op: insert an existing child ID into a parent's children array at index. */
export interface PatchOperationInsertChild {
	op: 'insert-child';
	parent: string;
	index: number;
	childId: string;
}

/** Node op: remove a child ID from a parent's children array. */
export interface PatchOperationRemoveChild {
	op: 'remove-child';
	parent: string;
	childId: string;
}

/** Tagged union of all patch operations. Discriminated by `op`. */
export type PatchOperation =
	| PatchOperationSet
	| PatchOperationSetNode
	| PatchOperationDeleteNode
	| PatchOperationSetChildren
	| PatchOperationInsertChild
	| PatchOperationRemoveChild;

/** Validation error returned by the server */
export interface ValidationError {
	path?: string;
	message: string;
}

/** Action descriptor attached to a component node */
export interface ComponentAction {
	type: string;
	name?: string;
	target?: string;
	idPath?: string;
	/**
	 * Optional UI button variant hint (e.g. 'default' | 'destructive' |
	 * 'outline' | 'ghost' | 'link' | 'secondary'). Purely cosmetic — separate
	 * from `type`, which is a protocol classifier, and from `name`, which is
	 * the backend action identifier.
	 */
	variant?: string;
	[key: string]: unknown;
}

/** A node in the component adjacency list */
export interface ComponentNode {
	type: string;
	props?: Record<string, unknown>;
	children?: string[];
	bind?: string;
	action?: ComponentAction;
	visible?: string;
}

// --- Message types ---

/** Server hello message sent on connection */
export interface HelloMessage {
	type: 'hello';
	version: string;
}

/** Server render message: full UI tree for a surface */
export interface RenderMessage {
	type: 'render';
	id?: string;
	surface: string;
	root: string;
	nodes: Record<string, ComponentNode>;
	data: Record<string, unknown>;
}

/** Server patch message: incremental update to one surface (data and/or tree ops). */
export interface PatchMessage {
	type: 'patch';
	id?: string;
	surface: string;
	patch: PatchOperation[];
}

/** Client action message: user interaction */
export interface ActionMessage {
	type: 'action';
	id?: string;
	name: string;
	source?: string;
	payload?: Record<string, unknown>;
	optimistic?: { patch: PatchOperation[] };
}

/** Server/client event message: lifecycle events */
export interface EventMessage {
	type: 'event';
	id?: string;
	name: string;
	surface?: string;
	hint?: Record<string, unknown>;
}

/** Server error message: validation errors */
export interface ErrorMessage {
	type: 'error';
	id?: string;
	errors: ValidationError[];
}

/** Discriminated union of all protocol message types */
export type ProtocolMessage =
	| HelloMessage
	| RenderMessage
	| PatchMessage
	| ActionMessage
	| EventMessage
	| ErrorMessage;
