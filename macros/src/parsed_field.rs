use proc_macro2::Ident;
use std::convert::TryFrom;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{Attribute, Expr, Field, Lit, Meta, NestedMeta, Token};

const ATTR_FROM_INPUT: &str = "from_cli_input";
const ATTR_KEY_PROMPT: &str = "prompt";
const ATTR_KEY_DEFAULT_VALUE: &str = "default";
const ATTR_KEY_SKIP_PROMPT: &str = "skip_prompt_and_use_default";

#[derive(Debug, Clone)]
pub(crate) struct ParsedField<'a> {
    pub(crate) field: &'a Field,
    pub(crate) ident: &'a Ident,
    pub(crate) prompt: Option<String>,
    pub(crate) default: Option<Expr>,
    pub(crate) skip_prompt_and_use_default: bool,
}

impl<'a> TryFrom<&'a Field> for ParsedField<'a> {
    type Error = syn::Error;

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
                            ATTR_KEY_PROMPT => {
                                prompt = match &kv.lit {
                                    Lit::Str(s) => Ok(Some(s.value())),
                                    _ => Err(syn::Error::new(kv.lit.span(), "expected a string")),
                                }?
                            }
                            ATTR_KEY_DEFAULT_VALUE => match &kv.lit {
                                Lit::Str(expr) => default = Some(expr.parse::<Expr>()?),
                                _ => {
                                    return Err(syn::Error::new(
                                        kv.lit.span(),
                                        "expected an expression",
                                    ))
                                }
                            },
                            ATTR_KEY_SKIP_PROMPT => match &kv.lit {
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
            field,
            ident,
            prompt,
            default,
            skip_prompt_and_use_default,
        })
    }
}

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
