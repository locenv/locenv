use self::matcher::MatcherBuilder;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::fmt::{Display, Formatter};
use syn::punctuated::Punctuated;
use syn::{Attribute, Error, Fields, Ident, Lit, Meta, NestedMeta, Token, Type, TypePath, Variant};

mod matcher;
mod pattern;

pub struct RequestParser {
    matchers: Vec<TokenStream>,
    methods: Vec<TokenStream>,
    paths: Vec<TokenStream>,
    formats: Vec<TokenStream>,
}

impl RequestParser {
    pub fn new() -> Self {
        Self {
            matchers: Vec::new(),
            methods: Vec::new(),
            paths: Vec::new(),
            formats: Vec::new(),
        }
    }

    pub fn matchers(&self) -> &[TokenStream] {
        self.matchers.as_ref()
    }

    pub fn methods(&self) -> &[TokenStream] {
        self.methods.as_ref()
    }

    pub fn paths(&self) -> &[TokenStream] {
        self.paths.as_ref()
    }

    pub fn formats(&self) -> &[TokenStream] {
        self.formats.as_ref()
    }

    pub fn parse(&mut self, variant: &Variant) -> syn::Result<()> {
        let name = &variant.ident;

        // Get fields.
        let params: Vec<&TypePath> = match &variant.fields {
            Fields::Unnamed(fields) => {
                let mut types: Vec<&TypePath> = Vec::with_capacity(fields.unnamed.len());

                for f in &fields.unnamed {
                    types.push(if let Type::Path(t) = &f.ty {
                        t
                    } else {
                        return Err(Error::new(name.span(), "unknow route parameter(s)"));
                    });
                }

                types
            }
            Fields::Unit => Vec::new(),
            _ => {
                return Err(Error::new(
                    name.span(),
                    "expected variant with unnamed fields or no data",
                ));
            }
        };

        // Process attributes.
        let attr = match parse_attributes(name, &variant.attrs, &params)? {
            Some(v) => v,
            None => return Err(Error::new(name.span(), "expected at least one HTTP method")),
        };

        // Get HTTP method identifier.
        let method = match attr.method {
            HttpMethod::Delete => quote! { http::method::Method::DELETE },
            HttpMethod::Get => quote! { http::method::Method::GET },
            HttpMethod::Patch => quote! { http::method::Method::PATCH },
            HttpMethod::Post => quote! { http::method::Method::POST },
            HttpMethod::Put => quote! { http::method::Method::PUT },
        };

        // Build matcher.
        self.matchers.push({
            let matcher = attr.matcher;

            quote! {
                if method == #method {
                    #matcher
                }
            }
        });

        // Build method getter.
        self.methods.push(if params.is_empty() {
            quote! {
                Self::#name => &#method,
            }
        } else {
            let mut fields: Punctuated<Ident, Token![,]> = Punctuated::new();

            for _ in 0..params.len() {
                fields.push(format_ident!("_"));
            }

            quote! {
                Self::#name(#fields) => &#method,
            }
        });

        // Build path and formatter.
        let mut fields: Punctuated<Ident, Token![,]> = Punctuated::new();

        for i in 0..params.len() {
            fields.push(format_ident!("p{}", i));
        }

        let name = if fields.is_empty() {
            quote! { #name }
        } else {
            quote! { #name(#fields) }
        };

        self.paths.push({
            let pattern = &attr.pattern;
            let path = if fields.is_empty() {
                quote! {
                    #pattern.into()
                }
            } else {
                quote! {
                    format!(#pattern, #fields).into()
                }
            };

            quote! {
                Self::#name => #path,
            }
        });

        self.formats.push({
            let format = format!("{} {}", attr.method, attr.pattern);
            let mut args: Punctuated<TokenStream, Token![,]> = Punctuated::new();

            for f in &fields {
                args.push(quote! {
                    percent_encoding::utf8_percent_encode(&#f.to_string(), percent_encoding::NON_ALPHANUMERIC)
                });
            }

            quote! {
                Self::#name => write!(f, #format, #args),
            }
        });

        Ok(())
    }
}

fn parse_attributes(
    variant: &Ident,
    attrs: &[Attribute],
    params: &[&TypePath],
) -> syn::Result<Option<AttributeData>> {
    for attr in attrs {
        let meta = match attr.parse_meta() {
            Ok(r) => r,
            Err(_) => continue,
        };

        // Get HTTP method.
        let (method, items) = if let Meta::List(m) = &meta {
            let method = if m.path.is_ident("delete") {
                HttpMethod::Delete
            } else if m.path.is_ident("get") {
                HttpMethod::Get
            } else if m.path.is_ident("patch") {
                HttpMethod::Patch
            } else if m.path.is_ident("post") {
                HttpMethod::Post
            } else if m.path.is_ident("put") {
                HttpMethod::Put
            } else {
                continue;
            };

            (method, &m.nested)
        } else {
            continue;
        };

        // Process attribute's items (e.g. #[get("/foo", ...)])
        //                                       ^^^^^^^^^^^
        let mut pattern = String::new();

        for i in items {
            match i {
                NestedMeta::Meta(_) => {}
                NestedMeta::Lit(i) => {
                    if let Lit::Str(i) = i {
                        pattern = i.value();
                    }
                }
            }
        }

        if pattern.is_empty() {
            return Err(Error::new(variant.span(), "no path pattern is specified"));
        }

        // Parse pattern.
        let segments = match self::pattern::parse(&pattern) {
            Ok(r) => r,
            Err(e) => match e {
                self::pattern::ParseError::InvalidPattern => {
                    return Err(Error::new(variant.span(), "invalid path pattern"));
                }
            },
        };

        // Build pattern matcher.
        let builder = MatcherBuilder::new(variant, &segments, params);
        let matcher = match builder.build() {
            Ok(r) => r,
            Err(e) => match e {
                self::matcher::BuilderError::PatternAndParametersMismatched => {
                    return Err(Error::new(
                        variant.span(),
                        "number of parameters and fields mismatched",
                    ));
                }
            },
        };

        // Wrap matcher in a length checking.
        let count = segments.len();
        let matcher = quote! {
            if segments.len() == #count {
                #matcher
            }
        };

        return Ok(Some(AttributeData {
            method,
            pattern,
            matcher,
        }));
    }

    Ok(None)
}

enum HttpMethod {
    Delete,
    Get,
    Patch,
    Post,
    Put,
}

impl Display for HttpMethod {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Self::Delete => f.write_str("DELETE"),
            Self::Get => f.write_str("GET"),
            Self::Patch => f.write_str("PATCH"),
            Self::Post => f.write_str("POST"),
            Self::Put => f.write_str("PUT"),
        }
    }
}

struct AttributeData {
    method: HttpMethod,
    pattern: String,
    matcher: TokenStream,
}
