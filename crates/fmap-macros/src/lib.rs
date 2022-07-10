use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::punctuated::Punctuated;
use syn::{
    parse_macro_input, Data, DeriveInput, Fields, Ident, Path, PathArguments, PathSegment, Type,
};

#[proc_macro_derive(Directory, attributes(file))]
pub fn directory(input: TokenStream) -> TokenStream {
    // Get list of field.
    let input = parse_macro_input!(input as DeriveInput);
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
    let mut files: Vec<&Ident> = Vec::new();
    let mut types: Vec<Path> = Vec::new();
    let mut names: Vec<String> = Vec::new();

    for field in &fields.named {
        let mut file = false;

        for attr in &field.attrs {
            if attr.path.is_ident("file") {
                file = true;
            }
        }

        if file {
            let name = field.ident.as_ref().unwrap();
            let r#type = if let Type::Path(v) = &field.ty {
                let t = &v.path.segments.last().unwrap().ident;
                let mut p = Path {
                    leading_colon: None,
                    segments: Punctuated::new(),
                };

                p.segments.push(PathSegment {
                    ident: format_ident!("fmap"),
                    arguments: PathArguments::None,
                });

                p.segments.push(PathSegment {
                    ident: format_ident!("{}File", t),
                    arguments: PathArguments::None,
                });

                p
            } else {
                panic!("Field {} has invalid type", name)
            };

            files.push(name);
            types.push(r#type);
            names.push(name.to_string());
        }
    }

    // Generate getters.
    let ident = input.ident;
    let generics = input.generics;
    let result = quote! {
        impl #generics #ident #generics {
            #( fn #files <'parent> (&'parent self) -> #types <'parent, Self> { #types::new(self, &self.#files, #names) } )*
        }
    };

    result.into()
}
