/**
 * Tests for D-B6: websocket.svelte.ts must push connection state
 * ('connected' | 'reconnecting' | 'offline') into /system/connectionStatus
 * on every open/close/disconnect transition via applyPatch('main', ...).
 */
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';

// --- Mock applyPatch so we can assert the exact calls ---
const applyPatchMock = vi.fn();
vi.mock('$lib/store/data.svelte', () => ({
	applyPatch: applyPatchMock,
	// Other exports are irrelevant for this test file.
	getStore: vi.fn(),
	setData: vi.fn(),
	getData: vi.fn(),
	setFullState: vi.fn(),
	resetStore: vi.fn(),
}));

// --- Minimal MockWebSocket (same pattern as websocket.svelte.test.ts) ---
type WsEventHandler = ((ev: Event) => void) | null;

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
	onmessage: ((ev: MessageEvent) => void) | null = null;
	onerror: WsEventHandler = null;

	constructor(url: string) {
		this.url = url;
		MockWebSocket.instances.push(this);
	}

	send() {}

	close() {
		this.readyState = MockWebSocket.CLOSED;
		this.onclose?.(new Event('close'));
	}

	simulateOpen() {
		this.readyState = MockWebSocket.OPEN;
		this.onopen?.(new Event('open'));
	}

	simulateClose() {
		this.readyState = MockWebSocket.CLOSED;
		this.onclose?.(new Event('close'));
	}

	static reset() {
		MockWebSocket.instances.length = 0;
	}

	static latest(): MockWebSocket {
		return MockWebSocket.instances[MockWebSocket.instances.length - 1];
	}
}

vi.stubGlobal('WebSocket', MockWebSocket);

describe('WebSocket connection-status publishing (D-B6)', () => {
	let ws: typeof import('./websocket.svelte');

	beforeEach(async () => {
		vi.useFakeTimers();
		MockWebSocket.reset();
		applyPatchMock.mockClear();
		// Re-import the module so the local `currentUrl` / `connected`
		// state starts fresh for each test.
		vi.resetModules();
		ws = await import('./websocket.svelte');
	});

	afterEach(() => {
		vi.useRealTimers();
	});

	it('onopen publishes {op:set, path:/system/connectionStatus, value:"connected"} to main', () => {
		ws.connect('ws://test/ws', vi.fn());
		// Clear any startup noise (there shouldn't be any before onopen)
		applyPatchMock.mockClear();

		MockWebSocket.latest().simulateOpen();

		expect(applyPatchMock).toHaveBeenCalledWith('main', [
			{ op: 'set', path: '/system/connectionStatus', value: 'connected' },
		]);
	});

	it('onclose with currentUrl set publishes "reconnecting"', () => {
		ws.connect('ws://test/ws', vi.fn());
		MockWebSocket.latest().simulateOpen();
		applyPatchMock.mockClear();

		MockWebSocket.latest().simulateClose();

		expect(applyPatchMock).toHaveBeenCalledWith('main', [
			{ op: 'set', path: '/system/connectionStatus', value: 'reconnecting' },
		]);
	});

	it('disconnect() publishes "offline"', () => {
		ws.connect('ws://test/ws', vi.fn());
		MockWebSocket.latest().simulateOpen();
		applyPatchMock.mockClear();

		ws.disconnect();

		// disconnect() clears currentUrl first, then closes the socket (which
		// triggers onclose and publishes 'offline' because currentUrl is null),
		// then publishes 'offline' again directly. Both calls carry 'offline'.
		const offlineCalls = applyPatchMock.mock.calls.filter(
			([surface, ops]) =>
				surface === 'main' &&
				Array.isArray(ops) &&
				ops.length === 1 &&
				ops[0].op === 'set' &&
				ops[0].path === '/system/connectionStatus' &&
				ops[0].value === 'offline'
		);
		expect(offlineCalls.length).toBeGreaterThanOrEqual(1);
		// And no non-offline emissions happened during disconnect.
		const nonOfflineCalls = applyPatchMock.mock.calls.filter(
			([, ops]) =>
				Array.isArray(ops) &&
				ops.length === 1 &&
				ops[0].op === 'set' &&
				ops[0].path === '/system/connectionStatus' &&
				ops[0].value !== 'offline'
		);
		expect(nonOfflineCalls).toHaveLength(0);
	});

	it('a full open→close→reconnect→open cycle emits connected → reconnecting → connected', () => {
		ws.connect('ws://test/ws', vi.fn());
		MockWebSocket.latest().simulateOpen();
		MockWebSocket.latest().simulateClose();

		// Advance past the first reconnect delay
		vi.advanceTimersByTime(1500);
		MockWebSocket.latest().simulateOpen();

		const statusCalls = applyPatchMock.mock.calls
			.filter(
				([, ops]) =>
					Array.isArray(ops) &&
					ops.length === 1 &&
					ops[0].op === 'set' &&
					ops[0].path === '/system/connectionStatus'
			)
			.map(([, ops]) => (ops as Array<{ value: string }>)[0].value);

		expect(statusCalls).toEqual(['connected', 'reconnecting', 'connected']);
	});

	it('publishConnectionStatus swallows applyPatch errors (store not ready)', () => {
		// Force applyPatch to throw on the next call
		applyPatchMock.mockImplementationOnce(() => {
			throw new Error('store not initialized');
		});
		// This must not throw
		expect(() => {
			ws.connect('ws://test/ws', vi.fn());
			MockWebSocket.latest().simulateOpen();
		}).not.toThrow();
	});
});
