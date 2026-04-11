import { describe, it, expect, vi, beforeEach } from 'vitest';

// Mock the websocket send function
vi.mock('./websocket.svelte', () => ({
  send: vi.fn()
}));

// Mock the optimistic store
vi.mock('$lib/store/optimistic.svelte', () => ({
  applyOptimistic: vi.fn()
}));

// Mock crypto.randomUUID
vi.stubGlobal('crypto', { randomUUID: vi.fn(() => 'test-uuid-1234') });

import { handleMessage, sendAction, registerHandler, resetHandlers } from './dispatcher';
import { send } from './websocket.svelte';
import { applyOptimistic } from '$lib/store/optimistic.svelte';

describe('Message dispatcher', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    resetHandlers();
  });

  it('handleMessage with type "render" calls the registered render handler', () => {
    const handler = vi.fn();
    registerHandler('render', handler);

    const msg = { type: 'render', surface: 'main', root: 'r1', nodes: {}, data: {} };
    handleMessage(msg);

    expect(handler).toHaveBeenCalledWith(msg);
  });

  it('handleMessage with type "patch" calls the registered patch handler', () => {
    const handler = vi.fn();
    registerHandler('patch', handler);

    const msg = { type: 'patch', patch: [{ path: '/name', value: 'test' }] };
    handleMessage(msg);

    expect(handler).toHaveBeenCalledWith(msg);
  });

  it('handleMessage with type "event" calls the registered event handler', () => {
    const handler = vi.fn();
    registerHandler('event', handler);

    const msg = { type: 'event', name: 'loaded', surface: 'main' };
    handleMessage(msg);

    expect(handler).toHaveBeenCalledWith(msg);
  });

  it('handleMessage with type "error" calls the registered error handler', () => {
    const handler = vi.fn();
    registerHandler('error', handler);

    const msg = { type: 'error', errors: [{ message: 'bad input' }] };
    handleMessage(msg);

    expect(handler).toHaveBeenCalledWith(msg);
  });

  it('handleMessage with unknown type logs warning (does not throw)', () => {
    const warnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});

    expect(() => {
      handleMessage({ type: 'unknown-type' });
    }).not.toThrow();

    expect(warnSpy).toHaveBeenCalledWith('Unhandled message type:', 'unknown-type');
    warnSpy.mockRestore();
  });

  it('sendAction creates ActionMessage with type "action", generates correlation ID, calls send()', () => {
    sendAction('navigate', { path: '/' });

    expect(send).toHaveBeenCalledWith({
      type: 'action',
      id: 'test-uuid-1234',
      name: 'navigate',
      payload: { path: '/' }
    });
  });

  it('sendAction with source includes source field', () => {
    sendAction('click', { id: '5' }, 'button-1');

    expect(send).toHaveBeenCalledWith({
      type: 'action',
      id: 'test-uuid-1234',
      name: 'click',
      payload: { id: '5' },
      source: 'button-1'
    });
  });

  it('sendAction with optimistic patch includes optimistic field and calls applyOptimistic', () => {
    const patch = [{ op: 'set' as const, path: '/name', value: 'new' }];
    sendAction('save', { id: '1' }, undefined, { patch });

    expect(send).toHaveBeenCalledWith({
      type: 'action',
      id: 'test-uuid-1234',
      name: 'save',
      payload: { id: '1' },
      optimistic: { patch }
    });

    expect(applyOptimistic).toHaveBeenCalledWith('test-uuid-1234', expect.anything(), patch);
  });

  // Phase 13 D-H3: sendAction must RETURN the generated id so callers
  // (DataTable) can correlate responses and discard stale ones.
  it('sendAction returns the generated action id', () => {
    const id = sendAction('filter', { search: 'alice' });
    expect(id).toBe('test-uuid-1234');
  });

  it('sendAction returns a non-empty string even without payload', () => {
    const id = sendAction('noop');
    expect(typeof id).toBe('string');
    expect(id.length).toBeGreaterThan(0);
  });

  it('sendAction generates a fresh id per call', () => {
    let counter = 0;
    vi.stubGlobal('crypto', { randomUUID: vi.fn(() => `id-${counter++}`) });
    expect(sendAction('a')).toBe('id-0');
    expect(sendAction('b')).toBe('id-1');
  });
});
