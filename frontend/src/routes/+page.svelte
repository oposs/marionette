<script lang="ts">
	import { onMount } from 'svelte';
	import { initMarionette, destroyMarionette, setSurfaceTree, setFullState } from '$lib';
	import { isConnected } from '$lib/transport/websocket.svelte';
	import type { ComponentNode } from '$lib/transport/messages';

	let demoTimer: ReturnType<typeof setTimeout> | undefined;

	function loadDemoData() {
		// Sidebar: navigation
		const sidebarNodes: Record<string, ComponentNode> = {
			'sidebar-root': { type: 'side-nav', children: ['nav-group-1'] },
			'nav-group-1': { type: 'nav-group', children: ['nav-dashboard', 'nav-contacts', 'nav-settings'] },
			'nav-dashboard': { type: 'nav-item', props: { label: 'Dashboard', href: '/dashboard', active: true } },
			'nav-contacts': { type: 'nav-item', props: { label: 'Contacts', href: '/contacts' } },
			'nav-settings': { type: 'nav-item', props: { label: 'Settings', href: '/settings' } },
		};
		setSurfaceTree('sidebar', 'sidebar-root', sidebarNodes);
		setFullState('sidebar', {});

		// Main: form + table demo
		const mainNodes: Record<string, ComponentNode> = {
			'main-root': { type: 'container', children: ['main-heading', 'main-text', 'main-form', 'main-table'] },
			'main-heading': { type: 'heading', props: { text: 'Contact Management', level: 1 } },
			'main-text': { type: 'text', props: { text: 'Manage your CRM contacts below.' } },
			'main-form': {
				type: 'form',
				action: { type: 'submit', name: 'save-contact' },
				children: ['form-name', 'form-email', 'form-submit'],
			},
			'form-name': { type: 'text-input', props: { label: 'Name', placeholder: 'Enter name' }, bind: '/contact/name' },
			'form-email': { type: 'text-input', props: { label: 'Email', placeholder: 'Enter email' }, bind: '/contact/email' },
			'form-submit': { type: 'button', props: { label: 'Save Contact', color: 'primary' }, action: { type: 'submit', name: 'save-contact' } },
			'main-table': {
				type: 'data-table',
				props: {
					columns: [
						{ key: 'name', label: 'Name', sortable: true },
						{ key: 'email', label: 'Email', sortable: true },
						{ key: 'company', label: 'Company', sortable: false },
					],
					totalRows: 3,
				},
				bind: '/contacts',
				action: { type: 'click', name: 'select-row' },
			},
		};
		setSurfaceTree('main', 'main-root', mainNodes);
		setFullState('main', {
			contact: { name: '', email: '' },
			contacts: {
				'1': { id: '1', name: 'Alice Johnson', email: 'alice@example.com', company: 'Acme Corp' },
				'2': { id: '2', name: 'Bob Smith', email: 'bob@example.com', company: 'Globex Inc' },
				'3': { id: '3', name: 'Carol White', email: 'carol@example.com', company: 'Initech' },
			},
		});
	}

	onMount(() => {
		initMarionette();

		// If WebSocket fails to connect within 2 seconds, switch to demo mode
		demoTimer = setTimeout(() => {
			if (!isConnected()) {
				loadDemoData();
			}
		}, 2000);

		return () => {
			if (demoTimer) clearTimeout(demoTimer);
			destroyMarionette();
		};
	});
</script>
