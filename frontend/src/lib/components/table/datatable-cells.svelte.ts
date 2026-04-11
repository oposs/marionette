/**
 * Per-kind cell snippet factories for DataTable. Each returns a snippet
 * that can be passed to FlexRender via `renderSnippet` in the TanStack
 * column `cell` callback (Phase 13 D-F1).
 *
 * Snippets produced via `createRawSnippet` emit raw HTML strings, so every
 * user-controlled value that flows through a template literal MUST be
 * HTML-escaped with `escapeHtml` below. The actions cell kind does NOT
 * use these factories — it uses `renderComponent(DataTableActions, …)`,
 * which is XSS-safe via Svelte's text interpolation in the component
 * itself (see DataTableActions.browser-test.ts test 5).
 */

import { createRawSnippet } from 'svelte';

/**
 * Renders a formatted date. Input is an ISO-8601 string; format uses
 * `Intl.DateTimeFormat` with `dateStyle: 'medium'` in the caller's locale.
 */
export const dateCellSnippet = createRawSnippet<[{ iso: string }]>((getArgs) => ({
	render: () => {
		const { iso } = getArgs();
		if (!iso) return '<span class="text-sm text-muted-foreground"></span>';
		const d = new Date(iso);
		if (Number.isNaN(d.getTime())) {
			return `<span class="text-sm text-muted-foreground">${escapeHtml(iso)}</span>`;
		}
		const formatted = new Intl.DateTimeFormat(undefined, {
			dateStyle: 'medium',
		}).format(d);
		return `<span class="text-sm">${escapeHtml(formatted)}</span>`;
	},
}));

/**
 * Renders a right-aligned number. Input is any number-coercible value;
 * format uses `Intl.NumberFormat` in the caller's locale. The output span
 * carries `tabular-nums` + `text-right` so columns of numbers align.
 */
export const numberCellSnippet = createRawSnippet<[{ value: number }]>((getArgs) => ({
	render: () => {
		const { value } = getArgs();
		const num = Number(value);
		const safe = Number.isFinite(num) ? num : 0;
		const formatted = new Intl.NumberFormat().format(safe);
		return `<span class="text-right tabular-nums block">${escapeHtml(formatted)}</span>`;
	},
}));

/**
 * Renders a pill-shaped badge around a label. Supports an optional variant
 * map for conventional status colors (success/error/outline/secondary/
 * default). Not tied to the shadcn `Badge` component itself to avoid
 * shipping two badge implementations — the inline classes mirror the
 * shadcn-svelte Badge's visual footprint.
 */
export const badgeCellSnippet = createRawSnippet<[{ label: string; variant?: string }]>(
	(getArgs) => ({
		render: () => {
			const { label, variant } = getArgs();
			const cls = variantToClass(variant);
			return `<span class="inline-flex items-center rounded-md px-2 py-0.5 text-xs font-medium ${cls}">${escapeHtml(String(label ?? ''))}</span>`;
		},
	}),
);

function variantToClass(variant?: string): string {
	switch (variant) {
		case 'success':
		case 'default':
			return 'bg-primary text-primary-foreground';
		case 'destructive':
		case 'error':
			return 'bg-destructive text-destructive-foreground';
		case 'outline':
			return 'border border-input bg-background text-foreground';
		case 'secondary':
		default:
			return 'bg-secondary text-secondary-foreground';
	}
}

/**
 * Minimal HTML-escape — `createRawSnippet` emits raw HTML, so anything
 * originating from server-supplied row data MUST pass through here before
 * being interpolated into a template string. Test coverage lives in
 * DataTable.browser-test.ts § Cell kinds.
 */
function escapeHtml(s: string): string {
	return s
		.replace(/&/g, '&amp;')
		.replace(/</g, '&lt;')
		.replace(/>/g, '&gt;')
		.replace(/"/g, '&quot;')
		.replace(/'/g, '&#39;');
}
