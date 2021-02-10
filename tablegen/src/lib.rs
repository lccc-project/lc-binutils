use std::path::PathBuf;

use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{parse::Parse, punctuated::Punctuated};

mod kw {
    syn::custom_keyword!(arch);
    syn::custom_keyword!(fields);
    syn::custom_keyword!(template);
}

struct Fields {
    pub bang: syn::Token![!],
    pub bracket: syn::token::Bracket,
    pub fields: Punctuated<syn::Ident, syn::Token![,]>,
}

impl Parse for Fields {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let buff;
        let bang = input.parse()?;
        let bracket = syn::bracketed!(buff in input);
        let fields = Punctuated::parse_terminated(input)?;
        Ok(Self {
            bang,
            bracket,
            fields,
        })
    }
}

struct Input {
    pub kw_arch: kw::arch,
    pub eq1: syn::Token![=],
    pub arch: syn::Ident,
}

impl Parse for Input {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            kw_arch: input.parse()?,
            eq1: input.parse()?,
            arch: input.parse()?,
        })
    }
}

#[proc_macro_attribute]
pub fn tablegen(attr: TokenStream, input: TokenStream) -> TokenStream {
    let attr = syn::parse_macro_input!(attr as Input);
    let invoke = syn::parse_macro_input!(input as syn::ItemMacro);
    let mut mac = invoke.mac;
    let dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let mut file = PathBuf::from(dir);

    input
}
