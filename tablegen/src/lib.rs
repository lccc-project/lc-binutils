use std::{fs::File, io::ErrorKind, path::PathBuf, process::Command};

use proc_macro::TokenStream;

use proc_macro2::{Delimiter, Group, Ident, Literal, Span, TokenTree};
use quote::ToTokens;
use serde_json::Value;
use syn::{parse::Parse, punctuated::Punctuated, token::Bracket};

mod kw {
    syn::custom_keyword!(arch);
}

struct Fields {
    pub bracket: syn::token::Bracket,
    pub fields: Punctuated<syn::Ident, syn::Token![,]>,
}

impl Parse for Fields {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let buff;
        let bracket = syn::bracketed!(buff in input);
        let fields = Punctuated::parse_terminated(&buff)?;
        Ok(Self { bracket, fields })
    }
}

struct FieldOutput {
    pub bracket: syn::token::Bracket,
    pub name: syn::Ident,
    pub comma: syn::Token![,],
    pub fields: Punctuated<proc_macro2::TokenStream, syn::Token![,]>,
}

impl ToTokens for FieldOutput {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = &self.name;
        let comma = &self.comma;
        let fields = &self.fields;

        tokens.extend(quote::quote! {[#name #comma #fields]})
    }
}

struct Input {
    pub kw_arch: kw::arch,
    pub eq1: syn::Token![=],
    pub arch: syn::Ident,
}

struct MacroInvoke {
    pub class: syn::Ident,
    pub fields: Fields,
}

impl Parse for MacroInvoke {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            class: input.parse()?,
            fields: input.parse()?,
        })
    }
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

fn value_to_tokens(v: &Value, s: Span) -> proc_macro2::TokenStream {
    match v {
        Value::Null => proc_macro2::TokenStream::new(),
        Value::Bool(true) => {
            let tt = TokenTree::Ident(Ident::new("true", s));
            tt.into()
        }
        Value::Bool(false) => {
            let tt = TokenTree::Ident(Ident::new("false", s));
            tt.into()
        }
        Value::Number(i) => {
            let lit;
            if i.is_u64() {
                lit = Literal::u64_unsuffixed(i.as_u64().unwrap());
            } else if i.is_i64() {
                lit = Literal::i64_unsuffixed(i.as_i64().unwrap());
            } else {
                lit = Literal::f64_unsuffixed(i.as_f64().unwrap());
            }

            TokenTree::Literal(lit).into()
        }
        Value::String(s) => TokenTree::Literal(Literal::string(&s)).into(),
        Value::Array(v) => {
            let st = v
                .iter()
                .map(|v| value_to_tokens(v, s))
                .collect::<Punctuated<proc_macro2::TokenStream, syn::Token![,]>>()
                .into_token_stream();

            TokenTree::Group(Group::new(Delimiter::Bracket, st)).into()
        }
        Value::Object(_) => panic!("Objects are not implemented yet"),
    }
}

#[proc_macro_attribute]
pub fn tablegen(attr: TokenStream, input: TokenStream) -> TokenStream {
    let attr = syn::parse_macro_input!(attr as Input);
    let arch = attr.arch.clone();
    let invoke = syn::parse_macro_input!(input as syn::ItemMacro);
    let mac = invoke.mac;
    let dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let mut file = PathBuf::from(&dir);
    let path = mac.path;
    let invoke: MacroInvoke = syn::parse2(mac.tokens).unwrap();
    file.push("generator");
    let generator_dir = file.clone();
    file.push(attr.arch.to_string());
    let mut source = file.clone();
    file.set_extension("td.json");
    source.set_extension("td");
    match (std::fs::metadata(&source), std::fs::metadata(&file)) {
        (Ok(_), Err(e)) if e.kind() == ErrorKind::NotFound => {
            if let Ok(s) = which::which("llvm-tblgen") {
                let output = File::create(&file).unwrap();

                match Command::new(s)
                    .arg("--dump-json")
                    .arg(&source)
                    .stdout(output)
                    .current_dir(generator_dir)
                    .status()
                {
                    Ok(s) => {
                        if !s.success() {
                            let _ = std::fs::remove_file(&file);
                            panic!(
                                "Failed to evaluate tablegen in {} for {}: llvm-tblgen returned an error",
                                &dir,
                                &attr.arch
                            );
                        }
                    }
                    Err(e) => {
                        let _ = std::fs::remove_file(&file);
                        panic!(
                            "Failed to evaluate tablegen in {} for {}: {}",
                            &dir, &attr.arch, e
                        )
                    }
                }
            } else {
                panic!(
                    "Cannot evaluate tablegen in {} for {}: No llvm-tblgen program ",
                    &dir, &attr.arch
                )
            }
        }
        (Ok(m1), Ok(m2))
            if m1
                .modified()
                .map_err(|_| ())
                .and_then(|d| Ok(d < m2.modified().map_err(|_| ())?))
                != Ok(true) =>
        {
            if let Ok(s) = which::which("llvm-tblgen") {
                let output = File::create(&file).unwrap();

                match Command::new(s)
                    .arg("--dump-json")
                    .arg(&source)
                    .stdout(output)
                    .current_dir(generator_dir)
                    .status()
                {
                    Ok(s) => {
                        if !s.success() {
                            panic!(
                                "Failed to evaluate tablegen for {}: llvm-tblgen returned an error",
                                &attr.arch
                            );
                        }
                    }
                    Err(e) => {
                        panic!("Failed to evaluate tablegen for {}: {}", &attr.arch, e);
                    }
                }
            } else {
                /* warning case: If we had it, emit a warning here. Since we don't, do nothing */
            }
        }
        (Err(e), _) | (_, Err(e)) => {
            panic!(
                "Cannot evaluate tablegen in {} for {}: {}",
                &dir, &attr.arch, e
            )
        }
        _ => {}
    }

    let file = File::open(file).expect("Cannot evaluate tablegen");
    let json: serde_json::Value = serde_json::from_reader(file).expect("Cannot evaluate tablegen");
    let mut output_fields = Punctuated::<FieldOutput, syn::Token![,]>::new();
    if let Some(a) = json["!instanceof"][invoke.class.to_string()].as_array() {
        for v in a {
            let name = v.as_str().expect("No !name from tablegen class");
            let mut f = FieldOutput {
                bracket: Bracket::default(),
                name: syn::Ident::new(name, proc_macro2::Span::call_site()),
                comma: Default::default(),
                fields: Punctuated::new(),
            };
            for k in &invoke.fields.fields {
                let field = &json[name][k.to_string()];
                f.fields.push(value_to_tokens(field, Span::call_site()));
            }
            output_fields.push(f)
        }
    }

    quote::quote!(
        #path!(#arch, #output_fields);
    )
    .into()
}
