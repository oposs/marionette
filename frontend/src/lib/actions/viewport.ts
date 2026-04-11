/**
 * Svelte action that fires a callback when the element enters the viewport
 * of its scroll container (or the root viewport if no container is given).
 *
 * Used by DataTable.svelte (Phase 13 Plan 05) as the infinite-scroll sentinel:
 * a zero-height div placed at (or near) the virtualizer's tail with
 * `use:onIntersect={{ onEnter: () => sendAction('fetch-rows', ...) }}`.
 *
 * Usage:
 * ```svelte
 * <div
 *   use:onIntersect={{
 *     onEnter: () => sendAction('fetch-rows', { offset, limit }),
 *     rootMargin: '100px',
 *     enabled: hasMoreRows,
 *   }}
 * ></div>
 * ```
 *
 * Semantics:
 *  - The callback fires on the LEADING edge of an intersection (when
 *    `isIntersecting` flips from false to true), NOT on every
 *    `intersectionRatio` change. This prevents duplicate dispatches while
 *    the user scrolls slowly past the sentinel.
 *  - The observer is disconnected and re-created when `options` change so
 *    callers can swap callbacks or toggle `enabled` cleanly.
 *  - When `enabled: false`, no observer is created (and any existing one is
 *    torn down) — use this to idle the sentinel once `rows.length >=
 *    total_rows` or a fetch response returned fewer than `limit`.
 */
export interface OnIntersectOptions {
	/** Callback fired when the element enters the viewport. */
	onEnter: () => void;
	/** Scroll-container root (default: null = browser viewport). */
	root?: Element | null;
	/** CSS `rootMargin` (default: '0px'). Useful for prefetching. */
	rootMargin?: string;
	/** `threshold` (default: 0 = fire as soon as any pixel is visible). */
	threshold?: number | number[];
	/** When false, no observer is created (default: true). */
	enabled?: boolean;
}

export function onIntersect(node: Element, options: OnIntersectOptions) {
	let observer: IntersectionObserver | undefined;
	let wasIntersecting = false;
	let currentOptions = options;

	function start(opts: OnIntersectOptions) {
		stop();
		currentOptions = opts;
		if (opts.enabled === false) return;
		observer = new IntersectionObserver(
			(entries) => {
				for (const entry of entries) {
					if (entry.isIntersecting && !wasIntersecting) {
						wasIntersecting = true;
						currentOptions.onEnter();
					} else if (!entry.isIntersecting) {
						wasIntersecting = false;
					}
				}
			},
			{
				root: opts.root ?? null,
				rootMargin: opts.rootMargin ?? '0px',
				threshold: opts.threshold ?? 0,
			},
		);
		observer.observe(node);
	}

	function stop() {
		observer?.disconnect();
		observer = undefined;
		wasIntersecting = false;
	}

	start(options);

	return {
		update(newOptions: OnIntersectOptions) {
			start(newOptions);
		},
		destroy() {
			stop();
		},
	};
}
