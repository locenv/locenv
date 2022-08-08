use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{AttributeArgs, DeriveInput, Error, ItemStruct, Lit, Meta, NestedMeta};

pub fn kuro(args: AttributeArgs, item: ItemStruct) -> syn::Result<TokenStream> {
    let name = &item.ident;
    let generics = &item.generics;
    let mut outputs: Vec<TokenStream> = Vec::new();

    for arg in &args {
        let arg = match arg {
            NestedMeta::Meta(m) => parse_kuro_arg_meta(m)?,
            _ => return Err(Error::new(Span::call_site(), "unknown argument")),
        };

        match arg {
            KuroArg::Error(ty) => {
                let ty = format_ident!("{}", ty);

                outputs.push(quote! {
                    impl #generics kuro::Error for #name #generics {
                        type Err = #ty;
                    }
                })
            }
        }
    }

    Ok(quote! {
        #item

        #( #outputs )*
    })
}

pub fn follow_location(input: DeriveInput) -> syn::Result<TokenStream> {
    let name = &input.ident;
    let generics = &input.generics;

    Ok(quote! {
        impl #generics kuro::FollowLocation for #name #generics {
            fn follow_location(&self) -> bool {
                true
            }
        }
    })
}

fn parse_kuro_arg_meta(meta: &Meta) -> syn::Result<KuroArg> {
    match meta {
        Meta::NameValue(p) => {
            if p.path.is_ident("error") {
                match &p.lit {
                    Lit::Str(v) => Ok(KuroArg::Error(v.value())),
                    _ => Err(Error::new(Span::call_site(), "invalid value for `error`")),
                }
            } else {
                Err(Error::new(Span::call_site(), "unknown argument"))
            }
        }
        _ => Err(Error::new(Span::call_site(), "unknown argument")),
    }
}

enum KuroArg {
    Error(String),
}
