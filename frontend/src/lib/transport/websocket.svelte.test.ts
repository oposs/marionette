import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';

// --- Mock WebSocket ---

type WsEventHandler = ((ev: Event) => void) | null;
type WsMsgHandler = ((ev: MessageEvent) => void) | null;

class MockWebSocket {
  static CONNECTING = 0;
  static OPEN = 1;
  static CLOSING = 2;
  static CLOSED = 3;
  static readonly instances: MockWebSocket[] = [];

  url: string;
  readyState = MockWebSocket.CONNECTING;
  onopen: WsEventHandler = null;
  onclose: WsEventHandler = null;
  onmessage: WsMsgHandler = null;
  onerror: WsEventHandler = null;
  sent: string[] = [];

  constructor(url: string) {
    this.url = url;
    MockWebSocket.instances.push(this);
  }

  send(data: string) {
    this.sent.push(data);
  }

  close() {
    this.readyState = MockWebSocket.CLOSED;
    this.onclose?.(new Event('close'));
  }

  // Test helpers
  simulateOpen() {
    this.readyState = MockWebSocket.OPEN;
    this.onopen?.(new Event('open'));
  }

  simulateMessage(data: unknown) {
    this.onmessage?.({ data: JSON.stringify(data) } as MessageEvent);
  }

  simulateClose() {
    this.readyState = MockWebSocket.CLOSED;
    this.onclose?.(new Event('close'));
  }

  simulateError() {
    this.onerror?.(new Event('error'));
  }

  static reset() {
    MockWebSocket.instances.length = 0;
  }

  static latest(): MockWebSocket {
    return MockWebSocket.instances[MockWebSocket.instances.length - 1];
  }
}

// Stub WebSocket globally
vi.stubGlobal('WebSocket', MockWebSocket);

// We need to import the module AFTER stubbing WebSocket
// Use dynamic import to ensure mock is in place

