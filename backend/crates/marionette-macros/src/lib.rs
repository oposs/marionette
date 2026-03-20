#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::needless_continue)] // darling-generated code triggers this

mod action;
mod component_builder;
mod requires;

use proc_macro::TokenStream;

/// Derive a fluent builder for a component struct.
///
/// Apply `#[component(type = "text-input")]` to specify the component type string.
/// Fields without `#[builder(optional)]` become required constructor parameters.
/// Fields with `#[builder(optional)]` become optional setter methods on the builder.
///
/// # Example
///
/// ```ignore
/// #[derive(ComponentBuilder)]
/// #[component(type = "button")]
/// pub struct Button {
///     pub label: String,
///     #[builder(optional)]
///     pub variant: Option<String>,
/// }
///
/// let (id, component) = Button::new("Save").variant("primary").build();
/// ```
#[proc_macro_derive(ComponentBuilder, attributes(component, builder))]
pub fn derive_component_builder(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    component_builder::derive_component_builder(&input).into()
}

/// Generate an action name constant from a handler function.
///
/// # Example
///
/// ```ignore
/// #[action(name = "save-contact")]
/// async fn save_contact() { }
/// // Generates: pub const SAVE_CONTACT: &str = "save-contact";
/// ```
#[proc_macro_attribute]
pub fn action(attr: TokenStream, item: TokenStream) -> TokenStream {
    action::action_impl(attr.into(), item.into()).into()
}

/// Generate authorization metadata for a handler function.
///
/// # Example
///
/// ```ignore
/// #[requires(authenticated)]
/// async fn save_contact() { }
/// // Generates: pub const SAVE_CONTACT_AUTH: AuthRequirement = AuthRequirement::Authenticated;
///
/// #[requires(role = "admin")]
/// async fn delete_user() { }
/// // Generates: pub const DELETE_USER_AUTH: AuthRequirement = AuthRequirement::Role("admin");
/// ```
#[proc_macro_attribute]
pub fn requires(attr: TokenStream, item: TokenStream) -> TokenStream {
    requires::requires_impl(attr.into(), item.into()).into()
}
