#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::needless_continue)] // darling-generated code triggers this

mod action;
mod component_builder;
mod gallery_demo;
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

/// Register a gallery demo function for auto-discovery.
///
/// Applied to a `pub fn name() -> Node`, emits a cfg-gated copy of the fn
/// plus a `#[linkme::distributed_slice(marionette::gallery::DEMOS)]` static
/// that registers the fn in the gallery registry. Both items are gated
/// behind `#[cfg(feature = "gallery")]` on the consumer crate — under
/// default build, neither the fn symbol nor the registry entry exists.
///
/// # Arguments
///
/// - `key = "..."` (optional) — registry sort key. Defaults to the fn ident.
/// - `name = "..."` (optional) — nav-facing label. Defaults to title-cased `key`.
///
/// # Example
///
/// ```ignore
/// use marionette::gallery::Node;
/// use marionette_macros::gallery_demo;
///
/// #[gallery_demo(key = "button", name = "Button")]
/// pub fn gallery_demo() -> Node {
///     Button::new("Click").build()
/// }
/// ```
///
/// Misuse (non-`pub`, args, generics, `async`, wrong return type, or applied
/// to a non-fn item) produces a compile error that names the violated rule.
#[proc_macro_attribute]
pub fn gallery_demo(attr: TokenStream, item: TokenStream) -> TokenStream {
    gallery_demo::gallery_demo_impl(attr.into(), item.into()).into()
}
