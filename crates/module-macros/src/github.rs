use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

pub fn derive_headers(input: DeriveInput) -> syn::Result<TokenStream> {
    let name = &input.ident;
    let generics = &input.generics;

    Ok(quote! {
        impl #generics kuro::DefaultHeaders for #name #generics {
            fn default_request_headers<'a>(&self) -> kuro::Headers<'a> {
                kuro::Headers{
                    content_length: None,
                    user_agent: Some("locenv"),
                    accept: None,
                }
            }
        }
    })
}
