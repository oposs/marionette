// WebSocket transport with reconnection and exponential backoff

import { applyPatch } from '$lib/store/data.svelte';

let socket: WebSocket | null = $state(null);
let connected = $state(false);
let reconnectDelay = 1000;
let reconnectTimer: ReturnType<typeof setTimeout> | null = null;
const MAX_DELAY = 30000;
const JITTER_FACTOR = 0.2;
let onMessageCallback: ((msg: unknown) => void) | null = null;
let currentUrl: string | null = null;

/**
 * Push the current connection state into /system/connectionStatus on the
 * `main` surface so AppShell's footer connection-status indicator (bound
 * to that data path) reactively reflects it. This is the migrated role of
 * the retired ConnectionBanner component (Phase 12 D-B6).
 *
 * Uses applyPatch with a single tagged Set op, mirroring the wire protocol's
 * data patch format. Safe to call before Render of main — `setAtPointer`
 * creates missing parent objects on the way down.
 *
 * @param state - "connected" | "reconnecting" | "offline"
 */
function publishConnectionStatus(state: 'connected' | 'reconnecting' | 'offline'): void {
  try {
    applyPatch('main', [
      { op: 'set', path: '/system/connectionStatus', value: state },
    ]);
  } catch (err) {
    // Store not initialized yet (happens in unit tests). Not fatal —
    // the first real Render will seed the path from the server.
    // eslint-disable-next-line no-console
    console.debug('publishConnectionStatus: store not ready', err);
  }
}

export function connect(url: string, onMessage: (msg: unknown) => void): void {
  currentUrl = url;
  onMessageCallback = onMessage;
  doConnect(url);
}

function doConnect(url: string): void {
  socket = new WebSocket(url);

  socket.onopen = () => {
    connected = true;
    reconnectDelay = 1000;
    publishConnectionStatus('connected');
    // Send hello
    send({ type: 'hello', version: '1.0.0' });
  };

  socket.onmessage = (event: MessageEvent) => {
    const msg = JSON.parse(event.data as string);
    onMessageCallback?.(msg);
  };

  socket.onclose = () => {
    connected = false;
    socket = null;
    // If we have a URL we will reconnect — surface "reconnecting".
    // If currentUrl is null (explicit disconnect) — surface "offline".
    publishConnectionStatus(currentUrl ? 'reconnecting' : 'offline');
    if (currentUrl) scheduleReconnect();
  };

  socket.onerror = () => {
    // onerror is always followed by onclose, so reconnect happens via onclose
  };
}

function scheduleReconnect(): void {
  const jitter = reconnectDelay * JITTER_FACTOR * (Math.random() * 2 - 1);
  const delay = Math.min(reconnectDelay + jitter, MAX_DELAY);
  reconnectTimer = setTimeout(() => {
    if (currentUrl) doConnect(currentUrl);
  }, delay);
  reconnectDelay = Math.min(reconnectDelay * 2, MAX_DELAY);
}

export function send(msg: unknown): void {
  if (socket?.readyState === WebSocket.OPEN) {
    socket.send(JSON.stringify(msg));
  }
}

export function disconnect(): void {
  currentUrl = null;
  if (reconnectTimer) {
    clearTimeout(reconnectTimer);
    reconnectTimer = null;
  }
  if (socket) {
    socket.close();
    socket = null;
  }
  connected = false;
  publishConnectionStatus('offline');
}

export function isConnected(): boolean {
  return connected;
}
