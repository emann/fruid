use itertools::{izip, MultiUnzip};
use proc_macro2::{Ident, Span, TokenStream};
use proc_macro_crate::{crate_name, FoundCrate};
use quote::{format_ident, quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{
    parse_quote, Attribute, Data, DataEnum, DataStruct, DeriveInput, Expr, Field, Fields, Lit,
    LitStr, Meta, NestedMeta, Path, Token, Type,
};

pub(crate) fn derive_from_cli_input(input: &DeriveInput) -> Result<TokenStream, syn::Error> {
    match &input.data {
        Data::Struct(struct_data) => get_struct_from_cli_input_impl(input, struct_data),
        Data::Enum(enum_data) => get_enum_from_cli_input_impl(input, enum_data),
        _ => Err(syn::Error::new(
            input.span(),
            "Unions are not supported at this time.",
        )),
    }
}

pub(crate) fn get_struct_from_cli_input_impl(
    input: &DeriveInput,
    data: &DataStruct,
) -> syn::Result<TokenStream> {
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

pub(crate) fn get_enum_from_cli_input_impl(
    input: &DeriveInput,
    data: &DataEnum,
) -> syn::Result<TokenStream> {
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

    let ts = quote! {
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
    };

    println!("{:?}", ts.to_string());

    Ok(ts)
}

#[derive(Debug, Clone)]
pub(crate) struct ParsedField<'a> {
    pub(crate) prompt: Option<String>,
    pub(crate) default: Option<Expr>,
    pub(crate) skip_prompt_and_use_default: bool,
    pub(crate) ident: &'a Ident,
    pub(crate) field: &'a Field,
}

impl<'a> TryFrom<&'a Field> for ParsedField<'a> {
    type Error = syn::Error;

    #[allow(clippy::similar_names)]
    fn try_from(field: &'a Field) -> syn::Result<Self> {
        let mut prompt = None;
        let mut default = None;
        let mut skip_prompt_and_use_default = false;
        let ident = field
            .ident
            .as_ref()
            .ok_or_else(|| syn::Error::new(field.span(), "expected a named field"))?;

        if let Some(nested) = get_inner_meta(&field.attrs)? {
            for meta in nested {
                if let NestedMeta::Meta(Meta::NameValue(kv)) = meta {
                    if let Some(id) = kv.path.get_ident() {
                        match id.to_string().as_str() {
                            "prompt" => {
                                prompt = match &kv.lit {
                                    Lit::Str(s) => Ok(Some(s.value())),
                                    _ => Err(syn::Error::new(kv.lit.span(), "expected a string")),
                                }?
                            }
                            "default" => match &kv.lit {
                                Lit::Str(expr) => default = Some(expr.parse::<Expr>()?),
                                _ => {
                                    return Err(syn::Error::new(
                                        kv.lit.span(),
                                        "expected an expression",
                                    ))
                                }
                            },
                            "skip_prompt_and_use_default" => match &kv.lit {
                                Lit::Bool(lit_bool) => skip_prompt_and_use_default = lit_bool.value,
                                _ => return Err(syn::Error::new(kv.lit.span(), "expected a bool")),
                            },
                            _ => {}
                        }
                    }
                }
            }
        };

        Ok(Self {
            prompt,
            default,
            skip_prompt_and_use_default,
            ident,
            field,
        })
    }
}

const ATTR_FROM_INPUT: &str = "from_cli_input";

/// Get all of the nested meta attributes in the *first* `from_cli_input()` attribute found.
fn get_inner_meta(attrs: &[Attribute]) -> syn::Result<Option<Punctuated<NestedMeta, Token![,]>>> {
    for attr in attrs {
        if let Some(id) = attr.path.get_ident() {
            if id == ATTR_FROM_INPUT {
                // Parse as `from_cli_input(...)`
                match attr.parse_meta()? {
                    Meta::List(list) => return Ok(Some(list.nested)),
                    _ => {
                        return Err(syn::Error::new(
                            attr.span(),
                            "expected list, i.e. #[from_cli_input(...)]",
                        ))
                    }
                }
            }
        }
    }

    Ok(None)
}

/// Gets the name of the `opu_macro_utils` crate, i.e. the one that defines the cli,
/// for dynamically generating paths.
pub(crate) fn macro_utils_crate_path() -> Path {
    crate_name("opu_macro_utils")
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

fn get_from_cli_input_trait_path() -> Path {
    let root = macro_utils_crate_path();
    parse_quote! { #root::FromCLIInput }
}

fn get_prompt_select_path() -> Path {
    let root = macro_utils_crate_path();
    parse_quote! { #root::prompt::select }
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
                // ::log::debug!("Creating a {} from CLI input", ::std::stringify!(#ident));
                println!("{}", prompt);
                #(#field_prompts)*
                #build_struct
            }
        }
    })
}
