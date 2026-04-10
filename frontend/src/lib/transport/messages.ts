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

/** A single patch operation: set value at path, or delete if value is null */
export interface PatchOperation {
	path: string;
	value: unknown;
}

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

/** Server patch message: incremental data updates */
export interface PatchMessage {
	type: 'patch';
	id?: string;
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
