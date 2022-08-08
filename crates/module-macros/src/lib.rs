use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput, Error};

mod github;

/// GitHub required User-Agent to be set otherwise we will get 403.
#[proc_macro_derive(GitHubHeaders)]
pub fn derive_github_headers(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    github::derive_headers(input)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}
