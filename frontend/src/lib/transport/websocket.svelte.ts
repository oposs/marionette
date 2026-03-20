// WebSocket transport with reconnection and exponential backoff

let socket: WebSocket | null = $state(null);
let connected = $state(false);
let reconnectDelay = 1000;
let reconnectTimer: ReturnType<typeof setTimeout> | null = null;
const MAX_DELAY = 30000;
const JITTER_FACTOR = 0.2;
let onMessageCallback: ((msg: unknown) => void) | null = null;
let currentUrl: string | null = null;

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
}

export function isConnected(): boolean {
  return connected;
}