describe('WebSocket transport', () => {
  let ws: typeof import('./websocket.svelte');

  beforeEach(async () => {
    vi.useFakeTimers();
    MockWebSocket.reset();
    // Re-import to get fresh module state
    vi.resetModules();
    ws = await import('./websocket.svelte');
  });

  afterEach(() => {
    ws.disconnect();
    vi.useRealTimers();
  });

  it('connect(url) creates WebSocket and sets connected = true on open', () => {
    const onMsg = vi.fn();
    ws.connect('ws://test/ws', onMsg);

    expect(MockWebSocket.instances).toHaveLength(1);
    expect(MockWebSocket.latest().url).toBe('ws://test/ws');
    expect(ws.isConnected()).toBe(false);

    MockWebSocket.latest().simulateOpen();
    expect(ws.isConnected()).toBe(true);
  });

  it('sends hello message { type: "hello", version: "1.0.0" } on open', () => {
    const onMsg = vi.fn();
    ws.connect('ws://test/ws', onMsg);
    MockWebSocket.latest().simulateOpen();

    const sent = MockWebSocket.latest().sent;
    expect(sent).toHaveLength(1);
    expect(JSON.parse(sent[0])).toEqual({ type: 'hello', version: '1.0.0' });
  });

  it('onmessage parses JSON and calls provided onMessage callback', () => {
    const onMsg = vi.fn();
    ws.connect('ws://test/ws', onMsg);
    MockWebSocket.latest().simulateOpen();

    const testMsg = { type: 'render', surface: 'main' };
    MockWebSocket.latest().simulateMessage(testMsg);

    expect(onMsg).toHaveBeenCalledWith(testMsg);
  });

  it('send(msg) serializes to JSON and sends via WebSocket', () => {
    const onMsg = vi.fn();
    ws.connect('ws://test/ws', onMsg);
    MockWebSocket.latest().simulateOpen();

    // Clear the hello message
    MockWebSocket.latest().sent.length = 0;

    ws.send({ type: 'action', name: 'test' });
    expect(MockWebSocket.latest().sent).toHaveLength(1);
    expect(JSON.parse(MockWebSocket.latest().sent[0])).toEqual({
      type: 'action',
      name: 'test'
    });
  });

  it('on WebSocket close, connected becomes false and reconnect is scheduled', () => {
    const onMsg = vi.fn();
    ws.connect('ws://test/ws', onMsg);
    MockWebSocket.latest().simulateOpen();
    expect(ws.isConnected()).toBe(true);

    MockWebSocket.latest().simulateClose();
    expect(ws.isConnected()).toBe(false);

    // Advance time to trigger reconnect (initial delay ~1000ms +/- jitter)
    vi.advanceTimersByTime(1500);
    // A new WebSocket should have been created
    expect(MockWebSocket.instances.length).toBeGreaterThanOrEqual(2);
  });

  it('reconnect uses exponential backoff: 1000, 2000, 4000, 8000, 16000, 30000 (capped)', () => {
    const onMsg = vi.fn();
    ws.connect('ws://test/ws', onMsg);
    MockWebSocket.latest().simulateOpen();

    const expectedDelays = [1000, 2000, 4000, 8000, 16000, 30000];

    for (let i = 0; i < expectedDelays.length; i++) {
      const prevCount = MockWebSocket.instances.length;
      MockWebSocket.latest().simulateClose();

      // Advance just past max possible delay (base + 20% jitter)
      const maxDelay = Math.min(expectedDelays[i] * 1.2, 30000);
      vi.advanceTimersByTime(maxDelay + 50);

      expect(MockWebSocket.instances.length).toBe(prevCount + 1);

      // Simulate successful reconnect (so we can trigger next close)
      MockWebSocket.latest().simulateOpen();
    }
  });

  it('successful reconnect resets delay to 1000ms', () => {
    const onMsg = vi.fn();
    ws.connect('ws://test/ws', onMsg);
    MockWebSocket.latest().simulateOpen();

    // Trigger a few reconnects to increase delay
    MockWebSocket.latest().simulateClose();
    vi.advanceTimersByTime(1500);
    MockWebSocket.latest().simulateOpen();

    MockWebSocket.latest().simulateClose();
    vi.advanceTimersByTime(3000);
    MockWebSocket.latest().simulateOpen();

    // Now disconnect and reconnect -- delay should be reset to ~1000ms
    const countBefore = MockWebSocket.instances.length;
    MockWebSocket.latest().simulateClose();

    // Should reconnect within ~1200ms (1000 + 20% jitter)
    vi.advanceTimersByTime(1300);
    expect(MockWebSocket.instances.length).toBe(countBefore + 1);
  });

  it('disconnect() closes WebSocket and cancels pending reconnect', () => {
    const onMsg = vi.fn();
    ws.connect('ws://test/ws', onMsg);
    MockWebSocket.latest().simulateOpen();
    MockWebSocket.latest().simulateClose();

    // Reconnect is pending
    ws.disconnect();

    // Advance time - no new WebSocket should be created
    const count = MockWebSocket.instances.length;
    vi.advanceTimersByTime(5000);
    expect(MockWebSocket.instances.length).toBe(count);
    expect(ws.isConnected()).toBe(false);
  });

  it('isConnected() returns current connection state', () => {
    expect(ws.isConnected()).toBe(false);

    const onMsg = vi.fn();
    ws.connect('ws://test/ws', onMsg);
    expect(ws.isConnected()).toBe(false);

    MockWebSocket.latest().simulateOpen();
    expect(ws.isConnected()).toBe(true);

    MockWebSocket.latest().simulateClose();
    expect(ws.isConnected()).toBe(false);
  });

  it('on WebSocket error, connection closes (triggering reconnect via onclose)', () => {
    const onMsg = vi.fn();
    ws.connect('ws://test/ws', onMsg);
    MockWebSocket.latest().simulateOpen();

    // Simulate error followed by close (which is browser behavior)
    MockWebSocket.latest().simulateError();
    MockWebSocket.latest().simulateClose();

    expect(ws.isConnected()).toBe(false);

    // Reconnect should be scheduled
    vi.advanceTimersByTime(1500);
    expect(MockWebSocket.instances.length).toBeGreaterThanOrEqual(2);
  });
});
