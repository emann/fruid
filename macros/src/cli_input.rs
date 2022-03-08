use crate::parsed_field::ParsedField;
use crate::utils;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use std::convert::TryFrom;
use syn::spanned::Spanned;
use syn::{parse_quote, DataEnum, DataStruct, DeriveInput, Fields, Path};

pub(crate) fn impl_for_struct(input: &DeriveInput, data: &DataStruct) -> syn::Result<TokenStream> {
    let ident = &input.ident;

    let fields = match &data.fields {
        Fields::Unnamed(_) | Fields::Unit => Err(syn::Error::new(
            data.fields.span(),
            "only named fields are currently supported",
        )),
        Fields::Named(named) => Ok(&named.named),
    }?
    .iter()
    .map(ParsedField::try_from)
    .collect::<syn::Result<Vec<_>>>()?;

    let from_cli_input_impl = impl_from_cli_input_for_struct(ident, &fields)?;

    Ok(quote! {
        #from_cli_input_impl
    })
}

/// Generate the implementation of `FromCLIInput` for this struct.
fn impl_from_cli_input_for_struct(
    ident: &Ident,
    fields: &[ParsedField],
) -> syn::Result<TokenStream> {
    let from_cli_input_trait_path = get_from_cli_input_trait_path();
    let field_prompts = fields
        .iter()
        .map(|pf| {
            let field_ident = pf.ident;
            let field_type = &pf.field.ty;
            let field_prompt = pf
                .prompt
                .clone()
                .unwrap_or_else(|| String::from("Default prompt"));
            // TODO: Fix default str values
            if pf.skip_prompt_and_use_default {
                match &pf.default {
                    Some(expr_for_default_value) => Ok(quote! {
                        let #field_ident: #field_type = #expr_for_default_value;
                    }),
                    // TODO make this a real error
                    None => panic!("No default provided when skip_prompt_and_use_default=true.")
                }
            }
            else {
                match &pf.default {
                    Some(expr_for_default_value) => Ok(quote! {
                        let #field_ident: #field_type = #from_cli_input_trait_path::from_cli_input(&#field_prompt, Some(#expr_for_default_value));
                    }),
                    None => Ok(quote! {
                        let #field_ident: #field_type = #from_cli_input_trait_path::from_cli_input(&#field_prompt, None);
                    })
                }
            }


        })
        .collect::<syn::Result<Vec<_>>>()?;

    let build_struct = {
        let field_idents = fields.iter().map(|field| field.ident);
        quote! { #ident { #(#field_idents),* } }
    };

    Ok(quote! {
        impl #from_cli_input_trait_path for #ident {
            fn from_cli_input(prompt: &str, _default: Option<Self>) -> Self {
                #(#field_prompts)*
                #build_struct
            }
        }
    })
}

pub(crate) fn impl_for_enum(input: &DeriveInput, data: &DataEnum) -> syn::Result<TokenStream> {
    let from_cli_input_trait_path = get_from_cli_input_trait_path();
    let prompt_select_path = get_prompt_select_path();

    let enum_name = &input.ident;
    let (variant_names, match_selected_variant_arms): (Vec<TokenStream>, Vec<TokenStream>) =
        data.variants
            .iter()
            .map(|v| {
                let ident_name = v.ident.to_string();
                let variant_name = quote! { #ident_name };

                let match_selected_variant_arm = if v.fields.is_empty() {
                    quote! {
                        // "A" => MyEnum::A
                        #variant_name => #enum_name::#v,
                    }
                } else {
                    quote! {
                        // "B" => MyEnum::B(FromCLIInput::from_cli_input())
                        #variant_name => #enum_name::#v(#from_cli_input_trait_path::from_cli_input("", None)),
                    }
                };


                (variant_name, match_selected_variant_arm)
            })
            .unzip();
    let choices_tokens: Vec<TokenStream> = variant_names.iter().map(|t| quote! {#t,}).collect();

    Ok(quote! {
        impl #from_cli_input_trait_path for #enum_name {
            fn from_cli_input(prompt: &str, _default: Option<Self>) -> Self {
                let choices = vec![#(#choices_tokens)*];
                let enum_variant_name: &str = #prompt_select_path(prompt, choices);
                match enum_variant_name {
                    #(#match_selected_variant_arms)*
                    _ => panic!("Unexpected selection")
                }
            }
        }
    })
}

fn get_from_cli_input_trait_path() -> Path {
    let root = utils::crate_path();
    parse_quote! { #root::FromCLIInput }
}

fn get_prompt_select_path() -> Path {
    let root = utils::crate_path();
    parse_quote! { #root::prompt::select }
}
