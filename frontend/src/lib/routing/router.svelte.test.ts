// @vitest-environment jsdom
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';

describe('URL Router', () => {
  let router: typeof import('./router.svelte');
  let sendActionFn: ReturnType<typeof vi.fn>;

  let originalPushState: typeof history.pushState;

  beforeEach(async () => {
    vi.resetModules();
    sendActionFn = vi.fn();

    // Save and mock history.pushState
    originalPushState = history.pushState;
    history.pushState = vi.fn();

    // Set location pathname for tests
    Object.defineProperty(window, 'location', {
      value: { pathname: '/contacts', href: 'http://localhost/contacts' },
      writable: true,
      configurable: true
    });

    router = await import('./router.svelte');
  });

  afterEach(() => {
    router.destroyRouter();
    history.pushState = originalPushState;
  });

  it('initRouter registers popstate listener', () => {
    const addSpy = vi.spyOn(window, 'addEventListener');
    router.initRouter(sendActionFn);

    expect(addSpy).toHaveBeenCalledWith('popstate', expect.any(Function));
    addSpy.mockRestore();
  });

  it('initRouter sends navigate action with current window.location.pathname', () => {
    router.initRouter(sendActionFn);

    expect(sendActionFn).toHaveBeenCalledWith('navigate', { path: '/contacts' });
  });

  it('updateUrl(path) calls history.pushState with the path', () => {
    router.initRouter(sendActionFn);
    router.updateUrl('/settings');

    expect(history.pushState).toHaveBeenCalledWith(null, '', '/settings');
  });

  it('updateUrl with same path as current does not call pushState (no-op)', () => {
    router.initRouter(sendActionFn);

    // Current path is /contacts
    router.updateUrl('/contacts');

    // pushState should not be called
    expect(history.pushState).not.toHaveBeenCalled();
  });

  it('popstate event sends navigate action to backend via sendAction', () => {
    router.initRouter(sendActionFn);
    sendActionFn.mockClear();

    // Update location for popstate
    (window as unknown as { location: { pathname: string } }).location.pathname = '/settings';

    // Dispatch popstate event
    window.dispatchEvent(new PopStateEvent('popstate'));

    expect(sendActionFn).toHaveBeenCalledWith('navigate', { path: '/settings' });
  });

  it('destroyRouter removes popstate listener', () => {
    const removeSpy = vi.spyOn(window, 'removeEventListener');
    router.initRouter(sendActionFn);
    router.destroyRouter();

    expect(removeSpy).toHaveBeenCalledWith('popstate', expect.any(Function));
    removeSpy.mockRestore();
  });
});
