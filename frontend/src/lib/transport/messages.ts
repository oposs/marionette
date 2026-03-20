// Protocol message types - derived from spec/schemas/message.yaml
// These are type stubs matching the protocol schema exactly.

export interface HelloMessage {
  type: 'hello';
  version: string;
}

export interface RenderMessage {
  type: 'render';
  id?: string;
  surface: string;
  root: string;
  nodes: Record<string, ComponentNode>;
  data: Record<string, unknown>;
}

export interface PatchMessage {
  type: 'patch';
  id?: string;
  patch: PatchOperation[];
}

export interface ActionMessage {
  type: 'action';
  id?: string;
  name: string;
  source?: string;
  payload?: Record<string, unknown>;
  optimistic?: { patch: PatchOperation[] };
}

export interface EventMessage {
  type: 'event';
  id?: string;
  name: string;
  surface?: string;
  hint?: Record<string, unknown>;
}

export interface ErrorMessage {
  type: 'error';
  id?: string;
  errors: ValidationError[];
}

export type ProtocolMessage =
  | HelloMessage
  | RenderMessage
  | PatchMessage
  | ActionMessage
  | EventMessage
  | ErrorMessage;

// From spec/schemas/component.yaml
export interface ComponentNode {
  type: string;
  props?: Record<string, unknown>;
  children?: string[];
  bind?: string;
  action?: ComponentAction;
  visible?: string;
}

export interface ComponentAction {
  type: string;
  name?: string;
  target?: string;
  idPath?: string;
  [key: string]: unknown;
}

// From spec/schemas/data.yaml
export interface PatchOperation {
  path: string;
  value: unknown;
}

export interface ValidationError {
  path?: string;
  message: string;
}
