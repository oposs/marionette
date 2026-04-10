let open = $state(false);
export function isSidebarOpen(): boolean { return open; }
export function toggleSidebar(): void { open = !open; }
export function closeSidebar(): void { open = false; }
