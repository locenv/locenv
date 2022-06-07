use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Config)]
pub fn derive_config(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = ast.ident;
    let parser = quote! {
        impl #name {
            pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self, crate::FromFileError> {
                let file = match std::fs::File::open(&path) {
                    Ok(r) => r,
                    Err(e) => return Err(crate::FromFileError::OpenFailed(e)),
                };

                let config = match Self::from_reader(file) {
                    Ok(r) => r,
                    Err(e) => return Err(crate::FromFileError::ParseFailed(e)),
                };

                Ok(config)
            }

            pub fn from_reader<R: std::io::Read>(reader: R) -> Result<Self, serde_yaml::Error> {
                serde_yaml::from_reader(reader)
            }
        }
    };

    parser.into()
}
