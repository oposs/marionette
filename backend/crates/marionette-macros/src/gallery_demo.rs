use darling::FromMeta;
use darling::ast::NestedMeta;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, ItemFn};

/// Attribute arguments for `#[gallery_demo(key = "...", name = "...")]`.
///
/// Both args are optional. When absent, `key` defaults to the annotated fn's
/// ident as a string, and `name` defaults to the title-cased form of `key`
/// per [`title_case`].
#[derive(FromMeta, Default)]
struct GalleryDemoOpts {
    #[darling(default)]
    key: Option<String>,
    #[darling(default)]
    name: Option<String>,
}

/// Entry point. Parses attribute args, validates the annotated item, and
/// emits the cfg-gated fn + cfg-gated linkme static.
pub fn gallery_demo_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    // 1. Parse attribute args via darling.
    let meta_list = match NestedMeta::parse_meta_list(attr) {
        Ok(v) => v,
        Err(e) => return darling::Error::from(e).write_errors(),
    };
    let opts = match GalleryDemoOpts::from_list(&meta_list) {
        Ok(v) => v,
        Err(e) => return e.write_errors(),
    };

    // 2. Parse the annotated item. Explicit failure branch so
    //    "applied to non-fn" produces a targeted error.
    let func: ItemFn = match syn::parse2::<ItemFn>(item.clone()) {
        Ok(v) => v,
        Err(_) => {
            return syn::Error::new_spanned(
                item,
                "#[gallery_demo] can only be applied to `pub fn` items \
                 (not structs, enums, modules, or other items)",
            )
            .to_compile_error();
        }
    };

    // 3. Validate signature + visibility.
    if let Err(e) = validate_item(&func) {
        return e.to_compile_error();
    }

    // 4. Derive key + display_name.
    let fn_ident = &func.sig.ident;
    let key = opts.key.unwrap_or_else(|| fn_ident.to_string());
    let display_name = opts.name.unwrap_or_else(|| title_case(&key));

    // 5. Synthesize a unique static ident per annotation site.
    let static_ident = Ident::new(
        &format!("__GALLERY_DEMO_{fn_ident}"),
        fn_ident.span(),
    );

    // 6. Emit cfg-gated fn + cfg-gated linkme static (D-B1).
    //    The linkme path re-routes through marionette::gallery::__linkme
    //    so consumer crates don't need their own `linkme` dep. The
    //    `#[linkme(crate = ...)]` attribute tells linkme's own proc macro
    //    where to find its runtime types — without it, linkme hardcodes
    //    `::linkme` and the consumer crate would need a direct linkme dep
    //    (see linkme-impl/src/attr.rs `linkme_path`).
    quote! {
        #[cfg(feature = "gallery")]
        #func

        #[cfg(feature = "gallery")]
        #[::marionette::gallery::__linkme::distributed_slice(::marionette::gallery::DEMOS)]
        #[linkme(crate = ::marionette::gallery::__linkme)]
        #[allow(non_upper_case_globals)]
        static #static_ident: ::marionette::gallery::DemoEntry =
            ::marionette::gallery::DemoEntry {
                key: #key,
                render: #fn_ident,
                display_name: #display_name,
            };
    }
}

/// Validate that `func` is `pub fn name() -> Node` with no args, no generics,
/// no async, no where-clause. Returns the first violation as a spanned error.
fn validate_item(func: &ItemFn) -> Result<(), syn::Error> {
    // Visibility.
    if !matches!(func.vis, syn::Visibility::Public(_)) {
        return Err(syn::Error::new_spanned(
            &func.sig.ident,
            "#[gallery_demo] requires `pub fn` visibility (found private fn)",
        ));
    }

    // Arguments.
    if !func.sig.inputs.is_empty() {
        return Err(syn::Error::new_spanned(
            &func.sig.inputs,
            format!(
                "#[gallery_demo] fn must be `fn() -> Node` with zero arguments (found {})",
                func.sig.inputs.len()
            ),
        ));
    }

    // async.
    if func.sig.asyncness.is_some() {
        return Err(syn::Error::new_spanned(
            func.sig.asyncness,
            "#[gallery_demo] fn must not be async",
        ));
    }

    // Generics.
    if !func.sig.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            &func.sig.generics,
            "#[gallery_demo] fn must not have generic parameters",
        ));
    }

    // where-clause.
    if func.sig.generics.where_clause.is_some() {
        return Err(syn::Error::new_spanned(
            &func.sig.generics.where_clause,
            "#[gallery_demo] fn must not have a where-clause",
        ));
    }

    // Return type must be `Vec<Node>`.
    match &func.sig.output {
        syn::ReturnType::Default => Err(syn::Error::new_spanned(
            &func.sig,
            "#[gallery_demo] fn must return `Vec<Node>` (found unit return type)",
        )),
        syn::ReturnType::Type(_, ty) => {
            if return_type_is_vec_node(ty) {
                Ok(())
            } else {
                Err(syn::Error::new_spanned(
                    ty,
                    "#[gallery_demo] fn must return `Vec<Node>` (an alias for \
                     `Vec<(String, marionette_protocol::Component)>`) — index 0 is the root, \
                     remaining entries are descendants",
                ))
            }
        }
    }
}

/// True if `ty` is a path whose last segment is `Vec<Node>` (the single
/// generic argument must be a path whose last segment ident equals `Node`).
/// Accepts `Vec<Node>`, `std::vec::Vec<Node>`, `::alloc::vec::Vec<crate::gallery::Node>`, etc.
fn return_type_is_vec_node(ty: &syn::Type) -> bool {
    let syn::Type::Path(p) = ty else { return false };
    let Some(last) = p.path.segments.last() else { return false };
    if last.ident != "Vec" {
        return false;
    }
    let syn::PathArguments::AngleBracketed(args) = &last.arguments else { return false };
    let Some(syn::GenericArgument::Type(inner)) = args.args.first() else { return false };
    let syn::Type::Path(inner_p) = inner else { return false };
    inner_p.path.segments.last().is_some_and(|s| s.ident == "Node")
}

/// ASCII title-casing for default `display_name`.
///
/// Splits on `-` and `_`, filters empty chunks, uppercases the first ASCII
/// char of each chunk, joins with a single space. Non-ASCII characters pass
/// through untouched (e.g. `"OKLCH-swatches"` -> `"OKLCH Swatches"` — the
/// already-upper `"OKLCH"` tail survives because only the first char is
/// touched).
fn title_case(key: &str) -> String {
    key.split(['-', '_'])
        .filter(|s| !s.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => {
                    let mut s = String::with_capacity(word.len());
                    s.extend(c.to_uppercase());
                    s.push_str(chars.as_str());
                    s
                }
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::title_case;

    #[test]
    fn title_case_single_word() {
        assert_eq!(title_case("button"), "Button");
    }

    #[test]
    fn title_case_hyphenated() {
        assert_eq!(title_case("text-input"), "Text Input");
    }

    #[test]
    fn title_case_underscored() {
        assert_eq!(title_case("data_table"), "Data Table");
    }

    #[test]
    fn title_case_preserves_mixed_case_tail() {
        assert_eq!(title_case("OKLCH-swatches"), "OKLCH Swatches");
    }

    #[test]
    fn title_case_empty_string() {
        assert_eq!(title_case(""), "");
    }

    #[test]
    fn title_case_trailing_separator() {
        assert_eq!(title_case("foo-"), "Foo");
    }

    #[test]
    fn title_case_double_separator() {
        assert_eq!(title_case("foo--bar"), "Foo Bar");
    }
}
