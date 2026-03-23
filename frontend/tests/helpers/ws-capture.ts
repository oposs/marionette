import type { Page } from '@playwright/test';

export interface CapturedFrame {
	direction: 'sent' | 'received';
	data: Record<string, unknown>;
	timestamp: number;
}

/**
 * Set up WebSocket frame capture on a Playwright page.
 *
 * IMPORTANT: Call this BEFORE page.goto() to capture the initial
 * WebSocket connection frames (including the server hello).
 *
 * Returns a reference to a growing array of captured frames.
 */
export function captureWebSocketFrames(page: Page): CapturedFrame[] {
	const frames: CapturedFrame[] = [];
	page.on('websocket', (ws) => {
		ws.on('framesent', (frame) => {
			try {
				frames.push({
					direction: 'sent',
					data: JSON.parse(frame.payload as string),
					timestamp: Date.now(),
				});
			} catch {
				/* binary frame, skip */
			}
		});
		ws.on('framereceived', (frame) => {
			try {
				frames.push({
					direction: 'received',
					data: JSON.parse(frame.payload as string),
					timestamp: Date.now(),
				});
			} catch {
				/* binary frame, skip */
			}
		});
	});
	return frames;
}
