use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::punctuated::Punctuated;
use syn::{
    parse_macro_input, Data, DeriveInput, Fields, Lit, Meta, NestedMeta, Token, Type, TypePath,
};

mod pattern;

#[proc_macro_derive(HttpRequest, attributes(get))]
pub fn derive_http_request(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let data = match input.data {
        Data::Enum(v) => v,
        _ => panic!("The HttpRequest can only be appli&ed to enum"),
    };

    // Iterate variants.
    let mut gets: Vec<TokenStream> = Vec::new();
    let mut methods: Vec<TokenStream> = Vec::new();
    let mut formats: Vec<TokenStream> = Vec::new();

    for variant in &data.variants {
        let name = &variant.ident;

        // Get inner fields.
        let params: Vec<&TypePath> = match &variant.fields {
            Fields::Unnamed(fields) => fields
                .unnamed
                .iter()
                .map(|f| {
                    if let Type::Path(t) = &f.ty {
                        t
                    } else {
                        panic!("Invalid route parameter on {}", name);
                    }
                })
                .collect(),
            Fields::Unit => Vec::new(),
            _ => panic!(
                "{} must be a plain or have only one or more unnamed fields",
                name
            ),
        };

        // Process attributes.
        let mut method = "";
        let mut pattern = String::new();

        for attr in &variant.attrs {
            let meta = match attr.parse_meta() {
                Ok(r) => r,
                Err(_) => continue,
            };

            if let Meta::List(list) = &meta {
                if list.path.is_ident("get") {
                    for inner in &list.nested {
                        match inner {
                            NestedMeta::Meta(_) => {}
                            NestedMeta::Lit(inner) => {
                                if let Lit::Str(inner) = inner {
                                    pattern = inner.value();
                                }
                            }
                        }
                    }

                    if pattern.is_empty() {
                        panic!("No pattern is specified on {}", name);
                    }

                    let pattern = match pattern::parse(&pattern) {
                        Ok(r) => r,
                        Err(e) => match e {
                            pattern::ParseError::InvalidPattern => {
                                panic!("{} has invalid pattern", name)
                            }
                        },
                    };

                    let count = pattern.len();
                    let block = generate_pattern_matching(name, &pattern, 0, &params, 0);
                    let matching = quote! {
                        if segments.len() == #count {
                            #block
                        }
                    };

                    gets.push(matching);

                    methods.push(if params.is_empty() {
                        quote! {
                            Self::#name => &http::method::Method::GET,
                        }
                    } else {
                        let mut fields: Punctuated<Ident, Token![,]> = Punctuated::new();

                        for _ in 0..params.len() {
                            fields.push(format_ident!("_"));
                        }

                        quote! {
                            Self::#name(#fields) => &http::method::Method::GET,
                        }
                    });

                    method = "GET";
                    break;
                }
            }
        }

        if pattern.is_empty() {
            panic!("No HTTP method has been specified on {}", name)
        }

        // Generate formatter.
        let mut fields: Punctuated<Ident, Token![,]> = Punctuated::new();

        for i in 0..params.len() {
            fields.push(format_ident!("p{}", i));
        }

        let name = if fields.is_empty() {
            quote! { #name }
        } else {
            quote! { #name(#fields) }
        };

        let format = format!("{} {}", method, pattern);
        let mut args: Punctuated<TokenStream, Token![,]> = Punctuated::new();

        for f in &fields {
            args.push(quote! {
                percent_encoding::utf8_percent_encode(&#f.to_string(), percent_encoding::NON_ALPHANUMERIC)
            });
        }

        formats.push(quote! {
            Self::#name => write!(f, #format, #args),
        });
    }

    // Generate implementation.
    let name = &input.ident;
    let output = quote! {
        impl #name {
            pub fn resolve(method: &http::method::Method, path: &str) -> Option<Self> {
                let segments: Vec<&str> = path.split_terminator('/').skip(1).collect();

                if method == http::method::Method::GET {
                    #( #gets )*
                }

                None
            }

            pub fn method(&self) -> &'static http::method::Method {
                match self {
                    #( #methods )*
                }
            }
        }

        impl std::fmt::Display for #name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                match self {
                    #( #formats )*
                }
            }
        }
    };

    output.into()
}

fn generate_pattern_matching(
    variant: &Ident,
    pattern: &[pattern::Segment],
    segment: usize,
    fields: &[&TypePath],
    param: usize,
) -> TokenStream {
    if segment == pattern.len() {
        if fields.is_empty() {
            quote! {
                return Some(Self::#variant);
            }
        } else {
            let mut params = Punctuated::<Ident, Token![,]>::new();

            for i in 0..fields.len() {
                params.push(format_ident!("param{}", i));
            }

            quote! {
                return Some(Self::#variant(#params));
            }
        }
    } else {
        match pattern[segment] {
            pattern::Segment::Static(value) => {
                let next = generate_pattern_matching(variant, pattern, segment + 1, fields, param);

                quote! {
                    if let Ok(decoded) = percent_encoding::percent_decode_str(segments[#segment]).decode_utf8() {
                        if decoded.as_ref() == #value {
                            #next
                        }
                    }
                }
            }
            pattern::Segment::Param => {
                let name = format_ident!("param{}", param);
                let ty = fields[param];
                let next =
                    generate_pattern_matching(variant, pattern, segment + 1, fields, param + 1);

                quote! {
                    if let Ok(decoded) = percent_encoding::percent_decode_str(segments[#segment]).decode_utf8() {
                        let segment = decoded.as_ref();

                        if !segment.is_empty() {
                            if let Ok(#name) = segment.parse::<#ty>() {
                                #next
                            }
                        }
                    }
                }
            }
        }
    }
}
