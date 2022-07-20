use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{format_ident, quote};
use syn::punctuated::Punctuated;
use syn::{
    parse_macro_input, Data, DeriveInput, Fields, Lit, Meta, NestedMeta, Token, Type, TypePath,
};

mod pattern;

#[proc_macro_derive(HttpRequest, attributes(get))]
pub fn derive_http_request(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let data = match input.data {
        Data::Enum(v) => v,
        _ => panic!("The HttpRequest can only be appli&ed to enum"),
    };

    // Iterate variants.
    let mut gets: Vec<proc_macro2::TokenStream> = Vec::new();

    for variant in &data.variants {
        // Process attributes.
        for attr in &variant.attrs {
            let meta = match attr.parse_meta() {
                Ok(r) => r,
                Err(_) => continue,
            };

            if let Meta::List(list) = &meta {
                if list.path.is_ident("get") {
                    let mut pattern: Option<String> = None;

                    for inner in &list.nested {
                        match inner {
                            NestedMeta::Meta(_) => {}
                            NestedMeta::Lit(inner) => {
                                if let Lit::Str(inner) = inner {
                                    pattern = Some(inner.value());
                                }
                            }
                        }
                    }

                    let pattern = if let Some(pattern) = &pattern {
                        match pattern::parse(pattern) {
                            Ok(r) => r,
                            Err(e) => match e {
                                pattern::ParseError::InvalidPattern => {
                                    panic!("{} has invalid pattern", variant.ident)
                                }
                            },
                        }
                    } else {
                        panic!("No pattern is specified on {}", variant.ident);
                    };

                    let params: Vec<&TypePath> = match &variant.fields {
                        Fields::Unnamed(inner) => inner
                            .unnamed
                            .iter()
                            .map(|f| {
                                if let Type::Path(v) = &f.ty {
                                    v
                                } else {
                                    panic!("Invalid route parameter on {}", variant.ident);
                                }
                            })
                            .collect(),
                        Fields::Unit => Vec::new(),
                        _ => panic!(
                            "{} must be a plain or have a only one or more unnamed fields",
                            variant.ident
                        ),
                    };

                    let count = pattern.len();
                    let block = generate_pattern_matching(&variant.ident, &pattern, 0, &params, 0);
                    let matching = quote! {
                        if segments.len() == #count {
                            #block
                        }
                    };

                    gets.push(matching);
                }
            }
        }
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
) -> proc_macro2::TokenStream {
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
                    if segments[#segment] == #value {
                        #next
                    }
                }
            }
            pattern::Segment::Param => {
                let name = format_ident!("param{}", param);
                let ty = fields[param];
                let next =
                    generate_pattern_matching(variant, pattern, segment + 1, fields, param + 1);

                quote! {
                    let segment = segments[#segment];

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
