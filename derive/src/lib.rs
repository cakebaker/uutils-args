use std::collections::HashMap;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    Attribute,
    Data::Struct,
    DeriveInput, Fields, Ident, Token,
};

#[derive(Eq, Hash, PartialEq, Debug)]
enum Arg {
    Short(char),
    Long(String),
}

enum OptionsAttribute {
    Flag(FlagAttribute),
}

struct FlagAttribute {
    flags: Vec<Arg>,
}

enum FlagArg {
    Short(char),
    Long(String),
}

// FIXME: Think of a better name
#[proc_macro_derive(Options, attributes(flag))]
pub fn options(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    // Used in the quasi-quotation below as `#name`.
    let name = input.ident;

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let Struct(data) = input.data else {
        panic!("Input should be a struct!");
    };

    let Fields::Named(fields) = data.fields else {
        panic!("Fields must be named");
    };

    // The key of this map is a literal pattern and the value
    // is whatever code needs to be run when that pattern is encountered.
    let mut map: HashMap<Arg, Vec<TokenStream2>> = HashMap::new();

    for field in fields.named {
        let field_ident = field.ident.as_ref().expect("Each field must be named.");
        let field_name = field_ident.to_string();
        let field_char = field_name.chars().next().unwrap();
        for attr in field.attrs {
            let Some(attr) = parse_attr(attr) else { continue; };
            match attr {
                OptionsAttribute::Flag(f) => {
                    let flags = if f.flags.is_empty() {
                        if field_name.len() > 1 {
                            vec![Arg::Short(field_char), Arg::Long(field_name.clone())]
                        } else {
                            vec![Arg::Short(field_char)]
                        }
                    } else {
                        f.flags
                    };
                    for flag in flags {
                        map.entry(flag)
                            .or_default()
                            .push(quote!(_self.#field_ident = true;));
                    }
                }
            }
        }
    }

    let mut match_arms = vec![];
    for (pattern, arms) in map {
        match pattern {
            Arg::Short(char) => match_arms.push(quote!(lexopt::Arg::Short(#char) => {#(#arms)*})),
            Arg::Long(string) => match_arms.push(quote!(lexopt::Arg::Long(#string) => {#(#arms)*})),
        }
    }

    let expanded = quote!(
        impl #impl_generics Options for #name #ty_generics #where_clause {
            fn parse<I>(args: I) -> Result<Self, lexopt::Error>
            where
                I: IntoIterator + 'static,
                I::Item: Into<std::ffi::OsString>,
            {
                use uutils_args::lexopt;
                let mut _self = #name::default();

                let mut parser = lexopt::Parser::from_args(args);
                while let Some(arg) = parser.next()? {
                    match arg {
                        #(#match_arms)*,
                        _ => { return Err(arg.unexpected());}
                    }
                }
                Ok(_self)
            }
        }
    );

    TokenStream::from(expanded)
}

fn parse_attr(attr: Attribute) -> Option<OptionsAttribute> {
    if attr.path.is_ident("flag") {
        return Some(OptionsAttribute::Flag(parse_flag_attr(attr)));
    }
    None
}

fn parse_flag_attr(attr: Attribute) -> FlagAttribute {
    let mut flag_attr = FlagAttribute { flags: vec![] };
    let Ok(parsed_args) = attr
        .parse_args_with(Punctuated::<FlagArg, Token![,]>::parse_terminated)
    else {
        return flag_attr;
    };
    for arg in parsed_args {
        match arg {
            FlagArg::Long(s) => flag_attr.flags.push(Arg::Long(s)),
            FlagArg::Short(c) => flag_attr.flags.push(Arg::Short(c)),
        };
    }
    flag_attr
}

impl Parse for FlagArg {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // If the arg starts with a `-`, then it's a flag
        if input.peek(Token![-]) {
            // Eat the first `-` and check for the second one
            // If there is one, it's a long arg, otherwise it's
            // a short arg.
            input.parse::<Token![-]>()?;

            if input.peek(Token![-]) {
                // long flag
                // Eat the second `-` and parse the identifier
                // FIXME: This should accept more than just rust
                // identifiers. For example, we want to support `-`
                // in the name.
                input.parse::<Token![-]>()?;
                let ident = input.parse::<Ident>()?;
                Ok(Self::Long(ident.to_string()))
            } else {
                // short flag
                let ident = input.parse::<Ident>()?;
                let name = ident.to_string();
                assert_eq!(name.len(), 1, "Short flag must be one character long");
                Ok(Self::Short(name.chars().next().unwrap()))
            }
        } else {
            // FIXME: Better error message
            panic!("Argument is not a flag");
        }
    }
}