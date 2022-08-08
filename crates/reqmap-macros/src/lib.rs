use self::request::RequestParser;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Error};

mod request;

#[proc_macro_derive(HttpRequest, attributes(delete, get, patch, post, put))]
pub fn derive_http_request(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    implement_http_request(input)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

fn implement_http_request(input: DeriveInput) -> syn::Result<TokenStream> {
    // Check if we are on enum.
    let data = match &input.data {
        Data::Enum(v) => v,
        _ => return Err(Error::new(input.ident.span(), "expected enum")),
    };

    // Parse variants.
    let mut parser = RequestParser::new();

    for variant in &data.variants {
        parser.parse(variant)?;
    }

    // Generate implementation.
    let name = &input.ident;
    let matchers = parser.matchers();
    let methods = parser.methods();
    let paths = parser.paths();
    let formats = parser.formats();
    let output = quote! {
        impl #name {
            pub fn resolve(method: &http::method::Method, path: &str) -> Option<Self> {
                let segments: Vec<&str> = path.split_terminator('/').skip(1).collect();

                #( #matchers )*

                None
            }

            pub fn method(&self) -> &'static http::method::Method {
                match self {
                    #( #methods )*
                }
            }

            pub fn path(&self) -> std::borrow::Cow<'static, str> {
                match self {
                    # ( #paths )*
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

    Ok(output)
}
