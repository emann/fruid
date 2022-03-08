#![warn(clippy::all)]
#![warn(clippy::correctness)]
#![warn(clippy::style)]
#![warn(clippy::complexity)]
#![warn(clippy::perf)]

mod cli_input;
mod parsed_field;
mod utils;

extern crate proc_macro;

use proc_macro::TokenStream;
use syn::spanned::Spanned;
use syn::{Data, DeriveInput};

#[proc_macro_derive(FromCLIInput, attributes(from_cli_input))]
pub fn derive_from_cli_input(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);
    let derive_result = match &input.data {
        Data::Struct(struct_data) => cli_input::impl_for_struct(&input, struct_data),
        Data::Enum(enum_data) => cli_input::impl_for_enum(&input, enum_data),
        _ => Err(syn::Error::new(
            input.span(),
            "Unions are not supported at this time.",
        )),
    };
    derive_result
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}
