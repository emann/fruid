#![warn(clippy::all)]
#![warn(clippy::correctness)]
#![warn(clippy::style)]
#![warn(clippy::complexity)]
#![warn(clippy::perf)]

mod cli_input;

extern crate proc_macro;

use proc_macro::TokenStream;
use syn::{parse_quote, DeriveInput, Path};

#[proc_macro_derive(FromCLIInput, attributes(from_cli_input))]
pub fn derive_from_cli_input(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);
    cli_input::derive_from_cli_input(&input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}
