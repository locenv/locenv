use self::directory::DirectoryParser;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Error, Fields};

mod directory;

#[proc_macro_derive(Directory, attributes(directory, file, placeholder))]
pub fn derive_directory(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    implement_directory(input)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

fn implement_directory(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let name = &input.ident;
    let data = if let Data::Struct(v) = &input.data {
        v
    } else {
        return Err(Error::new(name.span(), "expected struct"));
    };

    let fields = if let Fields::Named(v) = &data.fields {
        v
    } else {
        return Err(Error::new(
            name.span(),
            "expected struct with named field(s)",
        ));
    };

    // Iterate fields.
    let mut parser = DirectoryParser::new();

    for field in &fields.named {
        parser.parse_field(field);
    }

    // Generate implementation.
    let generics = &input.generics;
    let body = parser.generate_body();
    let result = quote! {
        impl #generics #name #generics {
            #body
        }
    };

    Ok(result)
}
