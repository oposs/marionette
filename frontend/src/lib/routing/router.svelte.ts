// URL routing: syncs URL with backend-driven navigation
// Uses dependency injection for sendAction to simplify testing

let currentPath = $state('');
let popstateHandler: ((e: PopStateEvent) => void) | null = null;
let sendActionFn: ((name: string, payload?: Record<string, unknown>) => void) | null = null;

/**
 * Initialize the router. Registers a popstate listener and sends
 * an initial navigate action with the current URL path.
 */
export function initRouter(
  sendAction: (name: string, payload?: Record<string, unknown>) => void
): void {
  sendActionFn = sendAction;
  currentPath = window.location.pathname;

  popstateHandler = () => {
    const path = window.location.pathname;
    currentPath = path;
    sendActionFn?.('navigate', { path });
  };

  window.addEventListener('popstate', popstateHandler);

  // Send initial navigation
  sendActionFn('navigate', { path: currentPath });
}

/**
 * Update the URL to the given path. No-op if path matches current.
 */
export function updateUrl(path: string): void {
  if (path !== currentPath) {
    history.pushState(null, '', path);
    currentPath = path;
  }
}

/**
 * Tear down the router, removing the popstate listener.
 */
export function destroyRouter(): void {
  if (popstateHandler) {
    window.removeEventListener('popstate', popstateHandler);
    popstateHandler = null;
  }
  sendActionFn = null;
  currentPath = '';
}
