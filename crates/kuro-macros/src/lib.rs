use proc_macro::TokenStream;
use syn::{parse_macro_input, AttributeArgs, DeriveInput, Error, ItemStruct};

mod general;

#[proc_macro_attribute]
pub fn kuro(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let input = parse_macro_input!(input as ItemStruct);

    general::kuro(args, input)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

#[proc_macro_derive(FollowLocation)]
pub fn derive_follow_location(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    general::follow_location(input)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}
