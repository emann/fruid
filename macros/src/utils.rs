use proc_macro2::Span;
use proc_macro_crate::{crate_name, FoundCrate};
use quote::format_ident;
use syn::parse_quote;
use syn::Path;

/// Gets the name of the `opu_macro_utils` crate, i.e. the one that defines the cli,
/// for dynamically generating paths.
pub(crate) fn crate_path() -> Path {
    crate_name("fruid")
        .map_err(|err| syn::Error::new(Span::call_site(), err))
        .map(|result| match result {
            // If the environment variable exists, we are running integration tests
            // and need to use the crate's actual name.
            // See https://github.com/bkchr/proc-macro-crate/issues/10
            FoundCrate::Itself => match std::env::var_os("CARGO_CRATE_NAME") {
                None => parse_quote! { ::opu_macro_utils },
                Some(name) => {
                    if name == "opu_macro_utils" {
                        parse_quote! { crate }
                    } else {
                        parse_quote! { ::opu_macro_utils }
                    }
                }
            },
            FoundCrate::Name(name) => {
                let ident = format_ident!("{}", name);
                parse_quote! { ::#ident }
            }
        })
        .expect("failed to find opu_macro_utils crate name")
}
