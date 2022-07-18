use self::directory::DirectoryParser;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

mod directory;

#[proc_macro_derive(Directory, attributes(directory, file, placeholder))]
pub fn derive_directory(item: TokenStream) -> TokenStream {
    // Get list of field.
    let input = parse_macro_input!(item as DeriveInput);
    let data = if let Data::Struct(v) = &input.data {
        v
    } else {
        panic!("The Container can only apply on struct")
    };

    let fields = if let Fields::Named(v) = &data.fields {
        v
    } else {
        panic!("The Container can only apply on a structs with named fields")
    };

    // Iterate fields.
    let mut parser = DirectoryParser::new();

    for field in &fields.named {
        parser.parse_field(field);
    }

    // Generate implementation.
    let generics = input.generics;
    let name = input.ident;
    let body = parser.generate_body();
    let result = quote! {
        impl #generics #name #generics {
            #body
        }
    };

    result.into()
}
