/**
 * Default component registrations.
 *
 * Registers all built-in Marionette SDUI components:
 * navigation, layout, and feedback components (Plan 04),
 * with form and data components to follow (Plan 05).
 */
import { registerAll } from './registry';
import SideNav from '../components/nav/SideNav.svelte';
import NavItem from '../components/nav/NavItem.svelte';
import NavGroup from '../components/nav/NavGroup.svelte';
import Container from '../components/layout/Container.svelte';
import Grid from '../components/layout/Grid.svelte';
import Heading from '../components/layout/Heading.svelte';
import Text from '../components/layout/Text.svelte';
import MSpinner from '../components/feedback/Spinner.svelte';
import ErrorDisplay from '../components/feedback/ErrorDisplay.svelte';

export function registerDefaults(): void {
	registerAll({
		'side-nav': SideNav,
		'nav-item': NavItem,
		'nav-group': NavGroup,
		'container': Container,
		'grid': Grid,
		'heading': Heading,
		'text': Text,
		'spinner': MSpinner,
		'error-display': ErrorDisplay,
	});
}
