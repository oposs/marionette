use darling::{FromDeriveInput, FromField};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Ident};

#[derive(FromDeriveInput)]
#[darling(attributes(component))]
struct ComponentOpts {
    ident: Ident,
    data: darling::ast::Data<(), FieldOpts>,
    #[darling(rename = "type")]
    component_type: String,
}

#[derive(FromField)]
#[darling(attributes(builder))]
struct FieldOpts {
    ident: Option<Ident>,
    ty: syn::Type,
    #[darling(default)]
    optional: bool,
}

#[allow(clippy::too_many_lines)]
pub fn derive_component_builder(input: &DeriveInput) -> TokenStream {
    let opts = match ComponentOpts::from_derive_input(input) {
        Ok(v) => v,
        Err(e) => return e.write_errors(),
    };

    let struct_name = &opts.ident;
    let builder_name = Ident::new(&format!("{struct_name}Builder"), struct_name.span());
    let component_type = &opts.component_type;

    let fields = opts
        .data
        .take_struct()
        .expect("ComponentBuilder can only be derived on structs")
        .fields;

    let required_fields: Vec<_> = fields.iter().filter(|f| !f.optional).collect();
    let optional_fields: Vec<_> = fields.iter().filter(|f| f.optional).collect();

    // Builder struct fields
    let required_field_defs = required_fields.iter().map(|f| {
        let name = f.ident.as_ref().unwrap();
        let ty = &f.ty;
        quote! { #name: #ty }
    });

    let optional_field_defs = optional_fields.iter().map(|f| {
        let name = f.ident.as_ref().unwrap();
        let ty = &f.ty;
        quote! { #name: #ty }
    });

    // Constructor parameters
    let new_params = required_fields.iter().map(|f| {
        let name = f.ident.as_ref().unwrap();
        let ty = &f.ty;
        if is_string_type(ty) {
            quote! { #name: impl ::core::convert::Into<::std::string::String> }
        } else {
            quote! { #name: #ty }
        }
    });

    let new_inits = required_fields.iter().map(|f| {
        let name = f.ident.as_ref().unwrap();
        let ty = &f.ty;
        if is_string_type(ty) {
            quote! { #name: #name.into() }
        } else {
            quote! { #name: #name }
        }
    });

    let optional_inits = optional_fields.iter().map(|f| {
        let name = f.ident.as_ref().unwrap();
        quote! { #name: ::core::option::Option::None }
    });

    // Optional setter methods
    let optional_setters = optional_fields.iter().map(|f| {
        let name = f.ident.as_ref().unwrap();
        let ty = &f.ty;
        let inner_ty = extract_option_inner(ty);
        match inner_ty {
            Some(inner) if is_string_type(inner) => {
                quote! {
                    #[must_use]
                    pub fn #name(mut self, value: impl ::core::convert::Into<::std::string::String>) -> Self {
                        self.#name = ::core::option::Option::Some(value.into());
                        self
                    }
                }
            }
            Some(inner) => {
                quote! {
                    #[must_use]
                    pub fn #name(mut self, value: #inner) -> Self {
                        self.#name = ::core::option::Option::Some(value);
                        self
                    }
                }
            }
            None => {
                quote! {
                    #[must_use]
                    pub fn #name(mut self, value: #ty) -> Self {
                        self.#name = ::core::option::Option::Some(value);
                        self
                    }
                }
            }
        }
    });

    // Build props JSON
    let required_props = required_fields.iter().map(|f| {
        let name = f.ident.as_ref().unwrap();
        let key = name.to_string();
        quote! {
            props_map.insert(
                ::std::string::String::from(#key),
                ::serde_json::to_value(&self.#name).unwrap_or(::serde_json::Value::Null),
            );
        }
    });

    let optional_props = optional_fields.iter().map(|f| {
        let name = f.ident.as_ref().unwrap();
        let key = name.to_string();
        quote! {
            if let ::core::option::Option::Some(ref val) = self.#name {
                props_map.insert(
                    ::std::string::String::from(#key),
                    ::serde_json::to_value(val).unwrap_or(::serde_json::Value::Null),
                );
            }
        }
    });

    quote! {
        impl #struct_name {
            /// Create a new builder with required fields.
            pub fn new(#(#new_params),*) -> #builder_name {
                #builder_name {
                    #(#new_inits,)*
                    #(#optional_inits,)*
                    __bind: ::core::option::Option::None,
                    __action: ::core::option::Option::None,
                    __visible: ::core::option::Option::None,
                    __children: ::std::vec::Vec::new(),
                    __id: ::core::option::Option::None,
                }
            }
        }

        /// Fluent builder generated by `#[derive(ComponentBuilder)]`.
        pub struct #builder_name {
            #(#required_field_defs,)*
            #(#optional_field_defs,)*
            __bind: ::core::option::Option<::std::string::String>,
            __action: ::core::option::Option<::marionette_protocol::ComponentAction>,
            __visible: ::core::option::Option<::std::string::String>,
            __children: ::std::vec::Vec<(::std::string::String, ::marionette_protocol::Component)>,
            __id: ::core::option::Option<::std::string::String>,
        }

        impl #builder_name {
            #(#optional_setters)*

            /// Bind this component to a data path (JSON Pointer).
            #[must_use]
            pub fn bind(mut self, path: impl ::core::convert::Into<::std::string::String>) -> Self {
                self.__bind = ::core::option::Option::Some(path.into());
                self
            }

            /// Set the action triggered by this component.
            #[must_use]
            pub fn action(mut self, action: ::marionette_protocol::ComponentAction) -> Self {
                self.__action = ::core::option::Option::Some(action);
                self
            }

            /// Set visibility binding (JSON Pointer to a boolean).
            #[must_use]
            pub fn visible(mut self, path: impl ::core::convert::Into<::std::string::String>) -> Self {
                self.__visible = ::core::option::Option::Some(path.into());
                self
            }

            /// Append a child node. The child's `(id, Component)` is collected for later flattening.
            #[must_use]
            pub fn child(mut self, child: (::std::string::String, ::marionette_protocol::Component)) -> Self {
                self.__children.push(child);
                self
            }

            /// Append multiple child nodes at once.
            #[must_use]
            pub fn children(mut self, children: ::std::vec::Vec<(::std::string::String, ::marionette_protocol::Component)>) -> Self {
                self.__children.extend(children);
                self
            }

            /// Override the auto-generated node ID.
            #[must_use]
            pub fn id(mut self, id: impl ::core::convert::Into<::std::string::String>) -> Self {
                self.__id = ::core::option::Option::Some(id.into());
                self
            }

            /// Build the component, returning `(node_id, Component)`.
            ///
            /// The node ID is either user-set via `.id()` or auto-generated as
            /// `"{component_type}-{uuid}"`.
            #[must_use]
            pub fn build(self) -> (::std::string::String, ::marionette_protocol::Component) {
                let mut props_map = ::serde_json::Map::new();
                #(#required_props)*
                #(#optional_props)*

                let props = if props_map.is_empty() {
                    ::core::option::Option::None
                } else {
                    ::core::option::Option::Some(::serde_json::Value::Object(props_map))
                };

                let child_ids: ::core::option::Option<::std::vec::Vec<::std::string::String>> = if self.__children.is_empty() {
                    ::core::option::Option::None
                } else {
                    ::core::option::Option::Some(
                        self.__children.iter().map(|(id, _)| id.clone()).collect()
                    )
                };

                let node_id = self.__id.unwrap_or_else(|| {
                    ::std::format!("{}-{}", #component_type, ::uuid::Uuid::new_v4())
                });

                let component = ::marionette_protocol::Component {
                    r#type: ::std::string::String::from(#component_type),
                    props,
                    children: child_ids,
                    bind: self.__bind,
                    action: self.__action,
                    visible: self.__visible,
                };

                (node_id, component)
            }

            /// Build this component and return a flat list of all nodes
            /// (this component plus all collected children).
            #[must_use]
            pub fn build_with_children(self) -> ::std::vec::Vec<(::std::string::String, ::marionette_protocol::Component)> {
                let children = self.__children.clone();
                let (id, component) = self.build();
                let mut nodes = ::std::vec![( id, component )];
                nodes.extend(children);
                nodes
            }
        }
    }
}

/// Check if a type is `String`.
fn is_string_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(p) = ty {
        let path = &p.path;
        if path.segments.len() == 1 {
            return path.segments[0].ident == "String";
        }
    }
    false
}

/// Extract the inner type from `Option<T>`.
fn extract_option_inner(ty: &syn::Type) -> Option<&syn::Type> {
    if let syn::Type::Path(p) = ty
        && p.path.segments.len() == 1
        && p.path.segments[0].ident == "Option"
        && let syn::PathArguments::AngleBracketed(args) = &p.path.segments[0].arguments
        && args.args.len() == 1
        && let syn::GenericArgument::Type(inner) = &args.args[0]
    {
        return Some(inner);
    }
    None
}
